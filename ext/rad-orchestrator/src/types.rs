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
    /// Name of the most recently executed tool, used to detect a run of
    /// consecutive failures of the *same* tool (see
    /// `orchestrator::runner::done`'s circuit breaker).
    pub last_tool_name: Option<String>,
    /// How many times `last_tool_name` has failed in a row. Reset to 0 on
    /// any success, and to 1 (not 0) when a *different* tool fails, since
    /// that failure itself starts a new streak.
    pub consecutive_tool_failures: u32,
    pub max_consecutive_tool_failures: Option<u32>,
    /// How many times this turn has already been re-issued with a reduced
    /// context budget after the backend rejected it for exceeding its
    /// context window (L3 recovery — see `context_recovery`). Bounded by
    /// `MAX_CONTEXT_RETRIES`; reset when a turn completes normally.
    pub context_retries_used: u32,
    /// Percentage the computed character budget is scaled by before being
    /// handed to `context-tools`. 100 in the normal case; reduced on each
    /// L3 retry so the re-issued request is strictly smaller.
    pub context_budget_scale_percent: u32,
}
