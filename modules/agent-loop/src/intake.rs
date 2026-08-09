//! The wire shape `llm-openai` emits, and the turn state it accumulates into.
//!
//! `RawEvent` is copied field for field from
//! `ext/rad-orchestrator/src/orchestrator/reasoning.rs` rather than redesigned.
//! It is the contract with the transport, which is still producing exactly
//! these bytes (AWU 967 pinned them in a unit test because the consumer was an
//! extension). Changing the shape and the consumer in one step would leave no
//! way to tell which of the two broke a turn.
use std::cell::RefCell;

#[derive(serde::Deserialize)]
pub(crate) struct ToolCallChunkEvent {
    pub(crate) index: u32,
    pub(crate) id: Option<String>,
    pub(crate) name: Option<String>,
    #[serde(alias = "arguments-chunk")]
    pub(crate) arguments_chunk: String,
}

#[derive(serde::Deserialize)]
pub(crate) struct CompletionUsageEvent {
    #[serde(alias = "prompt-tokens")]
    pub(crate) prompt_tokens: u32,
    #[serde(alias = "completion-tokens")]
    pub(crate) completion_tokens: u32,
}

#[derive(serde::Deserialize)]
pub(crate) struct RawEvent {
    #[serde(rename = "type")]
    pub(crate) event_type: Option<String>,
    pub(crate) payload: Option<String>,

    #[serde(rename = "ContentChunk")]
    pub(crate) content_chunk: Option<String>,
    #[serde(rename = "ReasoningChunk")]
    pub(crate) reasoning_chunk: Option<String>,
    #[serde(rename = "ToolCallChunk")]
    pub(crate) tool_call_chunk: Option<ToolCallChunkEvent>,
    #[serde(rename = "CompletionComplete")]
    pub(crate) completion_complete: Option<CompletionUsageEvent>,
    #[serde(rename = "Error")]
    pub(crate) error: Option<String>,
}

/// One tool call, reassembled from the chunks that arrive for it.
#[derive(Default, Clone, serde::Serialize)]
pub(crate) struct PartialToolCall {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) arguments: String,
}

/// What has arrived for the turn in progress.
#[derive(Default, serde::Serialize)]
pub(crate) struct Turn {
    pub(crate) text: String,
    pub(crate) reasoning: String,
    /// Indexed as the transport indexes them: a chunk names its slot, and slots
    /// need not arrive in order, so this is grown to fit rather than pushed to.
    pub(crate) tool_calls: Vec<PartialToolCall>,
    pub(crate) done: bool,
    pub(crate) error: Option<String>,
    pub(crate) prompt_tokens: u32,
    pub(crate) completion_tokens: u32,
}

thread_local! {
    /// One turn at a time. A module's store is entered by one caller at a time
    /// by construction, so this needs no lock and no unsafe `Send` claim —
    /// the shape `modules/llm-openai` and `modules/mcp` both settled on.
    static TURN: RefCell<Turn> = RefCell::new(Turn::default());
}

impl Turn {
    /// Folds one event in. Everything an event can carry is optional and more
    /// than one field may be set, so each is examined rather than matched as an
    /// exclusive variant — the extension read them the same way.
    fn absorb(&mut self, raw: &RawEvent) {
        match raw.event_type.as_deref() {
            Some("done") => self.done = true,
            Some("error") => {
                self.error = Some(raw.payload.clone().unwrap_or_else(|| {
                    // The extension's wording for a payload-less error.
                    "unknown error".to_string()
                }));
            }
            _ => {}
        }
        if let Some(chunk) = &raw.content_chunk {
            self.text.push_str(chunk);
        }
        if let Some(chunk) = &raw.reasoning_chunk {
            self.reasoning.push_str(chunk);
        }
        if let Some(err) = &raw.error {
            self.error = Some(err.clone());
        }
        if let Some(usage) = &raw.completion_complete {
            self.prompt_tokens = usage.prompt_tokens;
            self.completion_tokens = usage.completion_tokens;
        }
        if let Some(call) = &raw.tool_call_chunk {
            self.absorb_tool_call(call);
        }
    }

    fn absorb_tool_call(&mut self, chunk: &ToolCallChunkEvent) {
        let slot = chunk.index as usize;
        if self.tool_calls.len() <= slot {
            self.tool_calls.resize(slot + 1, PartialToolCall::default());
        }
        let call = &mut self.tool_calls[slot];
        // `id` and `name` arrive once, on the first chunk for a slot, and are
        // absent on the rest. Overwriting with a later `None` would erase them.
        if let Some(id) = &chunk.id {
            call.id.clone_from(id);
        }
        if let Some(name) = &chunk.name {
            call.name.clone_from(name);
        }
        call.arguments.push_str(&chunk.arguments_chunk);
    }
}

/// Parses one event and folds it into the turn in progress.
pub(crate) fn absorb(event_json: &str) -> Result<(), String> {
    let raw: RawEvent = serde_json::from_str(event_json)
        .map_err(|e| format!("Failed to parse LlmConnectorEvent JSON: {e} (raw={event_json})"))?;
    TURN.with_borrow_mut(|turn| turn.absorb(&raw));
    Ok(())
}

/// The turn so far, as JSON.
pub(crate) fn snapshot() -> serde_json::Value {
    TURN.with_borrow(|turn| serde_json::to_value(turn).unwrap_or(serde_json::Value::Null))
}

/// Clears the turn. Called when one starts, not when one ends: a finished turn
/// stays readable until the next begins.
pub(crate) fn reset() {
    TURN.with_borrow_mut(|turn| *turn = Turn::default());
}

#[cfg(test)]
mod tests;
