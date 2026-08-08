//! Spawning MCP servers and completing their handshake.
//!
//! Ported from `ext/mcp-tool-provider/src/client.rs`. Two things shrink rather
//! than move.
//!
//! **Configuration.** The extension had no way to be told its own settings, so
//! `mcp_config.rs` went looking: six candidate paths, a hand-written JSON
//! comment stripper, and three successive guesses at what shape `FileRead` had
//! returned the file in. All of it existed to answer "what is my config?".
//! `kernel.config` answers that in one call, so 175 lines become the handful
//! below.
//!
//! **Spawning.** The extension joined `command` and `args` into one string for
//! `open_process`, which then split it back apart on whitespace — mangling any
//! argument containing a space. `proc-spawn` takes the argv it already had.

use crate::types::{ByteStream, Process};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(serde::Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(serde::Deserialize, Default)]
pub struct McpConfig {
    #[serde(default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

pub struct ActiveMcpServer {
    /// Held for the connection's lifetime. Dropping it makes the kernel kill
    /// the process group, so discarding it after taking stdin/stdout would kill
    /// the server the moment the handshake finished.
    #[allow(dead_code)]
    pub process: Process,
    pub stdin: ByteStream,
    pub stdout: ByteStream,
}

// `Process` and `ByteStream` are guest-side resource handles: this module is
// single-threaded inside its own store, and the kernel serialises calls into it
// with a per-module lock.
unsafe impl Send for ActiveMcpServer {}

pub static SERVERS: Mutex<Option<HashMap<String, ActiveMcpServer>>> = Mutex::new(None);
/// tool name -> server name.
pub static TOOL_MAPPING: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
pub static TOOLS_CACHE: Mutex<Option<Vec<crate::tool::Tool>>> = Mutex::new(None);

/// Diagnostics go to the kernel's log rather than to the user's terminal.
///
/// The extension wrote straight to stdout through a `WriteStdout` host RPC,
/// gated behind `RAD_DEBUG` so it would not corrupt the display. `syscall::log`
/// is the kernel's own channel, so the gate is the kernel's to apply.
pub fn diag(msg: &str) {
    crate::syscall::log("mcp", "debug", msg);
}

pub fn config() -> McpConfig {
    match crate::dispatch::call("kernel", "kernel.config", "") {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(e) => {
            diag(&format!("kernel.config failed: {e}"));
            McpConfig::default()
        }
    }
}

/// Runs `f` against one live server.
pub fn with_server<T>(
    name: &str,
    f: impl FnOnce(&ActiveMcpServer) -> Result<T, String>,
) -> Result<T, String> {
    let guard = SERVERS.lock().map_err(|e| e.to_string())?;
    let servers = guard.as_ref().ok_or("MCP servers not initialized")?;
    let server = servers
        .get(name)
        .ok_or_else(|| format!("MCP server {name} not found"))?;
    f(server)
}

pub fn forget_servers() {
    if let Ok(mut guard) = SERVERS.lock() {
        *guard = None;
    }
    if let Ok(mut guard) = TOOLS_CACHE.lock() {
        *guard = None;
    }
}

/// `initialize` request plus the `notifications/initialized` that follows it.
fn handshake(name: &str, stdin: &ByteStream, stdout: &ByteStream) -> Result<(), String> {
    use rust_mcp_schema::schema_utils::{ClientJsonrpcRequest, RequestFromClient};
    use rust_mcp_schema::{
        ClientCapabilities, Implementation, InitializeRequestParams, InitializedNotification,
        ProtocolVersion, RequestId,
    };

    let init = ClientJsonrpcRequest::new(
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
    stdin.write(format!("{init}\n").as_bytes()).map_err(|e| {
        format!(
            "Stdin write failed for '{name}' during initialize: {}",
            e.message
        )
    })?;
    let line = crate::transport::read_line(stdout)
        .map_err(|e| format!("Handshake read failed for '{name}': {e}"))?;
    diag(&format!("Handshake response from '{name}': {line}"));

    let notif = InitializedNotification::new(None);
    let notif_str = format!("{}\n", serde_json::to_string(&notif).unwrap_or_default());
    stdin.write(notif_str.as_bytes()).map_err(|e| {
        format!(
            "Stdin write failed for '{name}' during notifications/initialized: {}",
            e.message
        )
    })?;
    Ok(())
}

/// `Ok(true)` if servers were actually (re)spawned, `Ok(false)` if the existing
/// connections were confirmed alive and reused. Callers use it to decide
/// whether cached tool lists are still valid.
pub fn init_servers() -> Result<bool, String> {
    let mut guard = SERVERS.lock().map_err(|e| e.to_string())?;
    if let Some(active) = guard.as_ref().filter(|s| !s.is_empty()) {
        // An empty write is the cheapest liveness probe: it fails once the
        // child is gone and its pipe is closed.
        if active.values().all(|s| s.stdin.write(b"").is_ok()) {
            return Ok(false);
        }
    }

    *guard = None;
    if let Ok(mut cache) = TOOLS_CACHE.lock() {
        *cache = None;
    }

    let configured = config().mcp_servers;
    if configured.is_empty() {
        return Err("no mcp_servers configured for this module".to_string());
    }
    diag(&format!(
        "Found {} server config(s): {:?}",
        configured.len(),
        configured.keys().collect::<Vec<_>>()
    ));

    let mut active = HashMap::new();
    for (name, cfg) in configured {
        let mut argv = vec![cfg.command.clone()];
        argv.extend(cfg.args.clone());
        diag(&format!("Spawning '{name}': {argv:?}"));

        let process = match crate::syscall::proc_spawn(&argv) {
            Ok(p) => p,
            Err(e) => {
                diag(&format!("Failed to spawn '{name}': {}", e.message));
                continue;
            }
        };
        let stdin = process.stdin();
        let stdout = process.stdout();
        if let Err(e) = handshake(&name, &stdin, &stdout) {
            diag(&e);
            continue;
        }
        diag(&format!("'{name}' initialized successfully"));
        active.insert(
            name,
            ActiveMcpServer {
                process,
                stdin,
                stdout,
            },
        );
    }

    if active.is_empty() {
        Err("no MCP server could be started".to_string())
    } else {
        *guard = Some(active);
        Ok(true)
    }
}
