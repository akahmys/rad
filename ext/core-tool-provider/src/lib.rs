#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-tool-provider",
});

use self::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;

mod conv;

struct ToolProviderImpl;

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

fn get_default_tools() -> Vec<Tool> {
    vec![
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "read".to_string(),
                description: Some(
                    "Read the entire contents of a file at the specified path.".to_string(),
                ),
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
                name: "write".to_string(),
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
                name: "edit".to_string(),
                description: Some(
                    "Apply a unified diff patch to modify a file at the specified path."
                        .to_string(),
                ),
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
                name: "bash".to_string(),
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
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "search_web".to_string(),
                description: Some("Search the web for a given query.".to_string()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query query string."
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "fetch_url".to_string(),
                description: Some(
                    "Fetch the cleaned markdown text content of a web page at a URL.".to_string(),
                ),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL of the web page to fetch."
                        }
                    },
                    "required": ["url"]
                }),
            },
        },
    ]
}

impl Guest for ToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        let tools = get_default_tools();
        serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"))
    }

    fn execute_tool(name: String, arguments: String) -> Result<wit::ExecutionHandle, String> {
        match name.as_str() {
            "read" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse read args: {e}"))?;
                let path_str = args.path.to_string_lossy();
                open_process(&format!("cat '{path_str}'"))
            }
            "write" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                    content: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse write args: {e}"))?;
                let path_str = args.path.to_string_lossy();
                let escaped_content = args.content.replace('\'', "'\\''");
                open_process(&format!("echo -n '{escaped_content}' > '{path_str}'"))
            }
            "edit" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: std::path::PathBuf,
                    diff: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse edit args: {e}"))?;
                let _ = call_host(CoreRpcCommand::FileEditPatch {
                    path: args.path,
                    diff: args.diff,
                })?;
                open_process("echo 'Patch applied successfully.'")
            }
            "bash" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    command: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse bash args: {e}"))?;
                open_process(&args.command)
            }
            "search_web" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    query: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse search_web args: {e}"))?;

                let res_val = call_host(CoreRpcCommand::CallExtension {
                    extension_id: "web-access".to_string(),
                    method: "search".to_string(),
                    arguments: args.query,
                })?;

                let out_str = res_val.as_str().unwrap_or("").to_string();
                let escaped = out_str.replace('\'', "'\\''");
                open_process(&format!("echo -n '{escaped}'"))
            }
            "fetch_url" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    url: String,
                }
                let args: Args = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse fetch_url args: {e}"))?;

                let res_val = call_host(CoreRpcCommand::CallExtension {
                    extension_id: "web-access".to_string(),
                    method: "fetch".to_string(),
                    arguments: args.url,
                })?;

                let out_str = res_val.as_str().unwrap_or("").to_string();
                let escaped = out_str.replace('\'', "'\\''");
                open_process(&format!("echo -n '{escaped}'"))
            }
            other => Err(format!("Unknown core tool '{other}'")),
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
                serde_json::from_str(&json_str)
                    .map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}
