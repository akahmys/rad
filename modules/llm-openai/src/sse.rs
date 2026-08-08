//! SSE parsing — one `data:` line at a time, into the events the agent loop
//! already understands.
//!
//! Ported from `ext/llm-connector/src/event_stream.rs`. The JSON-Pointer lookups
//! and the shape of every event are unchanged, because the consumer is still
//! `rad-orchestrator` until stage 8 and it matches on these names
//! (`ext/rad-orchestrator/src/orchestrator/reasoning.rs`).

/// One event, serialized exactly as the WIT variant it replaces did:
/// externally tagged, `{"ContentChunk": "..."}`. That is what the agent loop
/// deserializes today, and a port is not the place to change it.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub enum LlmEvent {
    ContentChunk(String),
    ReasoningChunk(String),
    ToolCallChunk(ToolCallChunk),
    CompletionComplete(CompletionUsage),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct ToolCallChunk {
    pub index: u32,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments_chunk: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct CompletionUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Accumulates text and yields whole `data:` lines as events.
#[derive(Default)]
pub struct Parser {
    buffer: String,
    /// Set by `data: [DONE]`. The server said it is finished, which is distinct
    /// from the connection closing.
    pub done: bool,
}

impl Parser {
    /// Feeds one decoded chunk in and appends whatever it completes.
    pub fn push(&mut self, text: &str, dialect: &crate::dialect::Dialect, out: &mut Vec<LlmEvent>) {
        self.buffer.push_str(text);
        while let Some(pos) = self.buffer.find('\n') {
            let line = self.buffer[..pos].trim().to_string();
            self.buffer.drain(..=pos);

            let Some(rest) = line.strip_prefix("data:") else {
                continue;
            };
            let data = rest.trim();
            if data == "[DONE]" {
                self.done = true;
                break;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data) {
                parse_frame(&val, dialect, out);
            }
        }
    }
}

/// One SSE frame's JSON, in the order the agent loop expects: text first, then
/// tool calls, then usage. A frame can legitimately carry more than one.
fn parse_frame(
    val: &serde_json::Value,
    dialect: &crate::dialect::Dialect,
    out: &mut Vec<LlmEvent>,
) {
    // Reasoning takes precedence over content, as it did in the extension: a
    // frame carrying `reasoning_content` is a thought, not an answer.
    if let Some(reasoning) = dialect
        .reasoning_ptr
        .and_then(|ptr| val.pointer(ptr))
        .and_then(serde_json::Value::as_str)
    {
        out.push(LlmEvent::ReasoningChunk(reasoning.to_string()));
    } else if let Some(content) = val
        .pointer(dialect.content_ptr)
        .and_then(serde_json::Value::as_str)
    {
        out.push(LlmEvent::ContentChunk(content.to_string()));
    }

    if let Some(tool_calls) = val
        .pointer(dialect.tool_calls_ptr)
        .and_then(serde_json::Value::as_array)
    {
        for tc in tool_calls {
            out.push(LlmEvent::ToolCallChunk(tool_call_chunk(tc)));
        }
    }

    if let Some(usage) = val.get("usage") {
        let prompt_tokens = u64_field(usage, "prompt_tokens");
        let completion_tokens = u64_field(usage, "completion_tokens");
        // A frame with a zeroed usage block is the server padding its output,
        // not a completion worth reporting.
        if prompt_tokens > 0 || completion_tokens > 0 {
            out.push(LlmEvent::CompletionComplete(CompletionUsage {
                prompt_tokens,
                completion_tokens,
            }));
        }
    }
}

fn tool_call_chunk(tc: &serde_json::Value) -> ToolCallChunk {
    ToolCallChunk {
        index: u64_field(tc, "index"),
        id: tc
            .get("id")
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        name: tc
            .pointer("/function/name")
            .and_then(serde_json::Value::as_str)
            .map(String::from),
        arguments_chunk: tc
            .pointer("/function/arguments")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("")
            .to_string(),
    }
}

/// Absent, non-numeric, or too large all read as zero — the extension's
/// behaviour, where every one of these went through `unwrap_or(0)` and a
/// saturating `try_from`.
fn u64_field(val: &serde_json::Value, key: &str) -> u32 {
    u32::try_from(
        val.get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    )
    .unwrap_or(0)
}

#[cfg(test)]
mod tests;
