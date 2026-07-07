use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::call_host;
use rad_models::RasRpcCommand;

#[derive(serde::Deserialize)]
struct ExtensionConfigInfo {
    name: String,
    config: Option<OrchestratorConfig>,
}

#[derive(serde::Deserialize)]
struct OrchestratorConfig {
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

fn load_mcp_configs() -> Result<HashMap<String, McpServerConfig>, String> {
    let paths = ["rad.json", ".rad/rad.json"];
    let mut content = None;
    for p in &paths {
        if let Ok(c) = std::fs::read_to_string(p) {
            content = Some(c);
            break;
        }
    }
    let Some(json_str) = content else {
        return Ok(HashMap::new());
    };

    let cfg: RadJsonConfig = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse rad.json for MCP: {e}"))?;

    let Some(extensions) = cfg.extensions else {
        return Ok(HashMap::new());
    };

    for ext in extensions {
        if ext.name == "openai-orchestrator" {
            if let Some(config) = ext.config {
                if let Some(servers) = config.mcp_servers {
                    return Ok(servers);
                }
            }
        }
    }

    Ok(HashMap::new())
}

/// Spawns the MCP servers configured in rad.json.
/// Guarantees that spawning is executed only once.
pub fn init_mcp_servers() -> Result<(), String> {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return Ok(());
    }

    let servers = load_mcp_configs()?;
    for (name, cfg) in servers {
        let payload = RasRpcCommand::SpawnMcpServer {
            name,
            command: cfg.command,
            args: cfg.args,
        };
        let _ = call_host(payload)?;
    }

    Ok(())
}

/// Returns a list of configured MCP server names.
pub fn get_configured_mcp_names() -> Result<Vec<String>, String> {
    let servers = load_mcp_configs()?;
    Ok(servers.keys().cloned().collect())
}

#[derive(serde::Deserialize)]
struct McpListToolsResponse {
    result: McpListToolsResult,
}

#[derive(serde::Deserialize)]
struct McpListToolsResult {
    tools: Vec<McpToolDefinition>,
}

#[derive(serde::Deserialize)]
struct McpToolDefinition {
    name: String,
    description: Option<String>,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

/// Parses the tools/list JSON response from an MCP server and maps it to LLM Tool structures.
pub fn parse_mcp_tools(json_str: &str) -> Result<Vec<crate::tool::Tool>, String> {
    let resp: McpListToolsResponse = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse tools/list JSON: {e}"))?;

    let mut tools = Vec::new();
    for t in resp.result.tools {
        tools.push(crate::tool::Tool {
            tool_type: "function".to_string(),
            function: crate::tool::FunctionDefinition {
                name: t.name,
                description: t.description,
                parameters: t.input_schema,
            },
        });
    }
    Ok(tools)
}

#[derive(serde::Deserialize)]
struct McpCallResponse {
    id: serde_json::Value,
    result: Option<McpCallResult>,
    error: Option<McpCallError>,
}

#[derive(serde::Deserialize)]
struct McpCallResult {
    content: Option<Vec<McpContentItem>>,
}

#[derive(serde::Deserialize)]
struct McpContentItem {
    #[serde(rename = "type")]
    item_type: String,
    text: Option<String>,
}

#[derive(serde::Deserialize)]
struct McpCallError {
    message: String,
}

/// Parses MCP tools/call responses and returns a tuple of (tool_call_id, content_string).
pub fn parse_mcp_call_response(json_str: &str) -> Result<(String, String), String> {
    let resp: McpCallResponse = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse MCP response JSON: {e}"))?;

    let id_str = match resp.id {
        serde_json::Value::String(s) => s,
        other => other.to_string(),
    };
    if !id_str.starts_with("mcp_call:") {
        return Err(format!("Invalid MCP call response ID format: {id_str}"));
    }
    let tool_call_id = id_str["mcp_call:".len()..].to_string();

    if let Some(err) = resp.error {
        return Ok((tool_call_id, format!("Error from MCP server: {}", err.message)));
    }

    if let Some(res) = resp.result {
        if let Some(content) = res.content {
            let texts: Vec<String> = content.into_iter()
                .filter(|item| item.item_type == "text")
                .filter_map(|item| item.text)
                .collect();
            return Ok((tool_call_id, texts.join("\n")));
        }
    }

    Ok((tool_call_id, "No content returned from MCP server.".to_string()))
}


