use crate::types::OrchestratorState;
use crate::call_host;
use crate::types::RasRpcCommand;

pub fn handle_stream_delta(state: &mut OrchestratorState, val: &serde_json::Value) {
    let mut reasoning_part = None;
    if let Some(reasoning) = val.pointer("/choices/0/delta/reasoning_content").and_then(serde_json::Value::as_str) {
        reasoning_part = Some(reasoning.to_string());
    }

    if let Some(reasoning) = reasoning_part {
        if !state.is_reasoning {
            let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string() });
            state.is_reasoning = true;
        }
        let _ = call_host(RasRpcCommand::WriteStdout { text: format!("\x1b[2m{}\x1b[0m", reasoning) });
        state.reasoning_buffered.push_str(&reasoning);
    } else if let Some(content) = val.pointer("/choices/0/delta/content").and_then(serde_json::Value::as_str) {
        handle_content_token(state, content);
    }
    if let Some(tool_calls) = val.pointer("/choices/0/delta/tool_calls").and_then(serde_json::Value::as_array) {
        for tc in tool_calls {
            if let Some(index) = tc.get("index").and_then(serde_json::Value::as_u64).and_then(|i| usize::try_from(i).ok()) {
                let entry = state.tool_calls.entry(index).or_default();
                if let Some(id) = tc.get("id").and_then(serde_json::Value::as_str) {
                    entry.id.push_str(id);
                }
                if let Some(func) = tc.get("function").and_then(serde_json::Value::as_object) {
                    if let Some(name) = func.get("name").and_then(serde_json::Value::as_str) {
                        entry.name.push_str(name);
                    }
                    if let Some(args) = func.get("arguments").and_then(serde_json::Value::as_str) {
                        entry.arguments.push_str(args);
                    }
                }
            }
        }
    }
    if let Some(usage) = val.get("usage") {
        let prompt_tokens = usage.get("prompt_tokens").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
        let completion_tokens = usage.get("completion_tokens").and_then(serde_json::Value::as_u64).unwrap_or(0) as u32;
        if prompt_tokens > 0 || completion_tokens > 0 {
            let _ = call_host(RasRpcCommand::ReportTokenUsage { prompt_tokens, completion_tokens });
        }
    }
}

pub fn process_sse_buffer(state: &mut OrchestratorState) -> Result<bool, String> {
    let mut done = false;
    while let Some(pos) = state.stream.find('\n') {
        let line = state.stream[..pos].trim().to_string();
        state.stream = state.stream[pos + 1..].to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("data:") {
            let data_str = line["data:".len()..].trim();
            if data_str == "[DONE]" {
                done = true;
                break;
            }
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                handle_stream_delta(state, &val);
            }
        }
    }
    Ok(done)
}

fn handle_content_token(state: &mut OrchestratorState, content: &str) {
    if state.is_reasoning && !content.contains("<thought>") && !content.contains("</thought>") {
        let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string() });
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
            if state.is_reasoning {
                let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string() });
            }
            let _ = call_host(RasRpcCommand::WriteStdout { text: before.to_string() });
            state.assistant.push_str(before);
        }
        let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thinking]\x1b[0m\n".to_string() });
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
                let _ = call_host(RasRpcCommand::WriteStdout { text: format!("\x1b[2m{}\x1b[0m", thought_content) });
                state.reasoning_buffered.push_str(thought_content);
            }
            let _ = call_host(RasRpcCommand::WriteStdout { text: "\n\x1b[2m[Thought End]\x1b[0m\n\n".to_string() });
            state.is_reasoning = false;
            let after = &text[pos + "</thought>".len()..];
            if !after.is_empty() {
                let _ = call_host(RasRpcCommand::WriteStdout { text: after.to_string() });
                state.assistant.push_str(after);
            }
        }
    } else {
        let _ = call_host(RasRpcCommand::WriteStdout { text: format!("\x1b[2m{}\x1b[0m", text) });
        state.reasoning_buffered.push_str(text);
    }
}

