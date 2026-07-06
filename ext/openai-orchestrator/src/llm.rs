use std::collections::HashMap;
use crate::types::{RasRpcCommand, Dag};
use crate::call_host;
use crate::tool::{Message, ChatCompletionsRequest, get_tool_definitions, StreamOptions};

fn get_system_prompt() -> String {
    "You are Antigravity, a powerful agentic AI coding assistant running inside the Rad (Rust Agent Dispatcher) environment.\n\
     You can read/write files and execute bash commands to achieve the user's goals.\n\
     Be concise and focus on implementing features directly and incrementally.".to_string()
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
    let mut all_messages = vec![Message {
        role: "system".to_string(),
        content: Some(get_system_prompt()),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    all_messages.extend(messages);
    Ok(all_messages)
}

pub fn trigger_llm_stream(messages: Vec<Message>) -> Result<(), String> {
    let req = ChatCompletionsRequest {
        model: "qwen".to_string(),
        messages,
        stream: true,
        stream_options: Some(StreamOptions { include_usage: true }),
        tools: Some(get_tool_definitions()),
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
