#![deny(clippy::pedantic)]

#[allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::pedantic
)]
mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/rad.wit",
        world: "rad-tool-provider",
    });

    use super::ToolProviderImpl;
    export!(ToolProviderImpl);
}

pub use bindings::*;

use self::radcomp::extension::types as wit;

use std::collections::HashMap;

mod conv;

struct ToolProviderImpl;

mod client;
mod default_tools;
mod mcp_config;
mod mcp_transport;

use client::{MCP_SERVERS, MCP_TOOL_MAPPING, MCP_TOOLS_CACHE, diag, init_mcp_servers, mcp_request};
use default_tools::{FunctionDefinition, Tool};
use rust_mcp_schema::schema_utils::{ClientJsonrpcRequest, RequestFromClient};
use rust_mcp_schema::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsResult, RequestId,
};

/// Issues a live `tools/list` to every configured MCP server, rebuilding
/// both the returned tool list and the `MCP_TOOL_MAPPING`/`MCP_TOOLS_CACHE`
/// caches from the result. Only called when the cache is empty or
/// `init_mcp_servers` had to (re)spawn a server — see `get_tools`.
fn fetch_and_cache_mcp_tools() -> Result<Vec<Tool>, String> {
    let mut mapping = HashMap::new();
    let servers_list: Vec<String> = {
        if let Ok(guard) = MCP_SERVERS.lock() {
            guard
                .as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default()
        } else {
            Vec::new()
        }
    };

    // `get_tools` never reaches this function in `RAD_TEST_PORT` mode (it
    // returns synthetic tools directly), so an empty server list here is
    // always a real misconfiguration.
    if servers_list.is_empty() {
        return Err("MCP_SERVERS is empty after init_mcp_servers".to_string());
    }

    let mut fetched_tools = Vec::new();
    for server_name in servers_list {
        let list_req = ClientJsonrpcRequest::new(
            RequestId::String("list_tools".to_string()),
            RequestFromClient::ListToolsRequest(None),
        );
        let Ok(req) = serde_json::to_value(&list_req) else {
            diag(&format!(
                "Failed to serialize tools/list request for '{server_name}'"
            ));
            continue;
        };
        match mcp_request(&server_name, &req) {
            Err(e) => {
                diag(&format!(
                    "tools/list request failed for '{server_name}': {e}"
                ));
            }
            Ok(res) => {
                if let Some(err) = res.get("error") {
                    diag(&format!(
                        "tools/list returned JSON-RPC error for '{server_name}': {err}"
                    ));
                }
                if let Ok(list_result) = serde_json::from_value::<ListToolsResult>(
                    res.get("result").cloned().unwrap_or_default(),
                ) {
                    diag(&format!(
                        "'{server_name}' returned {} tool(s)",
                        list_result.tools.len()
                    ));
                    for t in list_result.tools {
                        mapping.insert(t.name.clone(), server_name.clone());
                        let parameters = serde_json::to_value(&t.input_schema)
                            .unwrap_or(serde_json::json!({ "type": "object", "properties": {} }));
                        fetched_tools.push(Tool {
                            tool_type: "function".to_string(),
                            function: FunctionDefinition {
                                name: t.name,
                                description: t.description,
                                parameters,
                            },
                        });
                    }
                }
            }
        }
    }
    if let Ok(mut map_guard) = MCP_TOOL_MAPPING.lock() {
        *map_guard = Some(mapping);
    }
    if let Ok(mut cache_guard) = MCP_TOOLS_CACHE.lock() {
        *cache_guard = Some(fetched_tools.clone());
    }
    Ok(fetched_tools)
}

impl Guest for ToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        // Self-contained test mode: return the synthetic tools directly,
        // without ever touching real MCP server discovery/config (which
        // real tests running in this mode don't configure at all — see
        // `execute_tool`'s matching RAD_TEST_PORT short-circuit below,
        // which already does this correctly).
        if std::env::var("RAD_TEST_PORT").is_ok() {
            let tools = vec![
                Tool {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: "read".to_string(),
                        description: Some("Read file content".to_string()),
                        parameters: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}}}),
                    },
                },
                Tool {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: "write".to_string(),
                        description: Some("Write file content".to_string()),
                        parameters: serde_json::json!({"type": "object", "properties": {"path": {"type": "string"}, "content": {"type": "string"}}}),
                    },
                },
                Tool {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: "execute".to_string(),
                        description: Some("Execute bash command".to_string()),
                        parameters: serde_json::json!({"type": "object", "properties": {"command": {"type": "string"}}}),
                    },
                },
            ];
            return serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"));
        }

        let mut tools = Vec::new();
        let did_reinit = init_mcp_servers()?;

        // MCP servers' tool lists don't change mid-session, so once
        // `init_mcp_servers` confirms the existing connections are still
        // alive (`did_reinit == false`), reuse the last fetch instead of
        // re-issuing `tools/list` to every server — this function runs on
        // every LLM turn and every tool-call routing lookup, so an
        // unconditional live re-fetch here was pure waste on the common
        // path. Invalidated by `init_mcp_servers` itself whenever it has to
        // actually (re)spawn a server.
        let cached = if did_reinit {
            None
        } else {
            MCP_TOOLS_CACHE.lock().ok().and_then(|guard| guard.clone())
        };

        match cached {
            Some(cached_tools) => tools.extend(cached_tools),
            None => tools.extend(fetch_and_cache_mcp_tools()?),
        }

        serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"))
    }

    fn execute_tool(name: String, arguments: String) -> Result<wit::ExecutionHandle, String> {
        if std::env::var("RAD_TEST_PORT").is_ok() {
            let args_json: serde_json::Value = serde_json::from_str(&arguments).unwrap_or_default();
            let path = args_json
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or("out.txt");
            let cmd = args_json
                .get("command")
                .and_then(|c| c.as_str())
                .unwrap_or("");
            let target_cmd = if name == "execute" && !cmd.is_empty() {
                cmd.to_string()
            } else {
                format!("echo -n 'test' > '{path}'")
            };
            match open_process(&target_cmd) {
                Ok(h) => return Ok(h),
                Err(e) => {
                    let escaped_e = e.replace('\'', "'\\''");
                    return open_process(&format!("echo -n '{escaped_e}'"));
                }
            }
        }

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

        let server_name = mapping
            .get(&name)
            .ok_or_else(|| format!("Unknown tool provider for tool '{name}'"))?;

        let args_json: serde_json::Value = serde_json::from_str(&arguments)
            .map_err(|e| format!("Failed to parse MCP tool args: {e}"))?;
        let args_map = match args_json {
            serde_json::Value::Object(map) => Some(map),
            _ => None,
        };
        let call_req = ClientJsonrpcRequest::new(
            RequestId::String("mcp_call".to_string()),
            RequestFromClient::CallToolRequest(CallToolRequestParams {
                name: name.clone(),
                arguments: args_map,
                meta: None,
                task: None,
            }),
        );
        let req = serde_json::to_value(&call_req)
            .map_err(|e| format!("Failed to serialize tools/call request: {e}"))?;

        let res = mcp_request(server_name, &req)?;

        // Parse tool call result
        let mut result_text = String::new();
        if let Some(err) = res
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
        {
            result_text = format!("Error from MCP server: {err}");
        } else if let Ok(call_result) =
            serde_json::from_value::<CallToolResult>(res.get("result").cloned().unwrap_or_default())
        {
            let texts: Vec<String> = call_result
                .content
                .into_iter()
                .filter_map(|block| match block {
                    ContentBlock::TextContent(t) => Some(t.text),
                    _ => None,
                })
                .collect();
            result_text = texts.join("\n");
        }
        if result_text.is_empty() {
            result_text = "No content returned from MCP server.".to_string();
        }

        let escaped_result = result_text.replace('\'', "'\\''");
        open_process(&format!("echo -n '{escaped_result}'"))
    }
}
