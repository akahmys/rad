use crate::mcp_config::load_mcp_config;
use crate::mcp_transport::read_line;
use crate::open_process;
use crate::radcomp::extension::types as wit;
use rust_mcp_schema::schema_utils::{ClientJsonrpcRequest, RequestFromClient};
use rust_mcp_schema::{
    ClientCapabilities, Implementation, InitializeRequestParams, InitializedNotification,
    ProtocolVersion, RequestId,
};
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

/// Performs the MCP handshake (`initialize` request + `notifications/initialized`)
/// over an already-spawned server's stdin/stdout. `notifications/initialized` has no
/// `id`/response, so it's serialized directly; `initialize` goes through the same
/// `ClientJsonrpcRequest` envelope every other request uses (see `mcp_transport.rs`),
/// keeping this one request typed too rather than a special-cased literal.
fn perform_handshake(
    name: &str,
    stdin: &wit::StreamHandle,
    stdout: &wit::StreamHandle,
) -> Result<(), String> {
    let init_req = ClientJsonrpcRequest::new(
        RequestId::String("init_1".to_string()),
        RequestFromClient::InitializeRequest(InitializeRequestParams {
            protocol_version: ProtocolVersion::latest().to_string(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "rad".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                description: None,
                icons: Vec::new(),
                website_url: None,
            },
            meta: None,
        }),
    );
    let req_str = format!("{init_req}\n");
    stdin
        .write(req_str.as_bytes())
        .map_err(|e| format!("Stdin write failed for '{name}' during initialize: {e}"))?;
    let line =
        read_line(stdout).map_err(|e| format!("Handshake read_line failed for '{name}': {e}"))?;
    diag(&format!("Handshake response from '{name}': {line}"));

    let notif = InitializedNotification::new(None);
    let notif_str = format!("{}\n", serde_json::to_string(&notif).unwrap_or_default());
    stdin.write(notif_str.as_bytes()).map_err(|e| {
        format!("Stdin write failed for '{name}' during notifications/initialized: {e}")
    })?;
    Ok(())
}

pub fn init_mcp_servers() -> Result<(), String> {
    let mut servers_guard = MCP_SERVERS.lock().map_err(|e| e.to_string())?;
    if let Some(active) = servers_guard.as_ref().filter(|s| !s.is_empty()) {
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

    *servers_guard = None;

    let mut active = HashMap::new();
    let Some(config) = load_mcp_config() else {
        diag(
            "load_mcp_config() returned None: no mcp_servers block found in any discovered config file",
        );
        return Err("load_mcp_config returned None (no mcp_servers config found)".to_string());
    };

    if let Some(ref servers) = config.mcp_servers {
        diag(&format!(
            "Found {} server config(s): {:?}",
            servers.len(),
            servers.keys().collect::<Vec<_>>()
        ));
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

            if let Err(e) = perform_handshake(name, &stdin, &stdout) {
                diag(&e);
                continue;
            }

            diag(&format!("'{name}' initialized successfully"));
            active.insert(
                name.clone(),
                ActiveMcpServer {
                    exec,
                    stdin,
                    stdout,
                },
            );
        }
    } else {
        diag("load_mcp_config() succeeded but mcp_servers field was empty/None");
    }

    if active.is_empty() {
        Err(format!(
            "Failed to initialize active MCP servers (found_any={}, active_count=0)",
            config
                .mcp_servers
                .as_ref()
                .map_or(0, std::collections::HashMap::len)
        ))
    } else {
        *servers_guard = Some(active);
        Ok(())
    }
}
