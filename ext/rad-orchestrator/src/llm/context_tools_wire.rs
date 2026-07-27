//! JSON wire structs for the `context-tools.optimize` `CallExtension`
//! round-trip, split out of `llm.rs` to stay under the 300-line file
//! limit.
//!
//! Field names here must match the *Rust* field names wasmtime's
//! `additional_derives: [serde::Serialize, serde::Deserialize]` produces on
//! the host's generated `OptimizationRequest`/`Message`/`OptimizationResponse`
//! structs (`src/wasm/bindings.rs`'s `rad_context_tools` module) — i.e. plain
//! `snake_case` (`node_id`, `max_history`, ...), NOT the WIT source's
//! kebab-case spelling. These structs previously used
//! `#[serde(rename = "node-id")]`-style kebab-case renames on the assumption
//! the host serialized kebab-case; it doesn't, so every field silently
//! deserialized as its default (`None`/missing) on the host side and
//! `optimize` never actually windowed anything — see AWU 915's follow-up,
//! caught by `tests/context_tools_tests.rs` exercising this round-trip
//! through the real WASM component boundary for the first time.
#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct CtMessage {
    pub(super) node_id: Option<String>,
    pub(super) role: String,
    pub(super) content: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct CtOptimizationRequest {
    pub(super) messages: Vec<CtMessage>,
    pub(super) max_history: Option<u32>,
    pub(super) max_content_chars: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct CtOptimizationResponse {
    pub(super) optimized_messages: Vec<CtMessage>,
    pub(super) summary: String,
}
