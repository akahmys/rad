// SSE event-stream parsing and the `GuestEventStream` implementation,
// split out of `lib.rs` to stay under the 300-line file limit.
use crate::exports::radcomp::connector::producer::GuestEventStream;
use crate::radcomp::connector::types as conn_types;
use std::cell::RefCell;
use std::collections::VecDeque;

pub(crate) struct EventStreamImpl {
    pub(crate) stream_handle: conn_types::StreamHandle,
    pub(crate) buffer: RefCell<String>,
    pub(crate) event_queue: RefCell<VecDeque<conn_types::LlmEvent>>,
    pub(crate) done: RefCell<bool>,
}

impl EventStreamImpl {
    fn parse_sse_buffer(&self) {
        let mut buf = self.buffer.borrow_mut();
        let mut queue = self.event_queue.borrow_mut();

        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            *buf = buf[pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            if line.starts_with("data:") {
                let data_str = line["data:".len()..].trim();
                if data_str == "[DONE]" {
                    *self.done.borrow_mut() = true;
                    break;
                }

                if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                    // 1. Content and Reasoning Chunks
                    if let Some(reasoning) = val
                        .pointer("/choices/0/delta/reasoning_content")
                        .and_then(serde_json::Value::as_str)
                    {
                        queue
                            .push_back(conn_types::LlmEvent::ReasoningChunk(reasoning.to_string()));
                    } else if let Some(content) = val
                        .pointer("/choices/0/delta/content")
                        .and_then(serde_json::Value::as_str)
                    {
                        queue.push_back(conn_types::LlmEvent::ContentChunk(content.to_string()));
                    }

                    // 2. Tool Calls
                    if let Some(tool_calls) = val
                        .pointer("/choices/0/delta/tool_calls")
                        .and_then(serde_json::Value::as_array)
                    {
                        for tc in tool_calls {
                            let index = tc
                                .get("index")
                                .and_then(serde_json::Value::as_u64)
                                .unwrap_or(0) as u32;
                            let id = tc
                                .get("id")
                                .and_then(serde_json::Value::as_str)
                                .map(String::from);
                            let name = tc
                                .pointer("/function/name")
                                .and_then(serde_json::Value::as_str)
                                .map(String::from);
                            let arguments_chunk = tc
                                .pointer("/function/arguments")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("")
                                .to_string();

                            queue.push_back(conn_types::LlmEvent::ToolCallChunk(
                                conn_types::ToolCallChunk {
                                    index,
                                    id,
                                    name,
                                    arguments_chunk,
                                },
                            ));
                        }
                    }

                    // 3. Usage Info
                    if let Some(usage) = val.get("usage") {
                        let prompt_tokens = usage
                            .get("prompt_tokens")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0) as u32;
                        let completion_tokens = usage
                            .get("completion_tokens")
                            .and_then(serde_json::Value::as_u64)
                            .unwrap_or(0) as u32;
                        if prompt_tokens > 0 || completion_tokens > 0 {
                            queue.push_back(conn_types::LlmEvent::CompletionComplete(
                                conn_types::CompletionUsage {
                                    prompt_tokens,
                                    completion_tokens,
                                },
                            ));
                        }
                    }
                }
            }
        }
    }
}

impl GuestEventStream for EventStreamImpl {
    fn read(&self) -> Result<Option<conn_types::LlmEvent>, String> {
        loop {
            // If we have parsed events ready, return the first one
            if let Some(event) = self.event_queue.borrow_mut().pop_front() {
                return Ok(Some(event));
            }

            // If we've completed stream parsing, we are done
            if *self.done.borrow() {
                return Ok(None);
            }

            // Read next chunk of data from host HTTP stream
            match self.stream_handle.read(4096) {
                Ok(chunk) => {
                    if chunk.is_empty() {
                        *self.done.borrow_mut() = true;
                        // Parse any remaining line fragments
                        self.parse_sse_buffer();
                        if let Some(event) = self.event_queue.borrow_mut().pop_front() {
                            return Ok(Some(event));
                        }
                        return Ok(None);
                    }
                    if let Ok(text) = String::from_utf8(chunk) {
                        self.buffer.borrow_mut().push_str(&text);
                        self.parse_sse_buffer();
                    } else {
                        return Err("Invalid UTF-8 chunk received".to_string());
                    }
                }
                Err(e) => {
                    return Err(format!("HTTP stream read error: {e}"));
                }
            }
        }
    }
}
