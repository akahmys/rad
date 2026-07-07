use std::collections::HashMap;
use crate::types::{RasRpcCommand, Dag};
use crate::call_host;
use crate::tool::{Message, ChatCompletionsRequest, get_tool_definitions, StreamOptions};

fn load_local_agent_rules() -> String {
    let paths = [".agents/AGENTS.md", "AGENTS.md"];
    for p in &paths {
        let path_buf = std::path::PathBuf::from(p);
        if let Ok(val) = call_host(RasRpcCommand::FileRead { path: path_buf }) {
            if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(val) {
                if let Ok(content) = String::from_utf8(bytes) {
                    return format!("\n\n### Local Project Rules ({p}):\n{content}");
                }
            }
        }
    }
    String::new()
}

fn get_system_prompt() -> String {
    let mut prompt = "You are an expert coding assistant operating inside rad, a coding agent harness. You help users by reading files, executing commands, editing code, and writing new files.".to_string();
    let rules = load_local_agent_rules();
    if !rules.is_empty() {
        prompt.push_str(&rules);
    }
    prompt
}

pub fn load_messages_from_dag() -> Result<Vec<Message>, String> {
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let mut messages = Vec::new();
    let mut current_id = dag.current_node_id;

    while let Some(ref id) = current_id {
        if let Some(node) = dag.nodes.get(id) {
            let is_valid_role = match node.node_type.as_str() {
                "user" | "assistant" | "tool" | "system" => true,
                _ => false,
            };

            if is_valid_role {
                let msg = if let Ok(mut parsed_msg) = serde_json::from_str::<Message>(&node.text) {
                    parsed_msg.role = node.node_type.clone();
                    parsed_msg
                } else {
                    Message {
                        role: node.node_type.clone(),
                        content: if node.text.is_empty() { None } else { Some(node.text.clone()) },
                        name: None,
                        tool_call_id: None,
                        tool_calls: None,
                    }
                };

                let is_empty_msg = msg.content.as_ref().map_or(true, |c| c.is_empty())
                    && msg.tool_calls.as_ref().map_or(true, |t| t.is_empty());

                if !is_empty_msg {
                    messages.push(msg);
                }
            }
            current_id = node.parent_ids.first().cloned();
        } else {
            break;
        }
    }

    messages.reverse();

    let max_history = crate::orchestrator::STATE.lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|s| s.max_history_messages))
        .unwrap_or(6);

    let messages_to_send = if messages.len() > max_history && !messages.is_empty() {
        let first_goal = messages[0].clone();
        let remaining_len = messages.len() - 1;
        let limit = max_history - 1;
        let start_idx = if remaining_len > limit {
            messages.len() - limit
        } else {
            1
        };
        let mut trimmed = vec![first_goal];
        trimmed.extend(messages[start_idx..].to_vec());
        trimmed
    } else {
        messages
    };

    let mut all_messages = vec![Message {
        role: "system".to_string(),
        content: Some(get_system_prompt()),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    all_messages.extend(messages_to_send);
    Ok(all_messages)
}

pub fn trigger_llm_stream(messages: Vec<Message>) -> Result<(), String> {
    let mut tools = get_tool_definitions();

    if let Ok(state_guard) = crate::orchestrator::STATE.lock() {
        if let Some(state) = state_guard.as_ref() {
            tools.extend(state.mcp_tools.clone());
        }
    }

    let req = ChatCompletionsRequest {
        model: "qwen".to_string(),
        messages,
        stream: true,
        stream_options: Some(StreamOptions { include_usage: true }),
        tools: Some(tools),
    };
    let body = serde_json::to_string(&req).map_err(|e| format!("JSON serialize error: {e}"))?;
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    call_host(RasRpcCommand::OpenHttpStream {
        url: "http://127.0.0.1:8080/v1/chat/completions".to_string(),
        headers,
        body,
    })?;
    Ok(())
}
