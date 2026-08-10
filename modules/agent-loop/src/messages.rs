//! Turning the DAG into the message list a request is built from.
//!
//! Ported from `ext/rad-orchestrator/src/llm.rs`. The parts that move here are
//! the ones that are functions of their input: walking the DAG, dropping
//! orphaned tool replies, and assembling the system prompt. What stays behind
//! for now is everything that asks the host a question — `GetDag`,
//! `GetActiveLlmProfile`, `GenerateLlmStream` — because a module has no way to
//! ask those yet. See PLANS.md; the answer decides AWU 981's shape, not this
//! one's.
//!
//! One thing shrinks on the way. `read_rule_file` went through a `FileRead`
//! RPC and then decoded a `Vec<u8>` out of JSON; `std::fs::read_to_string`
//! does it, because §3.1's rule puts the filesystem on WASI where `std` already
//! insulates the module.
use std::collections::HashMap;
use std::fmt::Write as _;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct ToolCallFunction {
    pub(crate) name: String,
    pub(crate) arguments: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct ToolCall {
    pub(crate) id: String,
    #[serde(rename = "type")]
    pub(crate) tool_type: String,
    pub(crate) function: ToolCallFunction,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub(crate) struct Message {
    pub(crate) role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) tool_calls: Option<Vec<ToolCall>>,
}

#[derive(serde::Deserialize, Default)]
pub(crate) struct DagNode {
    pub(crate) parent_ids: Vec<String>,
    pub(crate) node_type: String,
    pub(crate) text: String,
}

#[derive(serde::Deserialize, Default)]
pub(crate) struct Dag {
    pub(crate) nodes: HashMap<String, DagNode>,
    pub(crate) current_node_id: Option<String>,
}

/// Only these become messages; anything else in the DAG is bookkeeping.
const MESSAGE_ROLES: [&str; 4] = ["user", "assistant", "tool", "system"];

/// Walks from `current_node_id` to the root and returns the messages in
/// conversation order.
///
/// A node's text is a serialised `Message` when the orchestrator wrote it and
/// plain text when a human did, so both are accepted — with the node's own
/// `node_type` winning over any `role` inside the JSON, since the DAG is what
/// actually knows who spoke.
pub(crate) fn traverse(dag: &Dag) -> Vec<Message> {
    let mut messages = Vec::new();
    let mut current_id = dag.current_node_id.clone();

    while let Some(ref id) = current_id {
        let Some(node) = dag.nodes.get(id) else {
            break;
        };
        if MESSAGE_ROLES.contains(&node.node_type.as_str()) {
            let msg = parse_node(node);
            let empty = msg.content.as_ref().is_none_or(String::is_empty)
                && msg.tool_calls.as_ref().is_none_or(Vec::is_empty);
            if !empty {
                messages.push(msg);
            }
        }
        current_id = node.parent_ids.first().cloned();
    }

    messages.reverse();
    messages
}

fn parse_node(node: &DagNode) -> Message {
    serde_json::from_str::<Message>(&node.text).map_or_else(
        |_| Message {
            role: node.node_type.clone(),
            content: if node.text.is_empty() {
                None
            } else {
                Some(node.text.clone())
            },
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
        |mut parsed| {
            parsed.role.clone_from(&node.node_type);
            parsed
        },
    )
}

/// Drops `tool` messages with no matching `assistant` `tool_calls` entry
/// earlier in the list.
///
/// Applied twice by the caller, and the comment the extension carried is the
/// reason: orphans can pre-exist from a rollback or a rehydration, and
/// `context-tools`' count-based windowing is purely positional, so it can split
/// an `assistant`/`tool` pair across the window boundary and *create* one. That
/// is the class of bug behind AWU 78's "400 Bad Request".
pub(crate) fn filter_orphaned_tool_messages(messages: Vec<Message>) -> Vec<Message> {
    let mut filtered: Vec<Message> = Vec::new();
    for msg in messages {
        if msg.role != "tool" {
            filtered.push(msg);
            continue;
        }
        let answered = msg.tool_call_id.as_ref().is_some_and(|tid| {
            filtered.iter().any(|prev| {
                prev.role == "assistant"
                    && prev
                        .tool_calls
                        .as_ref()
                        .is_some_and(|calls| calls.iter().any(|tc| tc.id == *tid))
            })
        });
        if answered {
            filtered.push(msg);
        }
    }
    filtered
}

/// Project rules, in the order the extension read them.
///
/// Read through `std::fs` rather than a `FileRead` RPC. A missing file is not
/// an error — most projects have neither.
fn load_local_agent_rules() -> String {
    let mut combined = String::new();
    for path in [".agents/AGENTS.md", "AGENTS.md"] {
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if !combined.is_empty() {
            combined.push_str("\n\n");
        }
        let _ = write!(combined, "### Local Project Rules ({path}):\n{content}");
    }
    if combined.is_empty() {
        String::new()
    } else {
        format!("\n\n{combined}")
    }
}

/// The base prompt, verbatim from the extension: it is what the model has been
/// tuned against in every transcript so far, so a reworded version is a
/// behaviour change wearing a port's clothes.
const BASE_PROMPT: &str = "You are an expert coding assistant operating inside rad, a coding agent harness. You help users by reading files, executing commands, editing code, and writing new files.";

pub(crate) fn system_prompt() -> String {
    let mut prompt = BASE_PROMPT.to_string();
    prompt.push_str(&load_local_agent_rules());
    prompt
}

#[cfg(test)]
mod tests;
