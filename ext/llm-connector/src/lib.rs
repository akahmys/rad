#![deny(clippy::pedantic)]
#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::needless_pass_by_value,
    clippy::same_length_and_capacity,
    clippy::collapsible_if,
    clippy::uninlined_format_args,
    clippy::cast_possible_truncation,
    clippy::manual_strip
)]

wit_bindgen::generate!({
    path: "../../wit/llm-connector.wit",
    world: "llm-connector",
});

use crate::exports::radcomp::connector::producer::GuestEventStream;
use crate::radcomp::connector::types as conn_types;
use std::cell::RefCell;
use std::collections::VecDeque;

#[derive(serde::Serialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<MessageSerialize>,
    stream: bool,
    stream_options: Option<StreamOptionsSerialize>,
    tools: Option<Vec<ToolSerialize>>,
}

#[derive(serde::Serialize)]
struct MessageSerialize {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCallSerialize>>,
}

#[derive(serde::Serialize)]
struct ToolCallSerialize {
    id: String,
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolCallFunctionSerialize,
}

#[derive(serde::Serialize)]
struct ToolCallFunctionSerialize {
    name: String,
    arguments: String,
}

#[derive(serde::Serialize)]
struct StreamOptionsSerialize {
    include_usage: bool,
}

#[derive(serde::Serialize)]
struct ToolSerialize {
    #[serde(rename = "type")]
    tool_type: String,
    function: FunctionDefinitionSerialize,
}

#[derive(serde::Serialize)]
struct FunctionDefinitionSerialize {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    parameters: serde_json::Value,
}

pub struct EventStreamImpl {
    stream_handle: conn_types::StreamHandle,
    buffer: RefCell<String>,
    event_queue: RefCell<VecDeque<conn_types::LlmEvent>>,
    done: RefCell<bool>,
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

struct ConnectorImpl;

impl exports::radcomp::connector::producer::Guest for ConnectorImpl {
    type EventStream = EventStreamImpl;

    fn generate_stream(
        model: String,
        messages: Vec<conn_types::Message>,
        tools: Vec<conn_types::Tool>,
    ) -> Result<exports::radcomp::connector::producer::EventStream, String> {
        let messages_serialize: Vec<MessageSerialize> = messages
            .into_iter()
            .map(|m| MessageSerialize {
                role: m.role,
                content: m.content,
                name: m.name,
                tool_call_id: m.tool_call_id,
                tool_calls: m.tool_calls.map(|calls| {
                    calls
                        .into_iter()
                        .map(|c| ToolCallSerialize {
                            id: c.id,
                            tool_type: c.tool_type,
                            function: ToolCallFunctionSerialize {
                                name: c.function.name,
                                arguments: c.function.arguments,
                            },
                        })
                        .collect()
                }),
            })
            .collect();

        let tools_serialize: Option<Vec<ToolSerialize>> = if tools.is_empty() {
            None
        } else {
            Some(
                tools
                    .into_iter()
                    .map(|t| {
                        let parameters: serde_json::Value =
                            serde_json::from_str(&t.function.parameters)
                                .unwrap_or(serde_json::Value::Null);
                        ToolSerialize {
                            tool_type: t.tool_type,
                            function: FunctionDefinitionSerialize {
                                name: t.function.name,
                                description: t.function.description,
                                parameters,
                            },
                        }
                    })
                    .collect(),
            )
        };

        let req = ChatCompletionsRequest {
            model,
            messages: messages_serialize,
            stream: true,
            stream_options: Some(StreamOptionsSerialize {
                include_usage: true,
            }),
            tools: tools_serialize,
        };

        let body = serde_json::to_string(&req).map_err(|e| format!("JSON serialize error: {e}"))?;
        let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        let api_key = std::env::var("LLM_API_KEY")
            .or_else(|_| std::env::var("RAD_API_KEY"))
            .or_else(|_| std::env::var("OPENAI_API_KEY"))
            .ok();

        if let Some(ref key) = api_key {
            if !key.trim().is_empty() {
                headers.push(("Authorization".to_string(), format!("Bearer {}", key.trim())));
            }
        }

        let base_url_env = std::env::var("LLM_BASE_URL")
            .or_else(|_| std::env::var("RAD_BASE_URL"))
            .or_else(|_| std::env::var("OPENAI_BASE_URL"))
            .ok();

        let url = if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
            format!("http://127.0.0.1:{}/v1/chat/completions", test_port)
        } else if let Some(base_url) = base_url_env {
            let trimmed = base_url.trim().trim_end_matches('/');
            if trimmed.ends_with("/chat/completions") {
                trimmed.to_string()
            } else if trimmed.ends_with("/v1") {
                format!("{trimmed}/chat/completions")
            } else {
                format!("{trimmed}/v1/chat/completions")
            }
        } else if api_key.is_some() {
            "https://api.openai.com/v1/chat/completions".to_string()
        } else {
            return Err("No LLM endpoint configured. Please set LLM_BASE_URL (or RAD_BASE_URL / OPENAI_BASE_URL) or API_KEY.".to_string());
        };

        eprintln!("[llm-connector] Connecting to endpoint: {url}");
        let stream_handle = open_http_stream(&url, &headers, &body)
            .map_err(|e| format!("open_http_stream to {url} failed: {e}"))?;

        let stream_impl = EventStreamImpl {
            stream_handle,
            buffer: RefCell::new(String::new()),
            event_queue: RefCell::new(VecDeque::new()),
            done: RefCell::new(false),
        };

        let stream_exp = exports::radcomp::connector::producer::EventStream::new(stream_impl);
        Ok(stream_exp)
    }
}

export!(ConnectorImpl);
