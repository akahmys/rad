use crate::radcomp::extension::types as wit;
use crate::{execute_tool_text, host_rpc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Default, Clone)]
pub struct ToolCallBuffer {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

/// Runs a tool and returns its output.
///
/// Uses `execute-tool-text` rather than `execute-tool`: this function only ever
/// wanted the string, and the drain loop it used to run — read, poll `wait`,
/// sleep, drain again, 30s ceiling — now lives on the host, where a module can
/// simply return its answer without a process in between.
pub fn execute_tool_sync(name: &str, arguments: &str) -> Result<String, String> {
    let res_str = execute_tool_text(name, arguments)?;
    let is_rehydrating = crate::orchestrator::STATE
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.is_rehydrated))
        .unwrap_or(false);

    assert!(
        !res_str.contains("CRASH_WASM") || is_rehydrating,
        "Simulated Wasm panic via CRASH_WASM stdout backdoor"
    );
    Ok(res_str)
}

pub fn get_available_tools() -> Result<Vec<Tool>, String> {
    let json_str = host_rpc(&wit::RasRpcCommand::GetTools)?;
    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse tools: {e}"))
}
