#![deny(clippy::pedantic)]

#[allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::pedantic
)]
mod bindings {
    // `path` points at the directory, not a single file: context-tools.wit
    // shares `package radcomp:extension` with rad.wit (same package split
    // across two files in the same dir) so it can reuse rad.wit's `types`
    // interface (the real `ras-rpc-command`) instead of carrying its own
    // bespoke one. See AWU 915's follow-up / wit/context-tools.wit's docs.
    wit_bindgen::generate!({
        path: "../../wit",
        world: "context-tools-extension",
    });

    use super::MyContextTools;
    export!(MyContextTools);
}

use self::bindings::exports::radcomp::extension::context_tools::{
    Guest, OptimizationRequest, OptimizationResponse,
};
use self::bindings::host_rpc;
use self::bindings::radcomp::extension::types::RasRpcCommand;

mod windowing;

#[cfg(test)]
mod tests;

struct MyContextTools;

impl Guest for MyContextTools {
    fn optimize(request: OptimizationRequest) -> Result<OptimizationResponse, String> {
        // NOTE: this used to also role-squash consecutive runs of
        // non-user/assistant messages down to the last one in the run. In
        // this system the only role that is ever valid outside `user` /
        // `assistant` / `system` is `tool` (see llm.rs's DAG walk), and a
        // single `assistant` turn can carry multiple parallel `tool_calls`,
        // each answered by its own consecutive `tool` message. Squashing
        // those down to one silently drops tool results that the
        // `assistant` message's `tool_calls` array still references,
        // producing a request the LLM API will reject. There is no role in
        // this system for which squashing is safe, so it was removed rather
        // than left in as dead/unsafe code. What remains below is stale
        // tool-result clearing followed by count- and size-bounded
        // windowing (with relevance-based retention).
        if request.messages.is_empty() {
            return Ok(OptimizationResponse {
                optimized_messages: Vec::new(),
                summary: "Empty request.".to_string(),
            });
        }

        let mut summary_parts = Vec::new();

        let cleared_messages = windowing::clear_stale_tool_results(
            request.messages,
            request.max_content_chars,
            &mut summary_parts,
        );

        let optimized_messages = windowing::apply_history_window(
            cleared_messages,
            request.max_history,
            request.max_content_chars,
            &mut summary_parts,
        );

        let summary = if summary_parts.is_empty() {
            "No messages were compressed.".to_string()
        } else {
            summary_parts.join(" ")
        };

        Ok(OptimizationResponse {
            optimized_messages,
            summary,
        })
    }

    fn get_repo_map() -> Result<String, String> {
        // Now that this extension shares the real `ras-rpc-command` type
        // (see module docs), delegate to the same semantic (tree-sitter
        // based) repo map every other extension gets via `GetRepoMap`,
        // instead of shelling out to `tree -L 2` through a bespoke
        // raw-shell command that bypassed permission checks entirely.
        host_rpc(&RasRpcCommand::GetRepoMap)
    }
}
