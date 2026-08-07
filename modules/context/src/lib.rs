//! Context compaction as a kernel module.
//!
//! Ported from the `context-tools` extension, which is still live and serving
//! until AWU 957 switches the caller across — §9.4 requires rad to work at the
//! end of every AWU, not only at the end of a stage, so the logic is duplicated
//! for exactly two AWUs rather than moved.
//!
//! The compaction itself is unchanged. What changed is the boundary: the same
//! records that were WIT `record`s are now plain serde structs, because
//! dispatch is opaque and the kernel never looks inside a payload. Adding a
//! field here breaks nothing and requires no rebuild of anything else — which
//! is the entire point of the migration, visible in one file.
#![deny(clippy::pedantic)]

mod windowing;

#[cfg(test)]
mod tests;

use rad_sdk::Error;

/// One conversation message. Was a WIT `record`; now serde.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Message {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub role: String,
    pub content: String,
}

#[derive(serde::Deserialize)]
pub struct OptimizeReq {
    pub messages: Vec<Message>,
    /// Maximum messages to retain: the first (the original goal) plus the most
    /// recent. `None` disables count-based windowing.
    #[serde(default)]
    pub max_history: Option<u32>,
    /// Maximum total content characters across the kept messages. Message count
    /// alone is a poor proxy for how much of a small local context window a
    /// handful of large tool outputs consumes. When both are set, whichever
    /// windows more aggressively wins. `None` disables size-based windowing.
    #[serde(default)]
    pub max_content_chars: Option<u32>,
}

#[derive(serde::Serialize)]
pub struct OptimizeRes {
    pub optimized_messages: Vec<Message>,
    pub summary: String,
}

/// Kept structurally identical to the extension's implementation, including the
/// exact summary strings: AWU 957 swaps the caller over, and a difference here
/// would surface as a behaviour change attributed to the migration rather than
/// to anything anyone chose.
fn optimize(req: OptimizeReq) -> Result<OptimizeRes, Error> {
    let OptimizeReq {
        messages,
        max_history,
        max_content_chars,
    } = req;

    // A zero budget is a caller bug, not a request to discard everything.
    // Silently returning an empty history would surface much later as a model
    // that has forgotten the conversation, with nothing pointing back here.
    if max_content_chars == Some(0) || max_history == Some(0) {
        return Err(Error::invalid(
            "max_content_chars and max_history must be greater than zero when set",
        ));
    }

    if messages.is_empty() {
        return Ok(OptimizeRes {
            optimized_messages: Vec::new(),
            summary: "Empty request.".to_string(),
        });
    }

    let mut summary_parts = Vec::new();

    let cleared =
        windowing::clear_stale_tool_results(messages, max_content_chars, &mut summary_parts);
    let optimized_messages = windowing::apply_history_window(
        cleared,
        max_history,
        max_content_chars,
        &mut summary_parts,
    );

    let summary = if summary_parts.is_empty() {
        "No messages were compressed.".to_string()
    } else {
        summary_parts.join(" ")
    };

    Ok(OptimizeRes {
        optimized_messages,
        summary,
    })
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "context",
    version: "0.1.0",
    methods: {
        "context.optimize" => optimize,
    }
}
