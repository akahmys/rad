use serde::{Deserialize, Serialize};
use crate::types::RasRpcCommand;
use crate::call_host;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Serialize, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Default)]
pub struct ToolCallBuffer {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub fn execute_tool_call(tc: &ToolCallBuffer) -> Result<(), String> {
    match tc.name.as_str() {
        "file_read" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
            }
            let args: Args = serde_json::from_str(&tc.arguments)
                .map_err(|e| format!("Failed to parse file_read args: {e}"))?;
            let _ = call_host(RasRpcCommand::FileRead { path: args.path })?;
        }
        "file_write" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                content: String,
            }
            let args: Args = serde_json::from_str(&tc.arguments)
                .map_err(|e| format!("Failed to parse file_write args: {e}"))?;
            let _ = call_host(RasRpcCommand::FileWrite {
                path: args.path,
                data: args.content.clone().into_bytes(),
            })?;
        }
        "file_edit_patch" => {
            #[derive(Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                diff: String,
            }
            let args: Args = serde_json::from_str(&tc.arguments)
                .map_err(|e| format!("Failed to parse file_edit_patch args: {e}"))?;
            let _ = call_host(RasRpcCommand::FileEditPatch {
                path: args.path,
                diff: args.diff,
            })?;
        }
        "spawn_bash_process" => {
            #[derive(Deserialize)]
            struct Args {
                command: String,
            }
            let args: Args = serde_json::from_str(&tc.arguments)
                .map_err(|e| format!("Failed to parse spawn_bash_process args: {e}"))?;
            let _ = call_host(RasRpcCommand::SpawnBashProcess {
                command: args.command,
            })?;
        }
        other => return Err(format!("Unknown tool call: {other}")),
    }
    Ok(())
}
