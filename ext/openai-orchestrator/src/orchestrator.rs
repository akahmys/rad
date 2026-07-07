use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use crate::types::{RasRpcCommand, RasCoreEvent, Dag, PendingToolCall, OrchestratorState};
use crate::call_host;
use crate::tool::Message;

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

fn handle_human_input(text: String) -> Result<(), String> {
    crate::mcp_client::init_mcp_servers()?;
    let mcp_names = crate::mcp_client::get_configured_mcp_names()?;

    {
        let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
        *state_guard = Some(OrchestratorState {
            assistant: String::new(),
            stream: String::new(),
            is_reasoning: false,
            reasoning_buffered: String::new(),
            tool_calls: HashMap::new(),
            pending_tool_calls: Vec::new(),
            expected_mcp_servers: mcp_names.clone(),
            mcp_tools: Vec::new(),
            mcp_tool_providers: HashMap::new(),
            max_history_messages: None,
            max_tool_output_chars: None,
        });
    }
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let parent_id = dag.current_node_id.unwrap_or_default();
    let user_node_id_val = call_host(RasRpcCommand::CreateNode { parent_id, node_type: "user".to_string() })?;
    let user_node_id = user_node_id_val.as_str().ok_or("Failed to get node id as string")?;
    call_host(RasRpcCommand::SetNodeText { node_id: user_node_id.to_string(), text })?;

    if mcp_names.is_empty() {
        let messages = crate::llm::load_messages_from_dag()?;
        crate::llm::trigger_llm_stream(messages)
    } else {
        for name in mcp_names {
            let req = serde_json::json!({
                "jsonrpc": "2.0",
                "id": "list_tools",
                "method": "tools/list",
                "params": {}
            });
            let msg_str = serde_json::to_string(&req)
                .map_err(|e| format!("Failed to serialize tools/list request: {e}"))?;
            let _ = call_host(RasRpcCommand::SendMcpRequest {
                name,
                message: msg_str,
            })?;
        }
        Ok(())
    }
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
            crate::tool_runner::process_completed_tool_calls(pending)?;
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
        RasCoreEvent::McpResponse { name, message } => {
            let is_list_tools = message.contains("\"id\":\"list_tools\"") || message.contains("\"id\": \"list_tools\"");
            if is_list_tools {
                let mcp_tools = crate::mcp_client::parse_mcp_tools(&message)?;
                let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
                if let Some(state) = state_guard.as_mut() {
                    for t in &mcp_tools {
                        state.mcp_tool_providers.insert(t.function.name.clone(), name.clone());
                    }
                    state.mcp_tools.extend(mcp_tools);
                    state.expected_mcp_servers.retain(|s| s != &name);
                    if state.expected_mcp_servers.is_empty() {
                        drop(state_guard);
                        let messages = crate::llm::load_messages_from_dag()?;
                        crate::llm::trigger_llm_stream(messages)?;
                    }
                }
            } else {
                let (tool_call_id, result_str) = crate::mcp_client::parse_mcp_call_response(&message)?;
                let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
                if let Some(state) = state_guard.as_mut() {
                    let mut found = false;
                    for tc in &mut state.pending_tool_calls {
                        if tc.id == tool_call_id {
                            tc.result = Some(result_str.clone());
                            found = true;
                        }
                    }
                    if found && state.pending_tool_calls.iter().all(|tc| tc.result.is_some()) {
                        let pending = std::mem::take(&mut state.pending_tool_calls);
                        drop(state_guard);
                        crate::tool_runner::process_completed_tool_calls(pending)?;
                    }
                }
            }
            Ok(())
        }
        RasCoreEvent::Rehydrate { active_calls } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = state_guard.get_or_insert_with(|| OrchestratorState {
                assistant: String::new(),
                stream: String::new(),
                is_reasoning: false,
                reasoning_buffered: String::new(),
                tool_calls: HashMap::new(),
                pending_tool_calls: Vec::new(),
                expected_mcp_servers: Vec::new(),
                mcp_tools: Vec::new(),
                mcp_tool_providers: HashMap::new(),
                max_history_messages: None,
                max_tool_output_chars: None,
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

fn handle_done(mut state_guard: MutexGuard<'_, Option<OrchestratorState>>) -> Result<(), String> {
    let state = state_guard.as_mut().ok_or("State is None in handle_done")?;
    if state.is_reasoning {
        let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string() });
        state.is_reasoning = false;
    }

    let (assistant_tool_calls, pending_calls) = crate::tool_runner::extract_tool_calls(state);

    let assistant_content = if state.assistant.is_empty() { None } else { Some(state.assistant.clone()) };
    state.assistant.clear();
    state.reasoning_buffered.clear();

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
        crate::tool_runner::execute_and_collect_tools(pending_calls)?;
    }
    Ok(())
}
