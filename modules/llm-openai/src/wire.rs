//! The OpenAI-compatible chat-completions wire types.
//!
//! Ported from `ext/llm-connector/src/serialize_types.rs`, with one change that
//! is inherent to the port rather than chosen: these types now `Deserialize`
//! too, and that deletes a whole layer.
//!
//! The extension had two parallel sets — a WIT record and a serde struct — and
//! `connector.rs` spent 40 lines converting one into the other, field by field,
//! on every request. The host had a third set (`RemoteMessage`, `RemoteTool` in
//! `src/wasm/rpc_meta_llm_connector.rs`) that it parsed the caller's JSON into
//! purely to fill the WIT record. All three described the same `OpenAI` wire
//! shape. A module receives JSON, so the wire shape can be the *only* shape:
//! what the host holds is what the module deserializes is what goes back out on
//! the request body.
//!
//! The same collapse `context` saw in AWU 956, where `Message` stopped being a
//! WIT record and became a plain serde struct.

/// A tool's `parameters` is a JSON object here, not a string containing JSON.
/// The WIT boundary had no way to carry a `Value`, so the host stringified it
/// and the extension immediately parsed it back — a double encode that existed
/// only to cross a boundary that is now gone.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub description: Option<String>,
    pub parameters: serde_json::Value,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolCallFunction,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(serde::Serialize)]
pub struct ChatCompletionsRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    pub stream_options: Option<StreamOptions>,
    pub tools: Option<Vec<Tool>>,
}

#[derive(serde::Serialize)]
pub struct StreamOptions {
    pub include_usage: bool,
}
