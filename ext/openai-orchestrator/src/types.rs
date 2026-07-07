use std::collections::HashMap;
use crate::tool::ToolCallBuffer;

pub use rad_models::{RasRpcCommand, RasCoreEvent, Dag};

pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub pgid: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub result: Option<String>,
}

pub struct OrchestratorState {
    pub assistant: String,
    pub stream: String,
    pub tool_calls: HashMap<usize, ToolCallBuffer>,
    pub pending_tool_calls: Vec<PendingToolCall>,
    pub expected_mcp_servers: Vec<String>,
    pub mcp_tools: Vec<crate::tool::Tool>,
    pub mcp_tool_providers: HashMap<String, String>,
    pub max_history_messages: Option<usize>,
    pub max_tool_output_chars: Option<usize>,
}
