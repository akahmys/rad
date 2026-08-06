//! Deterministic "session activity digest" (Phase 51-3): files touched and
//! commands run, extracted from `tool_calls` JSON on assistant messages —
//! no LLM summarization, no judgment calls, just a mechanical record.
//!
//! Appended to the system prompt, which `load_messages_from_dag` always
//! excludes from `context-tools` windowing/clearing (it's split out before
//! the `optimize` call and re-attached unconditionally afterward). That
//! makes the system message the one part of the request immune to any
//! future compaction, so a digest attached to it survives even the most
//! aggressive windowing without needing narrative LLM summarization —
//! exactly the property Phase 51-3 asked for.
//!
//! Best-effort and heuristic: this project has no built-in tools (Phase
//! 19–20 removed them), so tool schemas vary per MCP server and tool
//! names can't be relied on. Instead this looks for common argument key
//! names (`path`/`file_path`/`filename` for files, `command`/`cmd` for
//! shell commands) rather than any specific tool's identity.
use crate::tool::Message;

const MAX_DIGEST_ITEMS: usize = 30;

/// Builds the digest text to append to the system prompt, or `None` if no
/// file/command activity has happened yet (nothing to add).
pub fn build_digest_addendum(messages: &[Message]) -> Option<String> {
    let (files_touched, commands_run) = extract_facts(messages);
    if files_touched.is_empty() && commands_run.is_empty() {
        return None;
    }

    let mut parts = Vec::new();
    if !files_touched.is_empty() {
        parts.push(format!(
            "Files touched this session: {}",
            files_touched.join(", ")
        ));
    }
    if !commands_run.is_empty() {
        let cmds: Vec<String> = commands_run.iter().map(|c| format!("`{c}`")).collect();
        parts.push(format!("Commands run this session: {}", cmds.join(", ")));
    }

    Some(format!(
        "\n\n### Session Activity Digest (auto-tracked, survives context compaction)\n{}",
        parts.join("\n")
    ))
}

/// Walks every assistant `tool_calls` entry in order, collecting distinct
/// file paths and (deduplicated but order-preserving) commands from
/// whichever common argument key names are present. Caps each list to the
/// most recent `MAX_DIGEST_ITEMS` so the digest itself can't grow
/// unbounded over a very long session.
fn extract_facts(messages: &[Message]) -> (Vec<String>, Vec<String>) {
    let mut files_touched: Vec<String> = Vec::new();
    let mut commands_run: Vec<String> = Vec::new();

    for msg in messages {
        if msg.role != "assistant" {
            continue;
        }
        let Some(tool_calls) = &msg.tool_calls else {
            continue;
        };
        for call in tool_calls {
            let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.function.arguments)
            else {
                continue;
            };

            for key in ["path", "file_path", "filename"] {
                if let Some(p) = args.get(key).and_then(|v| v.as_str())
                    && !files_touched.iter().any(|f| f == p)
                {
                    files_touched.push(p.to_string());
                }
            }
            for key in ["command", "cmd"] {
                if let Some(c) = args.get(key).and_then(|v| v.as_str())
                    && !commands_run.iter().any(|existing| existing == c)
                {
                    commands_run.push(c.to_string());
                }
            }
        }
    }

    (
        cap_to_most_recent(files_touched),
        cap_to_most_recent(commands_run),
    )
}

fn cap_to_most_recent(mut items: Vec<String>) -> Vec<String> {
    if items.len() > MAX_DIGEST_ITEMS {
        items = items.split_off(items.len() - MAX_DIGEST_ITEMS);
    }
    items
}

#[cfg(test)]
mod tests;
