use serde::{Deserialize, Serialize};
use crate::types::RasRpcCommand;
use crate::call_host;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Clone)]
pub struct StreamOptions {
    pub include_usage: bool,
}

#[derive(Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

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

#[derive(Default, Clone)]
pub struct ToolCallBuffer {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ToolExecutionResult {
    Sync(String),
    Async(i32),
    McpAsync,
}

pub fn execute_tool(tool_call_id: &str, name: &str, arguments: &str) -> Result<ToolExecutionResult, String> {
    match name {
        "file_read" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse file_read args: {e}"))?;
            let val = call_host(RasRpcCommand::FileRead { path: args.path })?;
            
            let result_str = if let Some(bytes_val) = val.as_array() {
                let bytes: Vec<u8> = bytes_val.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect();
                String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in file: {e}"))?
            } else if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                val.to_string()
            };
            Ok(ToolExecutionResult::Sync(result_str))
        }
        "file_write" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                content: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse file_write args: {e}"))?;
            let _ = call_host(RasRpcCommand::FileWrite {
                path: args.path,
                data: args.content.clone().into_bytes(),
            })?;
            Ok(ToolExecutionResult::Sync("File written successfully.".to_string()))
        }
        "file_edit_patch" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                diff: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse file_edit_patch args: {e}"))?;
            let _ = call_host(RasRpcCommand::FileEditPatch {
                path: args.path,
                diff: args.diff,
            })?;
            Ok(ToolExecutionResult::Sync("Patch applied successfully.".to_string()))
        }
        "spawn_bash_process" => {
            #[derive(Deserialize)]
            struct Args {
                command: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse spawn_bash_process args: {e}"))?;
            let val = call_host(RasRpcCommand::SpawnBashProcess {
                command: args.command,
            })?;
            let pgid = val.as_i64().ok_or_else(|| format!("Expected process PGID integer, got: {val:?}"))?;
            Ok(ToolExecutionResult::Async(pgid as i32))
        }
        other => {
            let mut provider = None;
            if let Ok(state_guard) = crate::orchestrator::STATE.lock() {
                if let Some(state) = state_guard.as_ref() {
                    if let Some(p) = state.mcp_tool_providers.get(other) {
                        provider = Some(p.clone());
                    }
                }
            }

            if let Some(server_name) = provider {
                let args_json: serde_json::Value = serde_json::from_str(arguments)
                    .map_err(|e| format!("Failed to parse MCP tool args: {e}"))?;
                let req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": format!("mcp_call:{}", tool_call_id),
                    "method": "tools/call",
                    "params": {
                        "name": other,
                        "arguments": args_json
                    }
                });
                let req_str = serde_json::to_string(&req)
                    .map_err(|e| format!("Failed to serialize MCP request: {e}"))?;
                let _ = call_host(RasRpcCommand::SendMcpRequest {
                    name: server_name,
                    message: req_str,
                })?;
                Ok(ToolExecutionResult::McpAsync)
            } else {
                Err(format!("Unknown tool call: {other}"))
            }
        }
    }
}

pub fn get_tool_definitions() -> Vec<Tool> {
    vec![
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
    ]
}
