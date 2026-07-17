use super::STATE;
use crate::tool::{Message, ToolCall, ToolCallFunction, execute_tool_sync};
use crate::types::{Dag, OrchestratorState, PendingToolCall, RasRpcCommand};
use std::sync::MutexGuard;

pub(crate) fn trim_large_output(text: &str) -> String {
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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ExtUnifiedError {
    pub level: String,
    pub payload: serde_json::Value,
}

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
        let _ = call_host(RasRpcCommand::CompleteTask)?;
    } else {
        // Pillar 2: Take a snapshot of the workspace before running tools
        let _ = call_host(RasRpcCommand::TakeSnapshot {
            node_id: assistant_node_id.to_string(),
            target_paths: vec![std::path::PathBuf::from(".")],
        });

        for mut tc in pending_calls {
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

pub(crate) fn call_host(command: RasRpcCommand) -> Result<serde_json::Value, String> {
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
