use std::collections::HashMap;
use std::sync::Mutex;
use crate::types::{RasRpcCommand, RasCoreEvent, Dag, PendingToolCall, OrchestratorState};
use crate::call_host;
use crate::tool::{Message, ToolCall, ToolCallFunction};

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

fn process_completed_tool_calls(pending: Vec<PendingToolCall>) -> Result<(), String> {
    for tc in pending {
        let result_content = tc.result.unwrap_or_else(|| "No execution result.".to_string());
        let tool_msg = Message {
            role: "tool".to_string(),
            content: Some(result_content),
            name: Some(tc.name.clone()),
            tool_call_id: Some(tc.id.clone()),
            tool_calls: None,
        };
        let tool_text = serde_json::to_string(&tool_msg).map_err(|e| format!("Failed to serialize tool message: {e}"))?;
        let dag_val = call_host(RasRpcCommand::GetDag)?;
        let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
        let parent_id = dag.current_node_id.unwrap_or_default();

        let node_id_val = call_host(RasRpcCommand::CreateNode { parent_id, node_type: "tool".to_string() })?;
        let node_id = node_id_val.as_str().ok_or("Failed to get node id as string")?;
        call_host(RasRpcCommand::SetNodeText { node_id: node_id.to_string(), text: tool_text })?;
    }

    let messages = crate::llm::load_messages_from_dag()?;
    {
        let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
        state_guard.get_or_insert_with(|| OrchestratorState {
            assistant: String::new(),
            stream: String::new(),
            tool_calls: HashMap::new(),
            pending_tool_calls: Vec::new(),
        });
    }
    crate::llm::trigger_llm_stream(messages)
}

fn handle_human_input(text: String) -> Result<(), String> {
    {
        let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
        *state_guard = Some(OrchestratorState {
            assistant: String::new(),
            stream: String::new(),
            tool_calls: HashMap::new(),
            pending_tool_calls: Vec::new(),
        });
    }
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let parent_id = dag.current_node_id.unwrap_or_default();
    let user_node_id_val = call_host(RasRpcCommand::CreateNode { parent_id, node_type: "user".to_string() })?;
    let user_node_id = user_node_id_val.as_str().ok_or("Failed to get node id as string")?;
    call_host(RasRpcCommand::SetNodeText { node_id: user_node_id.to_string(), text })?;

    let messages = crate::llm::load_messages_from_dag()?;
    crate::llm::trigger_llm_stream(messages)
}

fn append_process_output(pgid: i32, data: &[u8], is_stderr: bool) -> Result<(), String> {
    let text = String::from_utf8_lossy(data);
    if text.contains("CRASH_WASM") {
        panic!("Simulated Wasm Crash");
    }
    let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
    if let Some(state) = state_guard.as_mut() {
        for tc in &mut state.pending_tool_calls {
            if tc.pgid == Some(pgid) {
                if is_stderr {
                    tc.stderr.extend_from_slice(data);
                } else {
                    tc.stdout.extend_from_slice(data);
                }
            }
        }
    }
    Ok(())
}

fn handle_process_exited(pgid: i32, exit_code: Option<i32>) -> Result<(), String> {
    let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
    if let Some(state) = state_guard.as_mut() {
        let mut found = false;
        for tc in &mut state.pending_tool_calls {
            if tc.pgid == Some(pgid) {
                let out_str = String::from_utf8_lossy(&tc.stdout).to_string();
                let err_str = String::from_utf8_lossy(&tc.stderr).to_string();
                tc.result = Some(format!("Command exited with code {exit_code:?}.\nStdout:\n{out_str}\nStderr:\n{err_str}"));
                found = true;
            }
        }
        if found && state.pending_tool_calls.iter().all(|tc| tc.result.is_some()) {
            let pending = std::mem::take(&mut state.pending_tool_calls);
            drop(state_guard);
            process_completed_tool_calls(pending)?;
        }
    }
    Ok(())
}

pub fn handle_event(event: RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::HumanInputReceived { text } => handle_human_input(text),
        RasCoreEvent::HttpChunkReceived { chunk } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let mut done = false;
            if let Some(state) = state_guard.as_mut() {
                state.stream.push_str(&chunk);
                done = crate::sse::process_sse_buffer(state)?;
            }
            if done {
                handle_done(state_guard)?;
            }
            Ok(())
        }
        RasCoreEvent::HttpErrorReceived { message } => {
            let error_text = format!("\n\x1b[1;31mLLM Stream Error: {message}\x1b[0m\n");
            let _ = call_host(RasRpcCommand::WriteStdout { text: error_text })?;
            let _ = call_host(RasRpcCommand::CompleteTask)?;
            Ok(())
        }
        RasCoreEvent::ProcessStdout { pgid, data } => append_process_output(pgid, &data, false),
        RasCoreEvent::ProcessStderr { pgid, data } => append_process_output(pgid, &data, true),
        RasCoreEvent::ProcessExited { pgid, exit_code } => handle_process_exited(pgid, exit_code),
        RasCoreEvent::Rehydrate { active_calls } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = state_guard.get_or_insert_with(|| OrchestratorState {
                assistant: String::new(),
                stream: String::new(),
                tool_calls: HashMap::new(),
                pending_tool_calls: Vec::new(),
            });
            state.pending_tool_calls.clear();
            for call in active_calls {
                state.pending_tool_calls.push(PendingToolCall {
                    id: call.id,
                    name: call.name,
                    arguments: call.arguments,
                    pgid: call.pgid,
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    result: None,
                });
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn execute_and_collect_tools(
    pending_calls: Vec<PendingToolCall>,
) -> Result<(), String> {
    let mut executed_calls = Vec::new();
    let mut all_sync_done = true;

    for mut tc in pending_calls {
        match crate::tool::execute_tool(&tc.name, &tc.arguments) {
            Ok(crate::tool::ToolExecutionResult::Sync(res)) => {
                tc.result = Some(res);
                executed_calls.push(tc);
            }
            Ok(crate::tool::ToolExecutionResult::Async(pgid)) => {
                tc.pgid = Some(pgid);
                executed_calls.push(tc);
                all_sync_done = false;
            }
            Err(e) => {
                tc.result = Some(format!("Error: {e}"));
                executed_calls.push(tc);
            }
        }
    }

    let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
    let state = state_guard.as_mut().ok_or("State is None in execute_and_collect_tools")?;
    state.pending_tool_calls.extend(executed_calls);

    if all_sync_done {
        let pending = std::mem::take(&mut state.pending_tool_calls);
        drop(state_guard);
        process_completed_tool_calls(pending)?;
    }
    Ok(())
}

type MutexGuard<'a, T> = std::sync::MutexGuard<'a, T>;

fn extract_tool_calls(
    state: &mut OrchestratorState
) -> (Vec<ToolCall>, Vec<PendingToolCall>) {
    let mut tool_indices: Vec<usize> = state.tool_calls.keys().copied().collect();
    tool_indices.sort_unstable();
    
    let mut assistant_tool_calls = Vec::new();
    let mut pending_calls = Vec::new();

    for idx in tool_indices {
        if let Some(tool_call) = state.tool_calls.get(&idx) {
            assistant_tool_calls.push(ToolCall {
                id: tool_call.id.clone(),
                tool_type: "function".to_string(),
                function: ToolCallFunction { name: tool_call.name.clone(), arguments: tool_call.arguments.clone() },
            });
            pending_calls.push(PendingToolCall {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                arguments: tool_call.arguments.clone(),
                pgid: None,
                stdout: Vec::new(),
                stderr: Vec::new(),
                result: None,
            });
        }
    }
    state.tool_calls.clear();
    (assistant_tool_calls, pending_calls)
}

fn handle_done(mut state_guard: MutexGuard<'_, Option<OrchestratorState>>) -> Result<(), String> {
    let state = state_guard.as_mut().ok_or("State is None in handle_done")?;
    let (assistant_tool_calls, pending_calls) = extract_tool_calls(state);

    let assistant_content = if state.assistant.is_empty() { None } else { Some(state.assistant.clone()) };
    state.assistant.clear();

    drop(state_guard);

    let _ = call_host(RasRpcCommand::WriteStdout { text: "\n".to_string() })?;

    let assistant_msg = Message {
        role: "assistant".to_string(),
        content: assistant_content,
        name: None,
        tool_call_id: None,
        tool_calls: if assistant_tool_calls.is_empty() { None } else { Some(assistant_tool_calls) },
    };
    let assistant_text = serde_json::to_string(&assistant_msg).map_err(|e| format!("Failed to serialize assistant message: {e}"))?;
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let parent_id = dag.current_node_id.unwrap_or_default();

    let assistant_node_id_val = call_host(RasRpcCommand::CreateNode { parent_id, node_type: "assistant".to_string() })?;
    let assistant_node_id = assistant_node_id_val.as_str().ok_or("Failed to get node id as string")?;
    call_host(RasRpcCommand::SetNodeText { node_id: assistant_node_id.to_string(), text: assistant_text })?;

    if pending_calls.is_empty() {
        let _ = call_host(RasRpcCommand::CompleteTask)?;
    } else {
        execute_and_collect_tools(pending_calls)?;
    }
    Ok(())
}
