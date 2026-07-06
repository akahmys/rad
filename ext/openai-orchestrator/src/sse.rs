use crate::types::OrchestratorState;
use crate::call_host;
use crate::types::RasRpcCommand;

pub fn handle_stream_delta(state: &mut OrchestratorState, val: &serde_json::Value) {
    if let Some(content) = val.pointer("/choices/0/delta/content").and_then(serde_json::Value::as_str) {
        let _ = call_host(RasRpcCommand::WriteStdout { text: content.to_string() });
        state.assistant.push_str(content);
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
