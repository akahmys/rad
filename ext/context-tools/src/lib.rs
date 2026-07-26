#![deny(clippy::pedantic)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::same_length_and_capacity)]

wit_bindgen::generate!({
    path: "../../wit/context-tools.wit",
    world: "context-tools-extension",
});

use crate::exports::radcomp::context_tools::context_tools::{
    Guest, Message, OptimizationRequest, OptimizationResponse,
};
use crate::radcomp::context_tools::host_rpc;
use crate::radcomp::context_tools::types::RasRpcCommand;

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

    /// Role-based squashing: collapses consecutive runs of non-user/assistant
    /// messages (e.g. tool results) into the last message of the run.
    fn compress_messages(messages: &[Message], summary_parts: &mut Vec<String>) -> Vec<Message> {
        let mut optimized_messages = Vec::new();
        let mut i = 0;

        while i < messages.len() {
            let role = messages[i].role.as_str();
            if role == "user" || role == "assistant" {
                optimized_messages.push(messages[i].clone());
                i += 1;
            } else {
                let mut j = i;
                while j < messages.len()
                    && messages[j].role != "user"
                    && messages[j].role != "assistant"
                {
                    j += 1;
                }

                let count = j - i;
                if count > 1 {
                    let last_msg = &messages[j - 1];
                    summary_parts.push(format!(
                        "Compressed {} messages (role: '{}') into one.",
                        count, last_msg.role
                    ));
                    optimized_messages.push(last_msg.clone());
                } else {
                    optimized_messages.push(messages[i].clone());
                }
                i = j;
            }
        }

        optimized_messages
    }
}

impl Guest for MyContextTools {
    fn optimize(request: OptimizationRequest) -> Result<OptimizationResponse, String> {
        if request.messages.is_empty() {
            return Ok(OptimizationResponse {
                optimized_messages: Vec::new(),
                summary: "Empty request.".to_string(),
            });
        }

        let mut summary_parts = Vec::new();

        let windowed =
            Self::apply_history_window(request.messages, request.max_history, &mut summary_parts);
        let optimized_messages = Self::compress_messages(&windowed, &mut summary_parts);

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

export!(MyContextTools);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::radcomp::context_tools::context_tools::Message;

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
    fn test_optimize_with_compression() {
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
        assert_eq!(result.optimized_messages.len(), 3);
        assert!(
            result
                .summary
                .contains("Compressed 2 messages (role: 'tool') into one.")
        );
        assert_eq!(result.optimized_messages[1].content, "Second tool result");
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
    fn test_optimize_windowing_then_compression() {
        // Goal + alternating tool/assistant turns; windowing first trims the
        // list, then role-squashing collapses any remaining tool runs.
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
        // Window keeps [goal, result A, result B, final reply] (4 messages),
        // then compression squashes the two consecutive tool messages into one.
        assert_eq!(result.optimized_messages.len(), 3);
        assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
        assert_eq!(result.optimized_messages[1].content, "result B");
        assert_eq!(result.optimized_messages[2].content, "final reply");
        assert!(result.summary.contains("Windowed history from 6 to 4"));
        assert!(
            result
                .summary
                .contains("Compressed 2 messages (role: 'tool') into one.")
        );
    }
}
