use crate::open_process;
use crate::radcomp::extension::types as wit;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(serde::Deserialize)]
pub struct McpProviderConfig {
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

#[derive(serde::Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

pub struct ActiveMcpServer {
    pub stdin: wit::StreamHandle,
    pub stdout: wit::StreamHandle,
}

pub static MCP_SERVERS: Mutex<Option<HashMap<String, ActiveMcpServer>>> = Mutex::new(None);
pub static MCP_TOOL_MAPPING: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn strip_json_comments(json_str: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut chars = json_str.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_line_comment {
            if c == '\n' {
                in_single_line_comment = false;
                result.push(c);
            }
            continue;
        }
        if in_multi_line_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_multi_line_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(&next_c) = chars.peek() {
                    result.push(next_c);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            if chars.peek() == Some(&'/') {
                chars.next();
                in_single_line_comment = true;
                continue;
            }
            if chars.peek() == Some(&'*') {
                chars.next();
                in_multi_line_comment = true;
                continue;
            }
        }

        result.push(c);
    }

    result
}

fn read_config_file(path: &str) -> Option<String> {
    let cmd = wit::RasRpcCommand::FileRead(path.to_string());
    if let Ok(res_str) = crate::host_rpc(&cmd) {
        if !res_str.is_empty() && res_str != "null" {
            if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&res_str) {
                if let Ok(s) = String::from_utf8(bytes) {
                    let cleaned = strip_json_comments(&s);
                    if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                        return Some(cleaned);
                    }
                }
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&res_str) {
                if let Some(s) = val.as_str() {
                    let cleaned = strip_json_comments(s);
                    if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                        return Some(cleaned);
                    }
                }
                if val.is_object() {
                    let cleaned = strip_json_comments(&res_str);
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

pub fn load_mcp_config() -> Option<McpProviderConfig> {
    if std::env::var("RAD_TEST_PORT").is_ok() {
        return Some(McpProviderConfig {
            mcp_servers: Some(HashMap::new()),
        });
    }

    let mut merged_servers: HashMap<String, McpServerConfig> = HashMap::new();
    let mut found_any = false;

    // Search paths in reverse precedence (global base first, local overrides last)
    let mut paths = vec![
        "~/.rad/config.json".to_string(),
        ".rad/rad.json".to_string(),
        "config.json".to_string(),
        ".rad/config.json".to_string(),
        "rad.json".to_string(),
    ];

    if let Ok(home) = std::env::var("HOME") {
        let user_global = format!("{home}/.rad/config.json");
        if !paths.contains(&user_global) {
            paths.insert(0, user_global);
        }
    }

    for p in &paths {
        if let Some(c) = read_config_file(p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                if let Some(exts) = v.get("extensions").and_then(|e| e.as_array()) {
                    for ext in exts {
                        if ext.get("name").and_then(|n| n.as_str()) == Some("mcp-tool-provider") {
                            if let Some(cfg_val) = ext.get("config") {
                                if let Ok(parsed_cfg) = serde_json::from_value::<McpProviderConfig>(cfg_val.clone()) {
                                    if let Some(servers) = parsed_cfg.mcp_servers {
                                        for (srv_name, srv_cfg) in servers {
                                            merged_servers.insert(srv_name, srv_cfg);
                                            found_any = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if found_any {
        Some(McpProviderConfig {
            mcp_servers: Some(merged_servers),
        })
    } else {
        None
    }
}

pub fn init_mcp_servers() -> Result<(), String> {
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    if let Some(ref active) = *servers_guard {
        if !active.is_empty() {
            let mut valid = true;
            for server in active.values() {
                if server.stdin.write(b"").is_err() {
                    valid = false;
                    break;
                }
            }
            if valid {
                return Ok(());
            }
        }
    }

    *servers_guard = None;

    let mut active = HashMap::new();
    let Some(config) = load_mcp_config() else {
        return Err("load_mcp_config returned None (no mcp_servers config found)".to_string());
    };

    if let Some(servers) = config.mcp_servers {
        let _ = open_process(&format!("echo '[MCP Diagnostic] Found {} server configs' >&2", servers.len()));
        for (name, cfg) in servers {
            let mut cmd_parts = vec![cfg.command.clone()];
            cmd_parts.extend(cfg.args.clone());
            let command_line = cmd_parts.join(" ");

            let _ = open_process(&format!("echo '[MCP Diagnostic] Spawning {name}: {command_line}' >&2"));

            let exec = match open_process(&command_line) {
                Ok(exec) => exec,
                Err(e) => {
                    let _ = open_process(&format!("echo '[MCP Diagnostic] Failed to spawn {name}: {e}' >&2"));
                    continue;
                }
            };
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
            if let Err(e) = stdin.write(req_str.as_bytes()) {
                let _ = open_process(&format!("echo '[MCP Diagnostic] Stdin write failed for {name}: {e}' >&2"));
                continue;
            }
            if let Err(e) = read_line(&stdout) {
                let _ = open_process(&format!("echo '[MCP Diagnostic] Handshake read_line failed for {name}: {e}' >&2"));
                continue;
            }

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

    if !active.is_empty() {
        *servers_guard = Some(active);
    }
    Ok(())
}

fn read_line(stdout: &wit::StreamHandle) -> Result<String, String> {
    let mut buffer = Vec::new();
    let start = std::time::Instant::now();
    loop {
        let chunk = stdout.read(1024)?;
        if chunk.is_empty() {
            if start.elapsed() > std::time::Duration::from_secs(10) {
                return Err("Timeout reading from MCP server (10s elapsed)".to_string());
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }
        for &b in &chunk {
            if b == b'\n' {
                let line = String::from_utf8(buffer.clone()).map_err(|e| e.to_string())?;
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    return Ok(trimmed.to_string());
                }
                buffer.clear();
            } else {
                buffer.push(b);
            }
        }
    }
}

fn send_mcp_bytes(
    server_name: &str,
    req_val: &serde_json::Value,
    req_bytes: &[u8],
) -> Result<serde_json::Value, String> {
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    let servers = servers_guard
        .as_mut()
        .ok_or("MCP servers not initialized")?;
    let server = servers
        .get_mut(server_name)
        .ok_or_else(|| format!("MCP server {server_name} not found"))?;

    server.stdin.write(req_bytes)?;

    let target_id = req_val.get("id").and_then(|v| v.as_str());

    for _ in 0..10 {
        let res_line = read_line(&server.stdout)?;
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&res_line) {
            if let Some(expected_id) = target_id {
                if parsed.get("id").and_then(|v| v.as_str()) == Some(expected_id) {
                    return Ok(parsed);
                }
            } else if parsed.get("jsonrpc").is_some() || parsed.get("result").is_some() || parsed.get("error").is_some() {
                return Ok(parsed);
            }
        }
    }

    Err(format!("Failed to get valid JSON-RPC response from {server_name}"))
}

pub fn mcp_request(
    server_name: &str,
    req_val: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    init_mcp_servers()?;

    let req_str = serde_json::to_string(req_val).map_err(|e| e.to_string())?;
    let mut req_bytes = req_str.into_bytes();
    req_bytes.push(b'\n');

    if let Ok(res) = send_mcp_bytes(server_name, req_val, &req_bytes) {
        return Ok(res);
    }

    // Connection dead or mangled -> clear dead server cache and re-initialize
    if let Ok(mut guard) = MCP_SERVERS.lock() {
        *guard = None;
    }
    init_mcp_servers()?;
    send_mcp_bytes(server_name, req_val, &req_bytes)
}
