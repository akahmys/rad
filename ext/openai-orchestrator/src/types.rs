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
}
