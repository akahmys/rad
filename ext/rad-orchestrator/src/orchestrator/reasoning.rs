use crate::call_host;
use crate::types::{OrchestratorState, RasRpcCommand};

/// Reasoning trace markers ([Thinking...]/[Thought End]) and the dimmed
/// `<thought>` content itself are silent by default; set `RAD_DEBUG=1` to
/// show them alongside the final answer.
pub(crate) fn debug_enabled() -> bool {
    std::env::var("RAD_DEBUG").is_ok()
}

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

pub(crate) fn handle_content_token(state: &mut OrchestratorState, content: &str) {
    if state.is_reasoning && !content.contains("<thought>") && !content.contains("</thought>") {
        if debug_enabled() {
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
            });
        }
        state.is_reasoning = false;
    }

    let mut text = content.to_string();
    if text.contains("<thought>") {
        text = handle_thought_start_tag(state, &text);
    }

    if state.is_reasoning {
        handle_reasoning_text(state, &text);
    } else {
        let _ = call_host(RasRpcCommand::WriteStdout { text: text.clone() });
        state.assistant.push_str(&text);
    }
}

fn handle_thought_start_tag(state: &mut OrchestratorState, text: &str) -> String {
    if let Some(pos) = text.find("<thought>") {
        let before = &text[..pos];
        if !before.is_empty() {
            if state.is_reasoning && debug_enabled() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
                });
            }
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: before.to_string(),
            });
            state.assistant.push_str(before);
        }
        if debug_enabled() {
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string(),
            });
        }
        state.is_reasoning = true;
        return text[pos + "<thought>".len()..].to_string();
    }
    text.to_string()
}

fn handle_reasoning_text(state: &mut OrchestratorState, text: &str) {
    if text.contains("</thought>") {
        if let Some(pos) = text.find("</thought>") {
            let thought_content = &text[..pos];
            if !thought_content.is_empty() {
                if debug_enabled() {
                    let _ = call_host(RasRpcCommand::WriteStdout {
                        text: format!("\x1b[2m{thought_content}\x1b[0m"),
                    });
                }
                state.reasoning_buffered.push_str(thought_content);
            }
            if debug_enabled() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string(),
                });
            }
            state.is_reasoning = false;
            let after = &text[pos + "</thought>".len()..];
            if !after.is_empty() {
                let _ = call_host(RasRpcCommand::WriteStdout {
                    text: after.to_string(),
                });
                state.assistant.push_str(after);
            }
        }
    } else {
        if debug_enabled() {
            let _ = call_host(RasRpcCommand::WriteStdout {
                text: format!("\x1b[2m{text}\x1b[0m"),
            });
        }
        state.reasoning_buffered.push_str(text);
    }
}
