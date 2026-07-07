use std::collections::HashMap;
use crate::types::{RasRpcCommand, Dag, PendingToolCall, OrchestratorState};
use crate::call_host;
use crate::tool::{Message, ToolCall, ToolCallFunction, execute_tool, ToolExecutionResult};
use crate::orchestrator::STATE;

fn trim_large_output(text: &str) -> String {
    let max_chars = STATE.lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|s| s.max_tool_output_chars))
        .unwrap_or(2000);

    if text.len() <= max_chars {
        return text.to_string();
    }

    let head_len = max_chars / 4;
    let tail_len = max_chars - head_len;

    // Safely collect characters to avoid slicing mid-multibyte characters
    let head: String = text.chars().take(head_len).collect();
    let tail: String = text.chars().rev().take(tail_len).collect::<String>().chars().rev().collect();

    format!(
        "{head}\n\n... [TRUNCATED {} CHARACTERS FOR TOKEN SAVINGS] ...\n\n{tail}",
        text.len() - max_chars
    )
}

pub fn process_completed_tool_calls(pending: Vec<PendingToolCall>) -> Result<(), String> {
    for tc in pending {
        let raw_result = tc.result.unwrap_or_else(|| "No execution result.".to_string());
        let result_content = trim_large_output(&raw_result);
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
    }
    crate::llm::trigger_llm_stream(messages)
}

pub fn execute_and_collect_tools(
    pending_calls: Vec<PendingToolCall>,
) -> Result<(), String> {
    let mut executed_calls = Vec::new();
    let mut all_sync_done = true;

    for mut tc in pending_calls {
        match execute_tool(&tc.id, &tc.name, &tc.arguments) {
            Ok(ToolExecutionResult::Sync(res)) => {
                tc.result = Some(res);
                executed_calls.push(tc);
            }
            Ok(ToolExecutionResult::Async(pgid)) => {
                tc.pgid = Some(pgid);
                executed_calls.push(tc);
                all_sync_done = false;
            }
            Ok(ToolExecutionResult::McpAsync) => {
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

pub fn extract_tool_calls(
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
