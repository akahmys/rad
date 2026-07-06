use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::types::{RasRpcCommand, RasCoreEvent, Dag};
use crate::call_host;

#[derive(Serialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

pub struct OrchestratorState {
    pub assistant_buffer: String,
    pub stream_buffer: String,
}

pub static STATE: Mutex<Option<OrchestratorState>> = Mutex::new(None);

pub fn load_messages_from_dag() -> Result<Vec<Message>, String> {
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;

    let mut messages = Vec::new();
    let mut current_id = dag.current_node_id;

    while let Some(ref id) = current_id {
        if let Some(node) = dag.nodes.get(id) {
            messages.push(Message {
                role: node.node_type.clone(),
                content: node.text.clone(),
            });
            current_id = node.parent_ids.first().cloned();
        } else {
            break;
        }
    }

    messages.reverse();
    Ok(messages)
}

pub fn handle_event(event: RasCoreEvent) -> Result<(), String> {
    match event {
        RasCoreEvent::HumanInputReceived { text } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            let state = state_guard.get_or_insert_with(|| OrchestratorState {
                assistant_buffer: String::new(),
                stream_buffer: String::new(),
            });
            
            let dag_val = call_host(RasRpcCommand::GetDag)?;
            let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
            let parent_id = dag.current_node_id.unwrap_or_default();

            let user_node_id_val = call_host(RasRpcCommand::CreateNode {
                parent_id,
                node_type: "user".to_string(),
            })?;
            let user_node_id = user_node_id_val.as_str().ok_or("Failed to get node id as string")?;
            
            call_host(RasRpcCommand::SetNodeText {
                node_id: user_node_id.to_string(),
                text,
            })?;

            let messages = load_messages_from_dag()?;
            
            trigger_llm_stream(state, messages)?;
        }
        RasCoreEvent::HttpChunkReceived { chunk } => {
            let mut state_guard = STATE.lock().map_err(|e| format!("Mutex lock error: {e}"))?;
            if let Some(state) = state_guard.as_mut() {
                state.stream_buffer.push_str(&chunk);
                process_sse_buffer(state)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub fn trigger_llm_stream(_state: &OrchestratorState, messages: Vec<Message>) -> Result<(), String> {
    let tools = vec![
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "file_read".to_string(),
                description: Some("Read the entire contents of a file at the specified path.".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The path to the file to read."
                        }
                    },
                    "required": ["path"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "file_write".to_string(),
                description: Some("Write content to a file at the specified path.".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The target path to write the file."
                        },
                        "content": {
                            "type": "string",
                            "description": "The raw text content to write."
                        }
                    },
                    "required": ["path", "content"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "file_edit_patch".to_string(),
                description: Some("Apply a unified diff patch to modify a file at the specified path.".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The target file path to patch."
                        },
                        "diff": {
                            "type": "string",
                            "description": "The unified diff content to apply."
                        }
                    },
                    "required": ["path", "diff"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "spawn_bash_process".to_string(),
                description: Some("Spawn a command in a non-interactive bash shell.".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command line string to execute."
                        }
                    },
                    "required": ["command"]
                }),
            },
        },
    ];

    let req = ChatCompletionsRequest {
        model: "qwen".to_string(),
        messages,
        stream: true,
        tools: Some(tools),
    };
    let body = serde_json::to_string(&req).map_err(|e| format!("JSON serialize error: {e}"))?;
    
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let url = "http://127.0.0.1:8080/v1/chat/completions".to_string();
    
    call_host(RasRpcCommand::OpenHttpStream {
        url,
        headers,
        body,
    })?;
    
    Ok(())
}

pub fn process_sse_buffer(state: &mut OrchestratorState) -> Result<(), String> {
    while let Some(pos) = state.stream_buffer.find('\n') {
        let line = state.stream_buffer[..pos].trim().to_string();
        state.stream_buffer = state.stream_buffer[pos + 1..].to_string();
        
        if line.is_empty() {
            continue;
        }
        
        if line.starts_with("data:") {
            let data_str = line["data:".len()..].trim();
            if data_str == "[DONE]" {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: "\n".to_string(),
                })?;
                
                let dag_val = call_host(RasRpcCommand::GetDag)?;
                let dag: Dag = serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
                let parent_id = dag.current_node_id.unwrap_or_default();

                let assistant_node_id_val = call_host(RasRpcCommand::CreateNode {
                    parent_id,
                    node_type: "assistant".to_string(),
                })?;
                let assistant_node_id = assistant_node_id_val.as_str().ok_or("Failed to get node id as string")?;
                
                call_host(RasRpcCommand::SetNodeText {
                    node_id: assistant_node_id.to_string(),
                    text: state.assistant_buffer.clone(),
                })?;

                state.assistant_buffer.clear();
                let _ = call_host(RasRpcCommand::CompleteTask)?;
                break;
            }
            
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                if let Some(content) = val.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
                    let _ = call_host(RasRpcCommand::WriteStdout {
                        text: content.to_string(),
                    })?;
                    state.assistant_buffer.push_str(content);
                }
            }
        }
    }
    Ok(())
}
