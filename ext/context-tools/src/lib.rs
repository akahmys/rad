#![deny(clippy::pedantic)]

#[allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::pedantic
)]
mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/context-tools.wit",
        world: "context-tools-extension",
    });

    use super::MyContextTools;
    export!(MyContextTools);
}

use self::bindings::exports::radcomp::context_tools::context_tools::{
    Guest, Message, OptimizationRequest, OptimizationResponse,
};
use self::bindings::radcomp::context_tools::host_rpc;
use self::bindings::radcomp::context_tools::types::RasRpcCommand;

struct MyContextTools;

impl MyContextTools {
    /// Count-based windowing: once the message list exceeds `max_history`,
    /// keep the first message (the original goal) plus the most recent
    /// messages, dropping everything in between. `None` disables windowing.
    fn apply_history_window(
        messages: Vec<Message>,
        max_history: Option<u32>,
        summary_parts: &mut Vec<String>,
    ) -> Vec<Message> {
        let Some(max_history) = max_history else {
            return messages;
        };
        let max_history = usize::try_from(max_history).unwrap_or(usize::MAX);

        if messages.len() <= max_history {
            return messages;
        }

        let original_len = messages.len();
        let first_goal = messages[0].clone();
        let remaining_len = messages.len() - 1;
        let limit = max_history.saturating_sub(1);
        let start_idx = if remaining_len > limit {
            messages.len() - limit
        } else {
            1
        };

        let mut trimmed = vec![first_goal];
        trimmed.extend(messages[start_idx..].iter().cloned());

        summary_parts.push(format!(
            "Windowed history from {original_len} to {} messages (kept first + most recent).",
            trimmed.len()
        ));

        trimmed
    }
}

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
        // than left in as dead/unsafe code. Count-based windowing below is
        // the only compaction strategy.
        if request.messages.is_empty() {
            return Ok(OptimizationResponse {
                optimized_messages: Vec::new(),
                summary: "Empty request.".to_string(),
            });
        }

        let mut summary_parts = Vec::new();

        let optimized_messages =
            Self::apply_history_window(request.messages, request.max_history, &mut summary_parts);

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
        // We use 'tree' command to get the directory structure.
        // We'll use -L 2 to keep it concise for the LLM.
        host_rpc::call(&RasRpcCommand::Command("tree -L 2".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use self::bindings::exports::radcomp::context_tools::context_tools::Message;

    fn msg(id: &str, role: &str, content: &str) -> Message {
        Message {
            node_id: Some(id.to_string()),
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[test]
    fn test_optimize_no_compression() {
        let request = OptimizationRequest {
            messages: vec![msg("1", "user", "Hello"), msg("2", "assistant", "Hi!")],
            max_history: None,
        };
        let result = MyContextTools::optimize(request).unwrap();
        assert_eq!(result.optimized_messages.len(), 2);
        assert_eq!(result.summary, "No messages were compressed.");
    }

    #[test]
    fn test_optimize_does_not_squash_parallel_tool_results() {
        // An assistant turn with 2 parallel tool_calls produces 2 consecutive
        // `tool` messages. Both must survive intact: dropping either would
        // leave the assistant message's tool_calls array referencing a
        // tool_call_id with no matching reply, which real LLM APIs reject.
        let request = OptimizationRequest {
            messages: vec![
                msg("1", "user", "Hello"),
                msg("2", "tool", "First tool result"),
                msg("3", "tool", "Second tool result"),
                msg("4", "assistant", "I got it."),
            ],
            max_history: None,
        };
        let result = MyContextTools::optimize(request).unwrap();
        assert_eq!(result.optimized_messages.len(), 4);
        assert_eq!(result.optimized_messages[1].content, "First tool result");
        assert_eq!(result.optimized_messages[2].content, "Second tool result");
        assert_eq!(result.summary, "No messages were compressed.");
    }

    #[test]
    fn test_optimize_windowing_only() {
        // 1 goal + 9 subsequent user/assistant turns = 10 messages, capped to 5.
        let mut messages = vec![msg("0", "user", "goal")];
        for i in 1..10 {
            let role = if i % 2 == 0 { "assistant" } else { "user" };
            messages.push(msg(&i.to_string(), role, "turn"));
        }
        let request = OptimizationRequest {
            messages,
            max_history: Some(5),
        };
        let result = MyContextTools::optimize(request).unwrap();
        // First (goal) + most recent 4 = 5 messages retained.
        assert_eq!(result.optimized_messages.len(), 5);
        assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
        assert_eq!(result.optimized_messages[1].node_id.as_deref(), Some("6"));
        assert_eq!(result.optimized_messages[4].node_id.as_deref(), Some("9"));
        assert!(result.summary.contains("Windowed history from 10 to 5"));
    }

    #[test]
    fn test_optimize_windowing_under_limit_is_noop() {
        let request = OptimizationRequest {
            messages: vec![msg("1", "user", "Hello"), msg("2", "assistant", "Hi!")],
            max_history: Some(10),
        };
        let result = MyContextTools::optimize(request).unwrap();
        assert_eq!(result.optimized_messages.len(), 2);
        assert_eq!(result.summary, "No messages were compressed.");
    }

    #[test]
    fn test_optimize_windowing_preserves_tool_pairs_in_window() {
        // Goal + a turn with 2 parallel tool_calls; windowing trims the
        // list but must not further collapse the surviving tool pair.
        let messages = vec![
            msg("0", "user", "goal"),
            msg("1", "tool", "old result"),
            msg("2", "assistant", "old reply"),
            msg("3", "tool", "result A"),
            msg("4", "tool", "result B"),
            msg("5", "assistant", "final reply"),
        ];
        let request = OptimizationRequest {
            messages,
            max_history: Some(4),
        };
        let result = MyContextTools::optimize(request).unwrap();
        // Window keeps [goal, result A, result B, final reply] (4 messages);
        // both tool results in the surviving pair remain, unsquashed.
        assert_eq!(result.optimized_messages.len(), 4);
        assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
        assert_eq!(result.optimized_messages[1].content, "result A");
        assert_eq!(result.optimized_messages[2].content, "result B");
        assert_eq!(result.optimized_messages[3].content, "final reply");
        assert!(result.summary.contains("Windowed history from 6 to 4"));
        assert!(!result.summary.contains("Compressed"));
    }
}
