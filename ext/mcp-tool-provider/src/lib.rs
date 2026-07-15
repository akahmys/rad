#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-tool-provider",
});

use self::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;
use std::collections::HashMap;
use std::sync::Mutex;

mod conv;

struct ToolProviderImpl;

#[derive(serde::Deserialize)]
struct ExtensionConfigInfo {
    name: String,
    config: Option<McpProviderConfig>,
}

#[derive(serde::Deserialize)]
struct McpProviderConfig {
    mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

#[derive(serde::Deserialize, Clone)]
struct McpServerConfig {
    command: String,
    args: Vec<String>,
}

#[derive(serde::Deserialize)]
struct RadJsonConfig {
    extensions: Option<Vec<ExtensionConfigInfo>>,
}

struct ActiveMcpServer {
    stdin: wit::StreamHandle,
    stdout: wit::StreamHandle,
}

static MCP_SERVERS: Mutex<Option<HashMap<String, ActiveMcpServer>>> = Mutex::new(None);
static MCP_TOOL_MAPPING: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn load_mcp_config() -> Result<Option<McpProviderConfig>, String> {
    let paths = ["rad.json", ".rad/rad.json"];
    let mut content = None;
    for p in &paths {
        if let Ok(c) = std::fs::read_to_string(p) {
            content = Some(c);
            break;
        }
    }
    let Some(json_str) = content else {
        return Ok(None);
    };

    let cfg: RadJsonConfig = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse rad.json: {e}"))?;

    let Some(extensions) = cfg.extensions else {
        return Ok(None);
    };

    // check both potential names for config compatibility
    for ext in extensions {
        if ext.name == "mcp-tool-provider" || ext.name == "openai-orchestrator" {
            if ext.config.is_some() {
                return Ok(ext.config);
            }
        }
    }

    Ok(None)
}

fn init_mcp_servers() -> Result<(), String> {
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    if servers_guard.is_some() {
        return Ok(());
    }

    let mut active = HashMap::new();
    if let Some(config) = load_mcp_config()? {
        if let Some(servers) = config.mcp_servers {
            for (name, cfg) in servers {
                // Construct command line args
                let mut cmd_parts = vec![cfg.command.clone()];
                cmd_parts.extend(cfg.args.clone());
                let command_line = cmd_parts
                    .iter()
                    .map(|arg| format!("'{arg}'"))
                    .collect::<Vec<_>>()
                    .join(" ");

                let exec = open_process(&command_line)?;
                let stdin = exec.get_stdin();
                let stdout = exec.get_stdout();
                active.insert(name, ActiveMcpServer { stdin, stdout });
            }
        }
    }

    *servers_guard = Some(active);
    Ok(())
}

fn read_line(stdout: &wit::StreamHandle) -> Result<String, String> {
    let mut buffer = Vec::new();
    let start = std::time::Instant::now();
    loop {
        let chunk = stdout.read(1024)?;
        if chunk.is_empty() {
            if start.elapsed() > std::time::Duration::from_secs(5) {
                return Err("Timeout reading from MCP server".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        for &b in &chunk {
            if b == b'\n' {
                return String::from_utf8(buffer).map_err(|e| e.to_string());
            }
            buffer.push(b);
        }
    }
}

fn mcp_request(server_name: &str, req_val: &serde_json::Value) -> Result<serde_json::Value, String> {
    init_mcp_servers()?;
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    let servers = servers_guard.as_mut().ok_or("MCP servers not initialized")?;
    let server = servers.get_mut(server_name).ok_or_else(|| format!("MCP server {server_name} not found"))?;

    let req_str = serde_json::to_string(req_val).map_err(|e| e.to_string())?;
    let mut req_bytes = req_str.into_bytes();
    req_bytes.push(b'\n');

    server.stdin.write(&req_bytes)?;

    let res_line = read_line(&server.stdout)?;
    serde_json::from_str(&res_line).map_err(|e| format!("Invalid JSON response: {e}. Raw: {res_line}"))
}

impl Guest for ToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        let mut tools = get_default_tools();

        if let Ok(()) = init_mcp_servers() {
            let mut mapping = HashMap::new();
            let servers_list: Vec<String> = {
                if let Ok(guard) = MCP_SERVERS.lock() {
                    guard.as_ref().map(|m| m.keys().cloned().collect()).unwrap_or_default()
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
                                    let description = t.get("description").and_then(|d| d.as_str()).map(ToString::to_string);
                                    let parameters = t.get("inputSchema").cloned().unwrap_or(serde_json::json!({
                                        "type": "object",
                                        "properties": {}
                                    }));
                                    tools.push(Tool {
                                        tool_type: "function".to_string(),
                                        function: FunctionDefinition {
                                            name: name.to_string(),
                                            description,
                                            parameters,
                                        }
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

                let server_name = mapping.get(other).ok_or_else(|| format!("Unknown tool provider for tool '{other}'"))?;

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
                if let Some(err) = res.get("error").and_then(|e| e.get("message")).and_then(|m| m.as_str()) {
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
    ]
}
