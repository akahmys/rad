#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-tool-provider",
});

use self::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;
use std::collections::HashMap;

mod conv;

struct ToolProviderImpl;

mod client;
mod default_tools;

use client::{init_mcp_servers, mcp_request, MCP_SERVERS, MCP_TOOL_MAPPING};
use default_tools::{get_default_tools, Tool, FunctionDefinition};

impl Guest for ToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        let mut tools = get_default_tools();

        if let Ok(()) = init_mcp_servers() {
            let mut mapping = HashMap::new();
            let servers_list: Vec<String> = {
                if let Ok(guard) = MCP_SERVERS.lock() {
                    guard
                        .as_ref()
                        .map(|m| m.keys().cloned().collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            };

            for server_name in servers_list {
                let req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "list_tools",
                    "method": "tools/list",
                    "params": {}
                });
                if let Ok(res) = mcp_request(&server_name, &req) {
                    if let Some(result) = res.get("result") {
                        if let Some(mcp_tools) = result.get("tools").and_then(|t| t.as_array()) {
                            for t in mcp_tools {
                                if let Some(name) = t.get("name").and_then(|n| n.as_str()) {
                                    mapping.insert(name.to_string(), server_name.clone());
                                    let description = t
                                        .get("description")
                                        .and_then(|d| d.as_str())
                                        .map(ToString::to_string);
                                    let parameters = t.get("inputSchema").cloned().unwrap_or(
                                        serde_json::json!({
                                            "type": "object",
                                            "properties": {}
                                        }),
                                    );
                                    tools.push(Tool {
                                        tool_type: "function".to_string(),
                                        function: FunctionDefinition {
                                            name: name.to_string(),
                                            description,
                                            parameters,
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
            }
            if let Ok(mut map_guard) = MCP_TOOL_MAPPING.lock() {
                *map_guard = Some(mapping);
            }
        }

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
            other => {
                let mapping = {
                    let mut map_guard = MCP_TOOL_MAPPING.lock().map_err(|e| e.to_string())?;
                    if map_guard.is_none() {
                        // Populate mapping by running get_tools once
                        drop(map_guard);
                        let _ = Self::get_tools()?;
                        map_guard = MCP_TOOL_MAPPING.lock().map_err(|e| e.to_string())?;
                    }
                    map_guard.clone().ok_or("MCP tool mapping unavailable")?
                };

                let server_name = mapping
                    .get(other)
                    .ok_or_else(|| format!("Unknown tool provider for tool '{other}'"))?;

                let args_json: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| format!("Failed to parse MCP tool args: {e}"))?;
                let req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "mcp_call",
                    "method": "tools/call",
                    "params": {
                        "name": other,
                        "arguments": args_json
                    }
                });

                let res = mcp_request(server_name, &req)?;

                // Parse tool call result
                let mut result_text = String::new();
                if let Some(err) = res
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                {
                    result_text = format!("Error from MCP server: {err}");
                } else if let Some(result) = res.get("result") {
                    if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
                        let mut texts = Vec::new();
                        for item in content {
                            if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                                    texts.push(text.to_string());
                                }
                            }
                        }
                        result_text = texts.join("\n");
                    }
                }
                if result_text.is_empty() {
                    result_text = "No content returned from MCP server.".to_string();
                }

                let escaped_result = result_text.replace('\'', "'\\''");
                open_process(&format!("echo -n '{escaped_result}'"))
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
                serde_json::from_str(&json_str)
                    .map_err(|e| format!("JSON parse error from host: {e}"))
            }
        }
        Err(err_msg) => Err(err_msg),
    }
}


