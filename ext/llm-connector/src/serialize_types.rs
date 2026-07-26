// OpenAI-compatible chat-completions request wire types, split out of
// `lib.rs` to stay under the 300-line file limit.
#[derive(serde::Serialize)]
pub(crate) struct ChatCompletionsRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<MessageSerialize>,
    pub(crate) stream: bool,
    pub(crate) stream_options: Option<StreamOptionsSerialize>,
    pub(crate) tools: Option<Vec<ToolSerialize>>,
}

#[derive(serde::Serialize)]
pub(crate) struct MessageSerialize {
    pub(crate) role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_calls: Option<Vec<ToolCallSerialize>>,
}

#[derive(serde::Serialize)]
pub(crate) struct ToolCallSerialize {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) tool_type: String,
    pub(crate) function: ToolCallFunctionSerialize,
}

#[derive(serde::Serialize)]
pub(crate) struct ToolCallFunctionSerialize {
    pub(crate) name: String,
    pub(crate) arguments: String,
}

#[derive(serde::Serialize)]
pub(crate) struct StreamOptionsSerialize {
    pub(crate) include_usage: bool,
}

#[derive(serde::Serialize)]
pub(crate) struct ToolSerialize {
    #[serde(rename = "type")]
    pub(crate) tool_type: String,
    pub(crate) function: FunctionDefinitionSerialize,
}

#[derive(serde::Serialize)]
pub(crate) struct FunctionDefinitionSerialize {
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) description: Option<String>,
    pub(crate) parameters: serde_json::Value,
}
