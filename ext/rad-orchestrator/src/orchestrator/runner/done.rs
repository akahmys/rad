/// Handles the `Done` streaming event: finalizes the assistant turn, records
/// it in the DAG, and synchronously executes any pending tool calls, split
/// out of `runner.rs` to stay under the 300-line file limit.
use super::inline_tool_calls::parse_inline_tool_calls;
use super::{ExtUnifiedError, call_host, trim_large_output};
use crate::tool::{Message, ToolCall, ToolCallFunction, execute_tool_sync};
use crate::types::{Dag, OrchestratorState, PendingToolCall, RasRpcCommand};
use std::sync::MutexGuard;

pub(crate) fn handle_done(
    mut state_guard: MutexGuard<'_, Option<OrchestratorState>>,
) -> Result<(), String> {
    let state = state_guard.as_mut().ok_or("State is None in handle_done")?;
    if state.is_reasoning {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
        });
        state.is_reasoning = false;
    }

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

    let assistant_content_str = state.assistant.clone();
    let assistant_content = if state.assistant.is_empty() {
        None
    } else {
        Some(state.assistant.clone())
    };
    state.assistant.clear();
    state.reasoning_buffered.clear();

    drop(state_guard);

    // Fallback parser for plain text tool calls like `<|tool_call>call:rad:execute_command{...}<tool_call|>`
    if assistant_tool_calls.is_empty() && !assistant_content_str.is_empty() {
        parse_inline_tool_calls(&assistant_content_str, &mut assistant_tool_calls, &mut pending_calls);
    }

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
        parent_id: parent_id.clone(),
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
        crate::log_trace("session", "No pending tool calls. Completing task.");
        let _ = call_host(RasRpcCommand::CompleteTask)?;
    } else {
        crate::log_trace("session", &format!("Found {} pending tool calls.", pending_calls.len()));
        // Pillar 2: Take a snapshot of the workspace before running tools
        let _ = call_host(RasRpcCommand::TakeSnapshot {
            node_id: assistant_node_id.to_string(),
            target_paths: vec![std::path::PathBuf::from(".")],
        });

        for mut tc in pending_calls {
            crate::log_trace("session", &format!("Executing tool '{}' with args: {}", tc.name, tc.arguments));
            let result_raw = match execute_tool_sync(&tc.name, &tc.arguments) {
                Ok(res) => res,
                Err(e) => {
                    if let Ok(ue) = serde_json::from_str::<ExtUnifiedError>(&e) {
                        match ue.level.as_str() {
                            "L2" => {
                                let msg = ue
                                    .payload
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("L2 error");
                                // Pillar 3: Semantic User Notification
                                let _ = call_host(RasRpcCommand::WriteStdout {
                                    text: format!(
                                        "\n\x1b[1;31m[Rollback] L2 Error: {msg}. Restoring checkpoint...\x1b[0m\n"
                                    ),
                                });
                                // Pillar 2: Roll back files synchronously
                                let _ = call_host(RasRpcCommand::CheckoutSnapshot {
                                    node_id: assistant_node_id.to_string(),
                                });
                                // Pillar 3: Raw LLM error feedback
                                format!("Tool call \"{}\" was not executed: {}", tc.name, msg)
                            }
                            "L3" => {
                                let msg = ue
                                    .payload
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("L3 error");
                                let _ = call_host(RasRpcCommand::WriteStdout {
                                    text: format!(
                                        "\n\x1b[1;31m[Reset] L3 Error: {msg}. Resetting session context...\x1b[0m\n"
                                    ),
                                });
                                format!("Error: {msg}")
                            }
                            _ => {
                                let msg = ue
                                    .payload
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("L1 error");
                                format!("Error: {msg}")
                            }
                        }
                    } else {
                        format!("Error: {e}")
                    }
                }
            };
            let result_content = trim_large_output(&result_raw);
            if !result_content.trim().is_empty() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: format!("\n\x1b[36m[Tool Output]\x1b[0m\n{}\n", result_content.trim()),
                });
            }
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
