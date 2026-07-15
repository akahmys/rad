use crate::tool::{Message, ToolCall, ToolCallFunction, execute_tool_sync};
use crate::types::{Dag, OrchestratorState, PendingToolCall, RasCoreEvent, RasRpcCommand};
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

fn trim_large_output(text: &str) -> String {
    let max_chars = STATE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|s| s.max_tool_output_chars))
        .unwrap_or(2000);

    if text.len() <= max_chars {
        return text.to_string();
    }

    let head_len = max_chars / 4;
    let tail_len = max_chars - head_len;

    let head: String = text.chars().take(head_len).collect();
    let tail: String = text
        .chars()
        .rev()
        .take(tail_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    format!(
        "{head}\n\n[ERROR: THIS PART IS TRUNCATED. YOU MUST READ THIS RANGE SEPARATELY BEFORE EDITING ({} characters saved)]\n\n{tail}",
        text.len() - max_chars
    )
}

fn handle_human_input(text: String) -> Result<(), String> {
    {
        let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
        *state_guard = Some(OrchestratorState {
            assistant: String::new(),
            stream: String::new(),
            is_reasoning: false,
            reasoning_buffered: String::new(),
            tool_calls: HashMap::new(),
            max_history_messages: Some(50),
            max_tool_output_chars: Some(2000),
            is_rehydrated: false,
        });
    }

    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag =
        serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let parent_id = dag.current_node_id.unwrap_or_default();
    let user_node_id_val = call_host(RasRpcCommand::CreateNode {
        parent_id,
        node_type: "user".to_string(),
    })?;
    let user_node_id = user_node_id_val
        .as_str()
        .ok_or("Failed to get node id as string")?;
    call_host(RasRpcCommand::SetNodeText {
        node_id: user_node_id.to_string(),
        text,
    })?;

    let messages = crate::llm::load_messages_from_dag()?;
    crate::llm::trigger_llm_stream(messages)
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
        RasCoreEvent::Rehydrate { active_calls } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = OrchestratorState {
                assistant: String::new(),
                stream: String::new(),
                is_reasoning: false,
                reasoning_buffered: String::new(),
                tool_calls: std::collections::HashMap::new(),
                max_history_messages: Some(50),
                max_tool_output_chars: Some(2000),
                is_rehydrated: true,
            };
            *state_guard = Some(state);
            drop(state_guard);

            // Re-execute rehydrated active tool calls
            if !active_calls.is_empty() {
                for call in active_calls {
                    let result_raw = match crate::tool::execute_tool_sync(&call.name, &call.arguments) {
                        Ok(res) => res,
                        Err(e) => format!("Error: {e}"),
                    };
                    let result_content = trim_large_output(&result_raw);
                    
                    let tool_msg = crate::tool::Message {
                        role: "tool".to_string(),
                        content: Some(result_content),
                        name: Some(call.name.clone()),
                        tool_call_id: Some(call.id.clone()),
                        tool_calls: None,
                    };
                    let tool_text = serde_json::to_string(&tool_msg)
                        .map_err(|e| format!("Failed to serialize tool message: {e}"))?;
                    
                    let dag_val = call_host(RasRpcCommand::GetDag)?;
                    let dag: rad_models::Dag = serde_json::from_value(dag_val)
                        .map_err(|e| format!("Failed to parse Dag: {e}"))?;
                    let current_parent = dag.current_node_id.unwrap_or_default();
                    
                    let node_id_val = call_host(RasRpcCommand::CreateNode {
                        parent_id: current_parent,
                        node_type: "tool".to_string(),
                    })?;
                    let node_id = node_id_val.as_str().ok_or("Failed to get node id as string")?;
                    call_host(RasRpcCommand::SetNodeText {
                        node_id: node_id.to_string(),
                        text: tool_text,
                    })?;
                }
                
                // Continue stream
                let messages = crate::llm::load_messages_from_dag()?;
                crate::llm::trigger_llm_stream(messages)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn handle_done(mut state_guard: MutexGuard<'_, Option<OrchestratorState>>) -> Result<(), String> {
    let state = state_guard.as_mut().ok_or("State is None in handle_done")?;
    if state.is_reasoning {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
        });
        state.is_reasoning = false;
    }

    // Extract tool calls from state
    let mut tool_indices: Vec<usize> = state.tool_calls.keys().copied().collect();
    tool_indices.sort_unstable();

    let mut assistant_tool_calls = Vec::new();
    let mut pending_calls = Vec::new();

    for idx in tool_indices {
        if let Some(tool_call) = state.tool_calls.get(&idx) {
            assistant_tool_calls.push(ToolCall {
                id: tool_call.id.clone(),
                tool_type: "function".to_string(),
                function: ToolCallFunction {
                    name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                },
            });
            pending_calls.push(PendingToolCall {
                id: tool_call.id.clone(),
                name: tool_call.name.clone(),
                arguments: tool_call.arguments.clone(),
                result: None,
            });
        }
    }
    state.tool_calls.clear();

    let assistant_content = if state.assistant.is_empty() {
        None
    } else {
        Some(state.assistant.clone())
    };
    state.assistant.clear();
    state.reasoning_buffered.clear();

    drop(state_guard);

    let _ = call_host(RasRpcCommand::WriteStdout {
        text: "\n".to_string(),
    })?;

    let assistant_msg = Message {
        role: "assistant".to_string(),
        content: assistant_content,
        name: None,
        tool_call_id: None,
        tool_calls: if assistant_tool_calls.is_empty() {
            None
        } else {
            Some(assistant_tool_calls)
        },
    };
    let assistant_text = serde_json::to_string(&assistant_msg)
        .map_err(|e| format!("Failed to serialize assistant message: {e}"))?;
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag =
        serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let parent_id = dag.current_node_id.unwrap_or_default();

    let assistant_node_id_val = call_host(RasRpcCommand::CreateNode {
        parent_id,
        node_type: "assistant".to_string(),
    })?;
    let assistant_node_id = assistant_node_id_val
        .as_str()
        .ok_or("Failed to get node id as string")?;
    call_host(RasRpcCommand::SetNodeText {
        node_id: assistant_node_id.to_string(),
        text: assistant_text,
    })?;

    if pending_calls.is_empty() {
        let _ = call_host(RasRpcCommand::CompleteTask)?;
    } else {
        // Execute tool calls synchronously
        for mut tc in pending_calls {
            let result_raw = match execute_tool_sync(&tc.name, &tc.arguments) {
                Ok(res) => res,
                Err(e) => format!("Error: {e}"),
            };
            let result_content = trim_large_output(&result_raw);
            tc.result = Some(result_content);

            let tool_msg = Message {
                role: "tool".to_string(),
                content: tc.result,
                name: Some(tc.name.clone()),
                tool_call_id: Some(tc.id.clone()),
                tool_calls: None,
            };
            let tool_text = serde_json::to_string(&tool_msg)
                .map_err(|e| format!("Failed to serialize tool message: {e}"))?;

            let dag_val = call_host(RasRpcCommand::GetDag)?;
            let dag: Dag =
                serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
            let current_parent = dag.current_node_id.unwrap_or_default();

            let node_id_val = call_host(RasRpcCommand::CreateNode {
                parent_id: current_parent,
                node_type: "tool".to_string(),
                })?;
            let node_id = node_id_val
                .as_str()
                .ok_or("Failed to get node id as string")?;
            call_host(RasRpcCommand::SetNodeText {
                node_id: node_id.to_string(),
                text: tool_text,
            })?;
        }

        let messages = crate::llm::load_messages_from_dag()?;
        crate::llm::trigger_llm_stream(messages)?;
    }
    Ok(())
}

fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
    let wit_cmd = crate::radcomp::extension::types::RasRpcCommand::from(command);
    match crate::host_rpc(&wit_cmd) {
        Ok(json_str) => {
            if json_str.is_empty() || json_str == "null" {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(&json_str)
                    .map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}
