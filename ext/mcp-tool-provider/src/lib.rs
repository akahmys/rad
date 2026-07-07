#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-tool-provider",
});

use rad_models::RasRpcCommand as CoreRpcCommand;
use self::radcomp::extension::types as wit;

mod conv;

struct ToolProviderImpl;

impl Guest for ToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        let tools = get_tool_definitions();
        serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"))
    }

    fn execute_tool(name: String, arguments: String) -> Result<String, String> {
        match name.as_str() {
            "file_read" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse file_read args: {e}"))?;
                let val = call_host(CoreRpcCommand::FileRead { path: args.path })?;
                
                let result_str = if let Some(bytes_val) = val.as_array() {
                    let bytes: Vec<u8> = bytes_val.iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect();
                    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in file: {e}"))?
                } else if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };
                Ok(result_str)
            }
            "file_write" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                    content: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse file_write args: {e}"))?;
                let _ = call_host(CoreRpcCommand::FileWrite {
                    path: args.path,
                    data: args.content.into_bytes(),
                })?;
                Ok("File written successfully.".to_string())
            }
            "file_edit_patch" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                    diff: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse file_edit_patch args: {e}"))?;
                let _ = call_host(CoreRpcCommand::FileEditPatch {
                    path: args.path,
                    diff: args.diff,
                })?;
                Ok("Patch applied successfully.".to_string())
            }
            "spawn_bash_process" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    command: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse spawn_bash_process args: {e}"))?;
                let val = call_host(CoreRpcCommand::SpawnBashProcess {
                    command: args.command,
                })?;
                let pgid = val.as_i64().ok_or_else(|| format!("Expected process PGID integer, got: {val:?}"))?;
                Ok(format!("Started background process group PGID: {pgid}"))
            }
            other => {
                // If it is an MCP tool, we delegate it via SendMcpRequest
                // Note: The orchestrator will handle the async McpResponse event
                let args_json: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse MCP tool args: {e}"))?;
                let req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": format!("mcp_call:{}", other),
                    "method": "tools/call",
                    "params": {
                        "name": other,
                        "arguments": args_json
                    }
                });
                let req_str = serde_json::to_string(&req)
                    .map_err(|e| format!("Failed to serialize MCP request: {e}"))?;

                // Note: MCP server name mapping would need to be resolved.
                // For now we assume a default echo-mcp or query mcp names.
                let _ = call_host(CoreRpcCommand::SendMcpRequest {
                    name: "echo-mcp".to_string(), // Default mapping or fallback
                    message: req_str,
                })?;
                Ok("mcp_async".to_string())
            }
        }
    }
}

export!(ToolProviderImpl);

fn call_host(command: CoreRpcCommand) -> Result<serde_json::Value, String> {
    let wit_cmd = wit::RasRpcCommand::from(command);
    match host_rpc(&wit_cmd) {
        Ok(json_str) => {
            if json_str.is_empty() || json_str == "null" {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(&json_str).map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

fn get_tool_definitions() -> Vec<Tool> {
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



