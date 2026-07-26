pub(crate) mod reasoning;
pub(crate) mod runner;

use crate::types::{Dag, OrchestratorState, RasCoreEvent, RasRpcCommand};
use reasoning::{RawEvent, debug_enabled, handle_content_token};
use runner::{call_host, handle_done, trim_large_output};
use std::collections::HashMap;
use std::sync::Mutex;

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

fn handle_human_input(text: &str) -> Result<(), String> {
    {
        let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
        *state_guard = Some(OrchestratorState {
            assistant: String::new(),
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
        text: text.to_string(),
    })?;

    if debug_enabled() {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\x1b[36m[Thinking...]\x1b[0m\n".to_string(),
        });
    }

    crate::log_trace("session", &format!("Received human input: {text}"));
    crate::log_trace("session", "Loading messages from DAG...");
    let messages = crate::llm::load_messages_from_dag()?;
    crate::log_trace("session", "Triggering LLM stream...");
    crate::llm::trigger_llm_stream(&messages)
}

fn handle_rehydrate(active_calls: Vec<rad_models::PendingToolCallInfo>) -> Result<(), String> {
    let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
    let state = OrchestratorState {
        assistant: String::new(),
        is_reasoning: false,
        reasoning_buffered: String::new(),
        tool_calls: HashMap::new(),
        max_history_messages: Some(50),
        max_tool_output_chars: Some(2000),
        is_rehydrated: true,
    };
    *state_guard = Some(state);
    drop(state_guard);

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
            let node_id = node_id_val
                .as_str()
                .ok_or("Failed to get node id as string")?;
            call_host(RasRpcCommand::SetNodeText {
                node_id: node_id.to_string(),
                text: tool_text,
            })?;
        }

        let messages = crate::llm::load_messages_from_dag()?;
        crate::llm::trigger_llm_stream(&messages)?;
    }
    Ok(())
}

pub fn handle_event(event: RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::HumanInputReceived { text } => handle_human_input(&text),
        RasCoreEvent::LlmConnectorEvent { event: event_json } => {
            let raw: RawEvent = serde_json::from_str(&event_json).map_err(|e| {
                format!("Failed to parse LlmConnectorEvent JSON: {e} (raw={event_json})")
            })?;

            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let mut done = false;

            if let Some(state) = state_guard.as_mut() {
                if let Some(ref et) = raw.event_type {
                    if et == "done" {
                        done = true;
                    } else if et == "error" {
                        let payload = raw.payload.as_deref().unwrap_or("unknown error");
                        let error_text =
                            format!("\n\x1b[1;31mLLM Stream Error: {payload}\x1b[0m\n");
                        let _ = call_host(RasRpcCommand::WriteStdout { text: error_text })?;
                        let _ = call_host(RasRpcCommand::CompleteTask)?;
                        return Ok(());
                    }
                }

                if let Some(ref chunk) = raw.content_chunk {
                    handle_content_token(state, chunk);
                }

                if let Some(ref reasoning) = raw.reasoning_chunk {
                    if !state.is_reasoning {
                        if debug_enabled() {
                            let _ = call_host(RasRpcCommand::WriteStdout {
                                text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string(),
                            });
                        }
                        state.is_reasoning = true;
                    }
                    if debug_enabled() {
                        let _ = call_host(RasRpcCommand::WriteStdout {
                            text: format!("\x1b[2m{reasoning}\x1b[0m"),
                        });
                    }
                    state.reasoning_buffered.push_str(reasoning);
                }

                if let Some(ref tc) = raw.tool_call_chunk {
                    let entry = state.tool_calls.entry(tc.index as usize).or_default();
                    if let Some(ref id) = tc.id {
                        entry.id.push_str(id);
                    }
                    if let Some(ref name) = tc.name {
                        entry.name.push_str(name);
                    }
                    entry.arguments.push_str(&tc.arguments_chunk);
                }

                if let Some(usage) = raw.completion_complete.filter(|u| u.prompt_tokens > 0 || u.completion_tokens > 0) {
                    let _ = call_host(RasRpcCommand::ReportTokenUsage {
                        prompt_tokens: usage.prompt_tokens,
                        completion_tokens: usage.completion_tokens,
                    });
                }

                if let Some(ref message) = raw.error {
                    let error_text = format!("\n\x1b[1;31mLLM Stream Error: {message}\x1b[0m\n");
                    let _ = call_host(RasRpcCommand::WriteStdout { text: error_text })?;
                    let _ = call_host(RasRpcCommand::CompleteTask)?;
                    return Ok(());
                }
            }

            if done {
                handle_done(state_guard)?;
            }
            Ok(())
        }
        RasCoreEvent::Rehydrate { active_calls } => handle_rehydrate(active_calls),
        _ => Ok(()),
    }
}
