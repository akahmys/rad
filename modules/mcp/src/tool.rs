//! `tools/list` and `tools/call` against the configured servers.

use crate::client::{TOOL_MAPPING, TOOLS_CACHE, diag, init_servers};
use rust_mcp_schema::schema_utils::{ClientJsonrpcRequest, RequestFromClient};
use rust_mcp_schema::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult, RequestId,
};
use std::collections::HashMap;

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

/// Issues a live `tools/list` to every server and rebuilds both caches.
fn fetch_and_cache() -> Result<Vec<Tool>, String> {
    let names: Vec<String> = crate::client::SERVERS
        .lock()
        .map_err(|e| e.to_string())?
        .as_ref()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    if names.is_empty() {
        return Err("no MCP servers are running after initialization".to_string());
    }

    let mut mapping = HashMap::new();
    let mut tools = Vec::new();
    for server in names {
        let req = ClientJsonrpcRequest::new(
            RequestId::String("list_tools".to_string()),
            RequestFromClient::ListToolsRequest(None),
        );
        let Ok(req) = serde_json::to_value(&req) else {
            diag(&format!("could not serialize tools/list for '{server}'"));
            continue;
        };
        let res = match crate::transport::request(&server, &req) {
            Ok(res) => res,
            Err(e) => {
                diag(&format!("tools/list failed for '{server}': {e}"));
                continue;
            }
        };
        if let Some(err) = res.get("error") {
            diag(&format!("tools/list error from '{server}': {err}"));
        }
        let Ok(listed) = serde_json::from_value::<ListToolsResult>(
            res.get("result").cloned().unwrap_or_default(),
        ) else {
            continue;
        };
        diag(&format!(
            "'{server}' returned {} tool(s)",
            listed.tools.len()
        ));
        for t in listed.tools {
            mapping.insert(t.name.clone(), server.clone());
            tools.push(Tool {
                tool_type: "function".to_string(),
                function: FunctionDefinition {
                    name: t.name,
                    description: t.description,
                    parameters: serde_json::to_value(&t.input_schema)
                        .unwrap_or(serde_json::json!({ "type": "object", "properties": {} })),
                },
            });
        }
    }

    if let Ok(mut guard) = TOOL_MAPPING.lock() {
        *guard = Some(mapping);
    }
    if let Ok(mut guard) = TOOLS_CACHE.lock() {
        *guard = Some(tools.clone());
    }
    Ok(tools)
}

pub fn list() -> Result<Vec<Tool>, String> {
    // A server's tool list does not change mid-session, so once the existing
    // connections are confirmed alive, reuse the last fetch. This runs on every
    // LLM turn and every tool-call lookup; an unconditional re-fetch was pure
    // waste on the common path.
    let respawned = init_servers()?;
    let cached = if respawned {
        None
    } else {
        TOOLS_CACHE.lock().ok().and_then(|g| g.clone())
    };
    match cached {
        Some(tools) => Ok(tools),
        None => fetch_and_cache(),
    }
}

pub fn call(name: &str, arguments: &str) -> Result<String, String> {
    let mapping = {
        let known = TOOL_MAPPING.lock().map_err(|e| e.to_string())?.clone();
        if let Some(m) = known {
            m
        } else {
            // Never listed in this session: one list populates the mapping.
            list()?;
            TOOL_MAPPING
                .lock()
                .map_err(|e| e.to_string())?
                .clone()
                .ok_or("MCP tool mapping unavailable")?
        }
    };
    let server = mapping
        .get(name)
        .ok_or_else(|| format!("Unknown tool provider for tool '{name}'"))?;

    let parsed: serde_json::Value =
        serde_json::from_str(arguments).map_err(|e| format!("Failed to parse tool args: {e}"))?;
    let req = ClientJsonrpcRequest::new(
        RequestId::String("mcp_call".to_string()),
        RequestFromClient::CallToolRequest(CallToolRequestParams {
            name: name.to_string(),
            arguments: match parsed {
                serde_json::Value::Object(map) => Some(map),
                _ => None,
            },
            meta: None,
            task: None,
        }),
    );
    let req = serde_json::to_value(&req)
        .map_err(|e| format!("Failed to serialize tools/call request: {e}"))?;

    let res = crate::transport::request(server, &req)?;
    Ok(render_result(&res))
}

/// Flattens a `tools/call` reply into the text the model sees.
fn render_result(res: &serde_json::Value) -> String {
    if let Some(err) = res
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
    {
        return format!("Error from MCP server: {err}");
    }
    let Ok(result) =
        serde_json::from_value::<CallToolResult>(res.get("result").cloned().unwrap_or_default())
    else {
        return "No content returned from MCP server.".to_string();
    };
    let is_error = result.is_error.unwrap_or(false);
    let text = result
        .content
        .into_iter()
        .filter_map(|block| match block {
            ContentBlock::TextContent(t) => Some(t.text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    if text.is_empty() {
        return "No content returned from MCP server.".to_string();
    }
    // Servers signal tool-level failure with `isError` rather than a JSON-RPC
    // error, so it is normalised to a leading `Error:` — rad-orchestrator's
    // consecutive-failure circuit breaker looks for that rather than depending
    // on any one server's wording.
    if is_error && !text.trim_start().starts_with("Error:") {
        format!("Error: {text}")
    } else {
        text
    }
}

#[cfg(test)]
mod tests;
