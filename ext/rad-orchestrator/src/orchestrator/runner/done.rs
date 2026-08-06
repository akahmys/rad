/// Handles the `Done` streaming event: finalizes the assistant turn, records
/// it in the DAG, and synchronously executes any pending tool calls, split
/// out of `runner.rs` to stay under the 300-line file limit.
use super::inline_tool_calls::parse_inline_tool_calls;
use super::{ExtUnifiedError, call_host, trim_large_output};
use crate::tool::{Message, ToolCall, ToolCallFunction, execute_tool_sync};
use crate::types::{Dag, OrchestratorState, PendingToolCall, RasRpcCommand};
use std::sync::MutexGuard;

fn extract_tool_calls(state: &mut OrchestratorState) -> (Vec<ToolCall>, Vec<PendingToolCall>) {
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
    (assistant_tool_calls, pending_calls)
}

fn handle_l2_l3_error(tc_name: &str, e: &str, assistant_node_id: &str) -> String {
    if let Ok(ue) = serde_json::from_str::<ExtUnifiedError>(e) {
        match ue.level.as_str() {
            "L2" => {
                let msg = ue
                    .payload
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("L2 error");
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: format!(
                        "\n\x1b[1;31m[Rollback] L2 Error: {msg}. Restoring checkpoint...\x1b[0m\n"
                    ),
                });
                let _ = call_host(RasRpcCommand::CheckoutSnapshot {
                    node_id: assistant_node_id.to_string(),
                });
                format!("Error: Tool call \"{tc_name}\" was not executed: {msg}")
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

fn execute_pending_calls(
    assistant_node_id: &str,
    pending_calls: Vec<PendingToolCall>,
) -> Result<(), String> {
    crate::log_trace(
        "session",
        &format!("Found {} pending tool calls.", pending_calls.len()),
    );
    let _ = call_host(RasRpcCommand::TakeSnapshot {
        node_id: assistant_node_id.to_string(),
        target_paths: vec![std::path::PathBuf::from(".")],
    });

    let mut breaker_tripped = false;

    for mut tc in pending_calls {
        crate::log_trace(
            "session",
            &format!("Executing tool '{}' with args: {}", tc.name, tc.arguments),
        );
        let result_raw = match execute_tool_sync(&tc.name, &tc.arguments) {
            Ok(res) => res,
            Err(e) => handle_l2_l3_error(&tc.name, &e, assistant_node_id),
        };
        let result_content = trim_large_output(&result_raw);
        if !result_content.trim().is_empty() {
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: format!(
                    "\n\x1b[36m[Tool Output]\x1b[0m\n{}\n",
                    result_content.trim()
                ),
            });
        }
        if update_and_check_circuit_breaker(&tc.name, &result_content) {
            breaker_tripped = true;
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

        if breaker_tripped {
            // Stop executing any further pending calls from this same
            // turn — the model asked for several at once, but continuing
            // to run more of them after the breaker has already decided
            // this turn is stuck wouldn't tell us anything new.
            break;
        }
    }

    if breaker_tripped {
        let _ = call_host(RasRpcCommand::WriteStdout {
            text: "\n\x1b[1;31m[Circuit Breaker] Stopping: the same tool failed too many times in a row. Review the errors above.\x1b[0m\n".to_string(),
        });
        let _ = call_host(RasRpcCommand::CompleteTask)?;
        return Ok(());
    }

    let messages = crate::llm::load_messages_from_dag()?;
    crate::llm::trigger_llm_stream(&messages)?;
    Ok(())
}

/// Updates the consecutive-same-tool-failure streak in `OrchestratorState`
/// and reports whether it just crossed `max_consecutive_tool_failures`.
/// Failure is detected via the `Error:` prefix convention every tool
/// result in this codebase now guarantees (`mcp-tool-provider`'s
/// `execute_tool`, `handle_l2_l3_error` above) — there's no other
/// structural signal available once a tool result is flattened to a
/// plain string, since MCP servers report tool-level failure via
/// `isError` rather than a protocol-level error.
fn update_and_check_circuit_breaker(tool_name: &str, result_content: &str) -> bool {
    let is_failure = result_content.trim_start().starts_with("Error:");
    let Ok(mut guard) = crate::orchestrator::STATE.lock() else {
        return false;
    };
    let Some(state) = guard.as_mut() else {
        return false;
    };

    if is_failure {
        if state.last_tool_name.as_deref() == Some(tool_name) {
            state.consecutive_tool_failures += 1;
        } else {
            state.consecutive_tool_failures = 1;
        }
    } else {
        state.consecutive_tool_failures = 0;
    }
    state.last_tool_name = Some(tool_name.to_string());

    let threshold = state.max_consecutive_tool_failures.unwrap_or(u32::MAX);
    is_failure && state.consecutive_tool_failures >= threshold
}

pub(crate) fn handle_done(
    mut state_guard: MutexGuard<'_, Option<OrchestratorState>>,
) -> Result<(), String> {
    let state = state_guard.as_mut().ok_or("State is None in handle_done")?;

    // The turn came back without a context-window rejection, so any budget
    // reduction from a previous L3 retry has served its purpose. Restoring
    // full budget here keeps one transient over-estimate from permanently
    // starving the rest of the task of context (ARCHITECTURE.md §5.1.2).
    state.context_retries_used = 0;
    state.context_budget_scale_percent = 100;

    if state.is_reasoning {
        if crate::orchestrator::reasoning::thinking_enabled() {
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
            });
        }
        state.is_reasoning = false;
    }

    let (mut assistant_tool_calls, mut pending_calls) = extract_tool_calls(state);

    let assistant_content_str = state.assistant.clone();
    let assistant_content = if state.assistant.is_empty() {
        None
    } else {
        Some(state.assistant.clone())
    };
    state.assistant.clear();
    state.reasoning_buffered.clear();

    drop(state_guard);

    if assistant_tool_calls.is_empty() && !assistant_content_str.is_empty() {
        parse_inline_tool_calls(
            &assistant_content_str,
            &mut assistant_tool_calls,
            &mut pending_calls,
        );
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
        crate::log_trace("session", "No pending tool calls. Completing task.");
        let _ = call_host(RasRpcCommand::CompleteTask)?;
    } else {
        execute_pending_calls(assistant_node_id, pending_calls)?;
    }
    Ok(())
}
