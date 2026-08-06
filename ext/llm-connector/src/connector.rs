// `ConnectorImpl`/`Guest` implementation: builds the chat-completions
// request and opens the SSE stream, split out of `lib.rs` to stay under
// the 300-line file limit.
use crate::event_stream::EventStreamImpl;
use crate::exports;
use crate::radcomp::connector::types as conn_types;
use crate::serialize_types::{
    ChatCompletionsRequest, FunctionDefinitionSerialize, MessageSerialize, StreamOptionsSerialize,
    ToolCallFunctionSerialize, ToolCallSerialize, ToolSerialize,
};
use std::cell::RefCell;
use std::collections::VecDeque;

pub(crate) struct ConnectorImpl;

impl exports::radcomp::connector::producer::Guest for ConnectorImpl {
    type EventStream = EventStreamImpl;

    fn generate_stream(
        model: String,
        base_url: Option<String>,
        api_key: Option<String>,
        dialect_name: Option<String>,
        messages: Vec<conn_types::Message>,
        tools: Vec<conn_types::Tool>,
    ) -> Result<exports::radcomp::connector::producer::EventStream, String> {
        let dialect = crate::dialect::resolve(dialect_name.as_deref());
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

        // Azure puts the deployment name in the URL path, so the model string
        // is needed after it has moved into the request body.
        let req_model = model.clone();
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
        let headers = dialect.headers(api_key.as_deref());

        // RAD_TEST_PORT is test infrastructure (redirects every call to a
        // local mock server) and stays env-var-based deliberately — it's
        // not part of the real base-url/api-key resolution the host now
        // does per call.
        let url = if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
            format!("http://127.0.0.1:{test_port}/v1/chat/completions")
        } else if let Some(base_url) = base_url.filter(|b| !b.trim().is_empty()) {
            dialect.url(&rad_models::normalize_base_url(&base_url), &req_model)
        } else if api_key.is_some() {
            dialect.url("https://api.openai.com", &req_model)
        } else {
            return Err(
                "No LLM endpoint configured. Set one up with /llm add <name> <url>.".to_string(),
            );
        };

        eprintln!("[llm-connector] Connecting to endpoint: {url}");
        let stream_handle = crate::open_http_stream(&url, &headers, &body)
            .map_err(|e| format!("open_http_stream to {url} failed: {e}"))?;

        let stream_impl = EventStreamImpl {
            dialect,
            stream_handle,
            buffer: RefCell::new(String::new()),
            event_queue: RefCell::new(VecDeque::new()),
            done: RefCell::new(false),
        };

        let stream_exp = exports::radcomp::connector::producer::EventStream::new(stream_impl);
        Ok(stream_exp)
    }
}
