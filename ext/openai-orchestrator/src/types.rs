use crate::tool::ToolCallBuffer;
use std::collections::HashMap;

pub use rad_models::{Dag, RasCoreEvent, RasRpcCommand};

pub struct PendingToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
}

pub struct OrchestratorState {
    pub assistant: String,
    pub is_reasoning: bool,
    pub reasoning_buffered: String,
    pub tool_calls: HashMap<usize, ToolCallBuffer>,
    pub max_history_messages: Option<usize>,
    pub max_tool_output_chars: Option<usize>,
    pub is_rehydrated: bool,
}
