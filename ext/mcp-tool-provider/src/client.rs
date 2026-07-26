use crate::mcp_config::load_mcp_config;
use crate::mcp_transport::read_line;
use crate::open_process;
use crate::radcomp::extension::types as wit;
use std::collections::HashMap;
use std::sync::Mutex;

pub use crate::mcp_transport::mcp_request;

pub struct ActiveMcpServer {
    // Kept alive for the lifetime of the connection: dropping this resource
    // triggers RunningProcess::drop() -> kill_group() on the host side, which
    // SIGKILLs the spawned MCP server. Must not be discarded after extracting
    // stdin/stdout, or the process dies right after the handshake completes.
    #[allow(dead_code)]
    pub exec: wit::ExecutionHandle,
    pub stdin: wit::StreamHandle,
    pub stdout: wit::StreamHandle,
}

pub static MCP_SERVERS: Mutex<Option<HashMap<String, ActiveMcpServer>>> = Mutex::new(None);
pub static MCP_TOOL_MAPPING: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

/// Prints a diagnostic line directly to the human-visible terminal output via host RPC.
/// Silent by default; set `RAD_DEBUG=1` in the environment to re-enable while troubleshooting
/// (same flag used by the core's [DEBUG]/[TRACE]/[Thinking] output).
pub fn diag(msg: &str) {
    if std::env::var("RAD_DEBUG").is_err() {
        return;
    }
    let _ = crate::host_rpc(&wit::RasRpcCommand::WriteStdout(format!(
        "\x1b[36m[MCP Diagnostic] {msg}\x1b[0m"
    )));
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
        diag("load_mcp_config() returned None: no mcp_servers block found in any discovered config file");
        return Err("load_mcp_config returned None (no mcp_servers config found)".to_string());
    };

    if let Some(ref servers) = config.mcp_servers {
        diag(&format!("Found {} server config(s): {:?}", servers.len(), servers.keys().collect::<Vec<_>>()));
        for (name, cfg) in servers {
            let mut cmd_parts = vec![cfg.command.clone()];
            cmd_parts.extend(cfg.args.clone());
            let command_line = cmd_parts.join(" ");

            diag(&format!("Spawning '{name}': {command_line}"));
            let exec = match open_process(&command_line) {
                Ok(exec) => exec,
                Err(e) => {
                    diag(&format!("Failed to spawn '{name}': {e}"));
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
                diag(&format!("Stdin write failed for '{name}' during initialize: {e}"));
                continue;
            }
            match read_line(&stdout) {
                Ok(line) => diag(&format!("Handshake response from '{name}': {line}")),
                Err(e) => {
                    diag(&format!("Handshake read_line failed for '{name}': {e}"));
                    continue;
                }
            }

            let notif = serde_json::json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized",
                "params": {}
            });
            let notif_str = format!("{}\n", serde_json::to_string(&notif).unwrap_or_default());
            if let Err(e) = stdin.write(notif_str.as_bytes()) {
                diag(&format!("Stdin write failed for '{name}' during notifications/initialized: {e}"));
            }

            diag(&format!("'{name}' initialized successfully"));
            active.insert(name.clone(), ActiveMcpServer { exec, stdin, stdout });
        }
    } else {
        diag("load_mcp_config() succeeded but mcp_servers field was empty/None");
    }

    if active.is_empty() {
        Err(format!(
            "Failed to initialize active MCP servers (found_any={}, active_count=0)",
            config.mcp_servers.as_ref().map_or(0, std::collections::HashMap::len)
        ))
    } else {
        *servers_guard = Some(active);
        Ok(())
    }
}
