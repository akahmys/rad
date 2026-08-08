//! JSON-RPC line transport over an MCP server's stdio.
//!
//! Ported from `ext/mcp-tool-provider/src/mcp_transport.rs`. The framing logic
//! is unchanged; what differs is that `read` now returns promptly with an empty
//! slice instead of blocking, so the timeout in `read_line` is a timeout rather
//! than decoration — under the extension host it could never fire, because the
//! host blocked in `recv()` until the pipe closed.

use crate::client::{ActiveMcpServer, with_server};
use crate::types::ByteStream;

/// A server that says nothing for this long is treated as wedged. `read` waits
/// 100ms per empty call, so this is a wall-clock ceiling, not a spin.
const READ_LINE_TIMEOUT_POLLS: u32 = 100;

/// How many lines to skip while looking for the one matching our request id.
/// Servers may interleave notifications with responses.
const MAX_UNMATCHED_LINES: u32 = 10;

pub fn read_line(stdout: &ByteStream) -> Result<String, String> {
    let mut buffer = Vec::new();
    let mut empties = 0;
    loop {
        let chunk = stdout
            .read(4096)
            .map_err(|e| format!("Stream error reading from MCP server: {}", e.message))?;
        if chunk.is_empty() {
            empties += 1;
            if empties > READ_LINE_TIMEOUT_POLLS {
                return Err("Timeout reading from MCP server".to_string());
            }
            continue;
        }
        empties = 0;
        for &b in &chunk {
            if b == b'\n' {
                let line = String::from_utf8_lossy(&buffer).trim().to_string();
                if !line.is_empty() {
                    return Ok(line);
                }
                buffer.clear();
            } else {
                buffer.push(b);
            }
        }
    }
}

fn send(
    server_name: &str,
    req_val: &serde_json::Value,
    req_bytes: &[u8],
) -> Result<serde_json::Value, String> {
    with_server(server_name, |server: &ActiveMcpServer| {
        server
            .stdin
            .write(req_bytes)
            .map_err(|e| format!("Stdin write failed for '{server_name}': {}", e.message))?;

        let target_id = req_val.get("id").and_then(|v| v.as_str());
        for _ in 0..MAX_UNMATCHED_LINES {
            let line = read_line(&server.stdout)?;
            let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            if let Some(expected) = target_id {
                if parsed.get("id").and_then(|v| v.as_str()) == Some(expected) {
                    return Ok(parsed);
                }
            } else if parsed.get("jsonrpc").is_some()
                || parsed.get("result").is_some()
                || parsed.get("error").is_some()
            {
                return Ok(parsed);
            }
        }
        Err(format!(
            "Failed to get valid JSON-RPC response from {server_name}"
        ))
    })
}

/// Sends a request, respawning the server set once if the connection is dead.
pub fn request(
    server_name: &str,
    req_val: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    crate::client::init_servers()?;

    let req_str = serde_json::to_string(req_val).map_err(|e| e.to_string())?;
    let mut req_bytes = req_str.into_bytes();
    req_bytes.push(b'\n');

    if let Ok(res) = send(server_name, req_val, &req_bytes) {
        return Ok(res);
    }
    crate::client::forget_servers();
    crate::client::init_servers()?;
    send(server_name, req_val, &req_bytes)
}
