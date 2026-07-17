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
    fn compress_messages(messages: &[Message]) -> (Vec<Message>, String) {
        let mut optimized_messages = Vec::new();
        let mut summary_parts = Vec::new();
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

        let summary = if summary_parts.is_empty() {
            "No messages were compressed.".to_string()
        } else {
            summary_parts.join(" ")
        };

        (optimized_messages, summary)
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

        let (optimized_messages, summary) = Self::compress_messages(&request.messages);

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

    #[test]
    fn test_optimize_no_compression() {
        let request = OptimizationRequest {
            messages: vec![
                Message {
                    node_id: Some("1".to_string()),
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                Message {
                    node_id: Some("2".to_string()),
                    role: "assistant".to_string(),
                    content: "Hi!".to_string(),
                },
            ],
        };
        let result = MyContextTools::optimize(request).unwrap();
        assert_eq!(result.optimized_messages.len(), 2);
        assert_eq!(result.summary, "No messages were compressed.");
    }

    #[test]
    fn test_optimize_with_compression() {
        let request = OptimizationRequest {
            messages: vec![
                Message {
                    node_id: Some("1".to_string()),
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                Message {
                    node_id: Some("2".to_string()),
                    role: "tool".to_string(),
                    content: "First tool result".to_string(),
                },
                Message {
                    node_id: Some("3".to_string()),
                    role: "tool".to_string(),
                    content: "Second tool result".to_string(),
                },
                Message {
                    node_id: Some("4".to_string()),
                    role: "assistant".to_string(),
                    content: "I got it.".to_string(),
                },
            ],
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
}
