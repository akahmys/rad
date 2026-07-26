// `GenerateLlmStream` handling via a raw HTTP SSE stream, split out of
// `rpc_meta.rs` to stay under the 300-line file limit. Used when no
// orchestrator (and therefore no Wasm `llm-connector` runtime) is present.
use crate::wasm::rpc::RpcContext;

pub(crate) fn generate(ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = ctx.network.open_http_stream(
        "http://127.0.0.1/v1/chat/completions",
        std::collections::HashMap::new(),
        "",
        tx,
        ctx.llm_timeout_policy.clone(),
    )?;

    let event_tx_clone = ctx.event_tx.clone();
    std::thread::spawn(move || {
        let mut buffer = String::new();
        while let Ok(event) = rx.recv() {
            if let crate::ipc::RasCoreEvent::HttpChunkReceived { chunk } = event {
                buffer.push_str(&chunk);
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();
                    if line.is_empty() {
                        continue;
                    }
                    if let Some(stripped) = line.strip_prefix("data:") {
                        handle_sse_data_line(stripped.trim(), &event_tx_clone);
                        if stripped.trim() == "[DONE]" {
                            break;
                        }
                    }
                }
            }
        }
    });
    Ok(serde_json::Value::Null)
}

fn handle_sse_data_line(
    data_str: &str,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
) {
    if data_str == "[DONE]" {
        let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
            event: serde_json::json!({ "type": "done" }).to_string(),
        });
        return;
    }

    let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) else {
        return;
    };

    if let Some(reasoning) = val
        .pointer("/choices/0/delta/reasoning_content")
        .and_then(serde_json::Value::as_str)
    {
        let ev = serde_json::json!({ "ReasoningChunk": reasoning });
        let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
            event: ev.to_string(),
        });
    } else if let Some(content) = val
        .pointer("/choices/0/delta/content")
        .and_then(serde_json::Value::as_str)
    {
        let ev = serde_json::json!({ "ContentChunk": content });
        let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
            event: ev.to_string(),
        });
    }

    if let Some(tool_calls) = val
        .pointer("/choices/0/delta/tool_calls")
        .and_then(serde_json::Value::as_array)
    {
        for tc in tool_calls {
            let index = tc.get("index").and_then(serde_json::Value::as_u64).unwrap_or(0);
            let id = tc.get("id").and_then(serde_json::Value::as_str);
            let name = tc.pointer("/function/name").and_then(serde_json::Value::as_str);
            let arguments_chunk = tc
                .pointer("/function/arguments")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let ev = serde_json::json!({
                "ToolCallChunk": {
                    "index": index,
                    "id": id,
                    "name": name,
                    "arguments-chunk": arguments_chunk,
                }
            });
            let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                event: ev.to_string(),
            });
        }
    }

    if let Some(usage) = val.get("usage") {
        let prompt_tokens = usage
            .get("prompt_tokens")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let completion_tokens = usage
            .get("completion_tokens")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if prompt_tokens > 0 || completion_tokens > 0 {
            let ev = serde_json::json!({
                "CompletionComplete": {
                    "prompt-tokens": prompt_tokens,
                    "completion-tokens": completion_tokens,
                }
            });
            let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                event: ev.to_string(),
            });
        }
    }
}
