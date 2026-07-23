use crate::open_process;
use crate::radcomp::extension::types as wit;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(serde::Deserialize)]
pub struct ExtensionConfigInfo {
    pub name: String,
    pub config: Option<McpProviderConfig>,
}

#[derive(serde::Deserialize)]
pub struct McpProviderConfig {
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

#[derive(serde::Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct RadJsonConfig {
    pub extensions: Option<Vec<ExtensionConfigInfo>>,
}

pub struct ActiveMcpServer {
    pub stdin: wit::StreamHandle,
    pub stdout: wit::StreamHandle,
}

pub static MCP_SERVERS: Mutex<Option<HashMap<String, ActiveMcpServer>>> = Mutex::new(None);
pub static MCP_TOOL_MAPPING: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

pub fn load_mcp_config() -> Result<Option<McpProviderConfig>, String> {
    let paths = [
        "config.json",
        "rad.json",
        ".rad/config.json",
        ".rad/rad.json",
    ];
    let mut content = None;
    for p in &paths {
        if let Ok(c) = std::fs::read_to_string(p) {
            content = Some(c);
            break;
        }
    }
    if content.is_none() {
        if let Ok(home) = std::env::var("HOME") {
            let user_global = format!("{home}/.rad/config.json");
            if let Ok(c) = std::fs::read_to_string(&user_global) {
                content = Some(c);
            }
        }
    }
    let Some(json_str) = content else {
        return Ok(None);
    };

    let cfg: RadJsonConfig =
        serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse rad.json: {e}"))?;

    let Some(extensions) = cfg.extensions else {
        return Ok(None);
    };

    for ext in extensions {
        if ext.name == "mcp-tool-provider" || ext.name == "rad-orchestrator" {
            if ext.config.is_some() {
                return Ok(ext.config);
            }
        }
    }

    Ok(None)
}

pub fn init_mcp_servers() -> Result<(), String> {
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    if servers_guard.is_some() {
        return Ok(());
    }

    let mut active = HashMap::new();
    let home = std::env::var("HOME").unwrap_or_default();
    if let Some(config) = load_mcp_config()? {
        if let Some(servers) = config.mcp_servers {
            for (name, cfg) in servers {
                let expanded_cmd = if cfg.command.starts_with("~/") && !home.is_empty() {
                    format!("{home}/{}", &cfg.command[2..])
                } else {
                    cfg.command.clone()
                };
                let expanded_args: Vec<String> = cfg
                    .args
                    .iter()
                    .map(|arg| {
                        if arg.starts_with("~/") && !home.is_empty() {
                            format!("{home}/{}", &arg[2..])
                        } else {
                            arg.clone()
                        }
                    })
                    .collect();
                let mut cmd_parts = vec![expanded_cmd];
                cmd_parts.extend(expanded_args);
                let command_line = cmd_parts.join(" ");

                let exec = open_process(&command_line)?;
                let stdin = exec.get_stdin();
                let stdout = exec.get_stdout();

                // Perform MCP handshake
                let init_req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "init_1",
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {
                            "name": "rad",
                            "version": "0.8.0"
                        }
                    }
                });
                let req_str = format!("{}\n", serde_json::to_string(&init_req).unwrap_or_default());
                let _ = stdin.write(req_str.as_bytes());
                let _ = read_line(&stdout);

                let notif = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized",
                    "params": {}
                });
                let notif_str = format!("{}\n", serde_json::to_string(&notif).unwrap_or_default());
                let _ = stdin.write(notif_str.as_bytes());

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

pub fn mcp_request(
    server_name: &str,
    req_val: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    init_mcp_servers()?;
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    let servers = servers_guard
        .as_mut()
        .ok_or("MCP servers not initialized")?;
    let server = servers
        .get_mut(server_name)
        .ok_or_else(|| format!("MCP server {server_name} not found"))?;

    let req_str = serde_json::to_string(req_val).map_err(|e| e.to_string())?;
    let mut req_bytes = req_str.into_bytes();
    req_bytes.push(b'\n');

    server.stdin.write(&req_bytes)?;

    let res_line = read_line(&server.stdout)?;
    serde_json::from_str(&res_line)
        .map_err(|e| format!("Invalid JSON response: {e}. Raw: {res_line}"))
}
