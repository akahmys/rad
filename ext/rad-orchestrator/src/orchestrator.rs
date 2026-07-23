pub(crate) mod runner;

use crate::types::{Dag, OrchestratorState, RasCoreEvent, RasRpcCommand};
use runner::{call_host, handle_done, trim_large_output};
use std::collections::HashMap;
use std::sync::Mutex;

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

fn handle_human_input(text: String) -> Result<(), String> {
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
        text: text.clone(),
    })?;

    let _ = call_host(RasRpcCommand::WriteStdout {
        text: "\x1b[36m[Thinking...]\x1b[0m\n".to_string(),
    });

    crate::log_trace("session", &format!("Received human input: {text}"));
    crate::log_trace("session", "Loading messages from DAG...");
    let messages = crate::llm::load_messages_from_dag()?;
    crate::log_trace("session", "Triggering LLM stream...");
    crate::llm::trigger_llm_stream(messages)
}

#[derive(serde::Deserialize)]
struct ToolCallChunkEvent {
    index: u32,
    id: Option<String>,
    name: Option<String>,
    #[serde(alias = "arguments-chunk")]
    arguments_chunk: String,
}

#[derive(serde::Deserialize)]
struct CompletionUsageEvent {
    #[serde(alias = "prompt-tokens")]
    prompt_tokens: u32,
    #[serde(alias = "completion-tokens")]
    completion_tokens: u32,
}

#[derive(serde::Deserialize)]
struct RawEvent {
    #[serde(rename = "type")]
    event_type: Option<String>,
    payload: Option<String>,

    #[serde(rename = "ContentChunk")]
    content_chunk: Option<String>,
    #[serde(rename = "ReasoningChunk")]
    reasoning_chunk: Option<String>,
    #[serde(rename = "ToolCallChunk")]
    tool_call_chunk: Option<ToolCallChunkEvent>,
    #[serde(rename = "CompletionComplete")]
    completion_complete: Option<CompletionUsageEvent>,
    #[serde(rename = "Error")]
    error: Option<String>,
}

fn handle_content_token(state: &mut OrchestratorState, content: &str) {
    if state.is_reasoning && !content.contains("<thought>") && !content.contains("</thought>") {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
        });
        state.is_reasoning = false;
    }

    let mut text = content.to_string();
    if text.contains("<thought>") {
        text = handle_thought_start_tag(state, &text);
    }

    if state.is_reasoning {
        handle_reasoning_text(state, &text);
    } else {
        let _ = call_host(RasRpcCommand::WriteStdout { text: text.clone() });
        state.assistant.push_str(&text);
    }
}

fn handle_thought_start_tag(state: &mut OrchestratorState, text: &str) -> String {
    if let Some(pos) = text.find("<thought>") {
        let before = &text[..pos];
        if !before.is_empty() {
            if state.is_reasoning {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
                });
            }
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: before.to_string(),
            });
            state.assistant.push_str(before);
        }
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string(),
        });
        state.is_reasoning = true;
        return text[pos + "<thought>".len()..].to_string();
    }
    text.to_string()
}

fn handle_reasoning_text(state: &mut OrchestratorState, text: &str) {
    if text.contains("</thought>") {
        if let Some(pos) = text.find("</thought>") {
            let thought_content = &text[..pos];
            if !thought_content.is_empty() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: format!("\x1b[2m{}\x1b[0m", thought_content),
                });
                state.reasoning_buffered.push_str(thought_content);
            }
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
            });
            state.is_reasoning = false;
            let after = &text[pos + "</thought>".len()..];
            if !after.is_empty() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: after.to_string(),
                });
                state.assistant.push_str(after);
            }
        }
    } else {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: format!("\x1b[2m{}\x1b[0m", text),
        });
        state.reasoning_buffered.push_str(text);
    }
}

pub fn handle_event(event: RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::HumanInputReceived { text } => handle_human_input(text),
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
                        let _ = call_host(RasRpcCommand::WriteStdout {
                            text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string(),
                        });
                        state.is_reasoning = true;
                    }
                    let _ = call_host(RasRpcCommand::WriteStdout {
                        text: format!("\x1b[2m{}\x1b[0m", reasoning),
                    });
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

                if let Some(ref usage) = raw.completion_complete {
                    if usage.prompt_tokens > 0 || usage.completion_tokens > 0 {
                        let _ = call_host(RasRpcCommand::ReportTokenUsage {
                            prompt_tokens: usage.prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                        });
                    }
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
        RasCoreEvent::Rehydrate { active_calls } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = OrchestratorState {
                assistant: String::new(),
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
                    let result_raw =
                        match crate::tool::execute_tool_sync(&call.name, &call.arguments) {
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

                // Continue stream
                let messages = crate::llm::load_messages_from_dag()?;
                crate::llm::trigger_llm_stream(messages)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
