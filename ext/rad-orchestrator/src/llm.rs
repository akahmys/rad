use std::fmt::Write;
use crate::call_host;
use crate::tool::{Message, get_available_tools};
use crate::types::{Dag, RasRpcCommand};

fn load_local_agent_rules() -> String {
    let paths = [".agents/AGENTS.md", "AGENTS.md"];
    let mut combined = String::new();
    for p in &paths {
        let path_buf = std::path::PathBuf::from(p);
        if let Ok(val) = call_host(RasRpcCommand::FileRead { path: path_buf }) {
            if let Ok(bytes) = serde_json::from_value::<Vec<u8>>(val) {
                if let Ok(content) = String::from_utf8(bytes) {
                    if !combined.is_empty() {
                        combined.push_str("\n\n");
                    }
                    let _ = write!(combined, "### Local Project Rules ({p}):\n{content}");
                }
            }
        }
    }
    if combined.is_empty() {
        String::new()
    } else {
        format!("\n\n{combined}")
    }
}

fn get_system_prompt() -> String {
    let mut prompt = "You are an expert coding assistant operating inside rad, a coding agent harness. You help users by reading files, executing commands, editing code, and writing new files.".to_string();
    let rules = load_local_agent_rules();
    if !rules.is_empty() {
        prompt.push_str(&rules);
    }
    prompt
}

pub fn load_messages_from_dag() -> Result<Vec<Message>, String> {
    let dag_val = call_host(RasRpcCommand::GetDag)?;
    let dag: Dag =
        serde_json::from_value(dag_val).map_err(|e| format!("Failed to parse Dag: {e}"))?;
    let mut messages = Vec::new();
    let mut current_id = dag.current_node_id;

    while let Some(ref id) = current_id {
        if let Some(node) = dag.nodes.get(id) {
            let is_valid_role = matches!(node.node_type.as_str(), "user" | "assistant" | "tool" | "system");

            if is_valid_role {
                let msg = if let Ok(mut parsed_msg) = serde_json::from_str::<Message>(&node.text) {
                    parsed_msg.role.clone_from(&node.node_type);
                    parsed_msg
                } else {
                    Message {
                        role: node.node_type.clone(),
                        content: if node.text.is_empty() {
                            None
                        } else {
                            Some(node.text.clone())
                        },
                        name: None,
                        tool_call_id: None,
                        tool_calls: None,
                    }
                };

                let is_empty_msg = msg.content.as_ref().is_none_or(String::is_empty)
                    && msg.tool_calls.as_ref().is_none_or(Vec::is_empty);

                if !is_empty_msg {
                    messages.push(msg);
                }
            }
            current_id = node.parent_ids.first().cloned();
        } else {
            break;
        }
    }

    messages.reverse();

    let max_history = crate::orchestrator::STATE
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|s| s.max_history_messages))
        .unwrap_or(30);

    let messages_to_send = if messages.len() > max_history && !messages.is_empty() {
        let first_goal = messages[0].clone();
        let remaining_len = messages.len() - 1;
        let limit = max_history - 1;
        let start_idx = if remaining_len > limit {
            messages.len() - limit
        } else {
            1
        };
        let mut trimmed = vec![first_goal];
        trimmed.extend(messages[start_idx..].to_vec());
        trimmed
    } else {
        messages
    };

    let mut filtered_messages = Vec::new();
    for msg in messages_to_send {
        if msg.role == "tool" {
            let has_matching_call = if let Some(ref tid) = msg.tool_call_id {
                filtered_messages.iter().any(|prev_msg: &Message| {
                    if prev_msg.role == "assistant" {
                        if let Some(ref tcalls) = prev_msg.tool_calls {
                            tcalls.iter().any(|tc| tc.id == *tid)
                        } else {
                            false
                        }
                    } else {
                        // Correct type type
                        false
                    }
                })
            } else {
                false
            };
            if has_matching_call {
                filtered_messages.push(msg);
            }
        } else {
            filtered_messages.push(msg);
        }
    }

    let mut all_messages = vec![Message {
        role: "system".to_string(),
        content: Some(get_system_prompt()),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    all_messages.extend(filtered_messages);

    // Split system message and non-system messages for context optimization
    let mut system_msg = None;
    let mut non_system_msgs = Vec::new();
    for msg in all_messages {
        if msg.role == "system" {
            system_msg = Some(msg);
        } else {
            non_system_msgs.push(msg);
        }
    }

    let mut optimized_non_system = non_system_msgs;

    let ct_req = CtOptimizationRequest {
        messages: optimized_non_system
            .iter()
            .map(|m| CtMessage {
                node_id: None,
                role: m.role.clone(),
                content: serde_json::to_string(m).unwrap_or_default(),
            })
            .collect(),
    };

    if let Ok(req_json) = serde_json::to_string(&ct_req) {
        if let Ok(res_val) = call_host(RasRpcCommand::CallExtension {
            extension_id: "context-tools".to_string(),
            method: "optimize".to_string(),
            arguments: req_json,
        }) {
            if let Some(res_str) = res_val.as_str() {
                if let Ok(ct_resp) = serde_json::from_str::<CtOptimizationResponse>(res_str) {
                    let mut temp_msgs = Vec::new();
                    for m in ct_resp.optimized_messages {
                        if let Ok(parsed) = serde_json::from_str::<Message>(&m.content) {
                            temp_msgs.push(parsed);
                        } else {
                            temp_msgs.push(Message {
                                role: m.role,
                                content: Some(m.content),
                                name: None,
                                tool_call_id: None,
                                tool_calls: None,
                            });
                        }
                    }
                    if !temp_msgs.is_empty() {
                        let _ = call_host(RasRpcCommand::WriteStdout {
                            text: format!(
                                "\n\x1b[2m[Context optimized: {}]\x1b[0m\n",
                                ct_resp.summary
                            ),
                        });
                        optimized_non_system = temp_msgs;
                    }
                }
            }
        }
    }

    let mut final_messages = Vec::new();
    if let Some(sys) = system_msg {
        final_messages.push(sys);
    }
    final_messages.extend(optimized_non_system);

    Ok(final_messages)
}

pub fn trigger_llm_stream(messages: Vec<Message>) -> Result<(), String> {
    crate::log_trace("session", "Getting available tools...");
    let tools = get_available_tools().unwrap_or_default();
    crate::log_trace("session", &format!("Got {} available tools. Serializing...", tools.len()));

    let messages_json = serde_json::to_string(&messages)
        .map_err(|e| format!("Failed to serialize messages: {e}"))?;
    let tools_json =
        serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"))?;

    crate::log_trace("session", "Calling GenerateLlmStream host RPC...");
    call_host(RasRpcCommand::GenerateLlmStream {
        model: "qwen".to_string(),
        messages_json,
        tools_json,
    })?;
    crate::log_trace("session", "GenerateLlmStream host RPC call completed.");
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CtMessage {
    #[serde(rename = "node-id")]
    node_id: Option<String>,
    role: String,
    content: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CtOptimizationRequest {
    messages: Vec<CtMessage>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CtOptimizationResponse {
    #[serde(rename = "optimized-messages")]
    optimized_messages: Vec<CtMessage>,
    summary: String,
}
