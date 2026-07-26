// JSON-RPC line transport over an MCP server's stdio pipes, split out of
// `client.rs` to stay under the 300-line file limit.
use crate::client::MCP_SERVERS;
use crate::radcomp::extension::types as wit;

pub(crate) fn read_line(stdout: &wit::StreamHandle) -> Result<String, String> {
    let mut buffer = Vec::new();
    let start = std::time::Instant::now();
    loop {
        match stdout.read(1024) {
            Ok(chunk) => {
                if chunk.is_empty() {
                    if start.elapsed() > std::time::Duration::from_secs(10) {
                        return Err("Timeout reading from MCP server (10s elapsed)".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
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
            Err(e) => {
                if !buffer.is_empty() {
                    if let Ok(line) = String::from_utf8(buffer.clone()) {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            return Ok(trimmed.to_string());
                        }
                    }
                }
                return Err(format!("Stream error reading from MCP server: {e}"));
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
    crate::client::init_mcp_servers()?;

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
    crate::client::init_mcp_servers()?;
    send_mcp_bytes(server_name, req_val, &req_bytes)
}
