use serde::{Deserialize, Serialize};
use crate::radcomp::extension::types as wit;
use crate::{execute_tool, host_rpc};

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

pub fn execute_tool_sync(name: &str, arguments: &str) -> Result<String, String> {
    let exec = execute_tool(name, arguments)?;
    let stdout = exec.get_stdout();
    let mut output = Vec::new();
    let start = std::time::Instant::now();

    loop {
        let chunk = stdout.read(4096)?;
        if chunk.is_empty() {
            match exec.wait() {
                Ok(_) => {
                    // final drain
                    let last_chunk = stdout.read(4096)?;
                    output.extend(last_chunk);
                    break;
                }
                Err(_) => {
                    if start.elapsed() > std::time::Duration::from_secs(30) {
                        return Err("Tool execution timed out".to_string());
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            }
        } else {
            output.extend(chunk);
        }
    }

    let res_str = String::from_utf8(output).map_err(|e| format!("Invalid UTF-8 from tool: {e}"))?;
    let is_rehydrating = crate::orchestrator::STATE.lock().ok()
        .and_then(|g| g.as_ref().map(|s| s.is_rehydrated))
        .unwrap_or(false);

    if res_str.contains("CRASH_WASM") && !is_rehydrating {
        panic!("Simulated Wasm panic via CRASH_WASM stdout backdoor");
    }
    Ok(res_str)
}

pub fn get_available_tools() -> Result<Vec<Tool>, String> {
    let json_str = host_rpc(&wit::RasRpcCommand::GetTools)?;
    serde_json::from_str(&json_str).map_err(|e| format!("Failed to parse tools: {e}"))
}
