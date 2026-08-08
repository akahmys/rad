//! SKILL.md discovery and inline execution, as a kernel module.
//!
//! Ported from `skill-tool-provider`. Two things disappear rather than move,
//! and both were artefacts of the old boundary.
//!
//! The extension returned a skill's body by shelling out —
//! `open_process("echo -n '...'")` — because its WIT export had to return an
//! `execution-handle`. A module that only reads Markdown therefore needed bash
//! execution permission. `handle()` returns a string, so the hack and the
//! permission requirement both go.
//!
//! And `mode: subagent` is gone: it only ever returned "not implemented", and
//! subagents were dropped as a goal (§1.2).
#![deny(clippy::pedantic)]

mod skill;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct ListReq {}

#[derive(serde::Serialize)]
pub struct ListRes {
    pub tools: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct CallReq {
    pub name: String,
    /// The tool-call arguments, as the JSON string the model produced.
    #[serde(default)]
    pub arguments: String,
}

#[derive(serde::Serialize)]
pub struct CallRes {
    pub content: String,
}

/// The single tool name. Skills are selected by argument, not by tool.
const TOOL: &str = "skill";

/// One tool, whose description is the index (§4.5 ③).
///
/// The extension published one tool per skill. A tool schema costs 468
/// characters on average (§4.4), so that grew the prompt linearly in the number
/// of skills, for entries that differ only in a name and one line of prose. The
/// index carries the same information at a fraction of the size, and progressive
/// disclosure is unchanged: the body is still read only when the skill runs.
///
/// The cost is real — separate tools are easier for a model to notice, and
/// their arguments can be typed individually. Claude Code publishes a single
/// `Skill` tool, which is the evidence that this side of the trade works in
/// practice.
///
/// Cannot fail — discovery returns whatever is on disk, and a malformed
/// `SKILL.md` is skipped rather than raised. Wrapped at the call site with
/// `rad_sdk::infallible` instead of returning a `Result` nothing ever fills.
fn list(_req: ListReq) -> ListRes {
    let skills = skill::discover_skills();
    // No skills, no tool. An empty index would spend schema on an offer the
    // model cannot take, and invite a call that can only fail.
    if skills.is_empty() {
        return ListRes { tools: Vec::new() };
    }

    let width = skills.iter().map(|s| s.name.len()).max().unwrap_or(0);
    let index = skills
        .iter()
        .map(|s| format!("  {:width$}  {}", s.name, s.description))
        .collect::<Vec<_>>()
        .join("\n");

    ListRes {
        tools: vec![serde_json::json!({
            "type": "function",
            "function": {
                "name": TOOL,
                "description": format!(
                    "Run one of the available skills. Each is a set of \
                     instructions to follow; running one returns its text.\n\n\
                     Available skills:\n{index}"
                ),
                "parameters": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            // Enumerated as well as listed: the index is prose the
                            // model may paraphrase, whereas this constrains what it
                            // can emit.
                            "enum": skills.iter().map(|s| &s.name).collect::<Vec<_>>(),
                            "description": "Which skill to run"
                        },
                        "args": {
                            "type": "string",
                            "description": "Optional context or arguments to pass to the skill"
                        }
                    },
                    "required": ["name"]
                }
            }
        })],
    }
}

fn call(req: CallReq) -> Result<CallRes, Error> {
    let CallReq { name, arguments } = req;
    if name != TOOL {
        return Err(Error::invalid(format!(
            "Unknown tool: {name} (this module provides '{TOOL}')"
        )));
    }

    let args: serde_json::Value =
        serde_json::from_str(&arguments).unwrap_or(serde_json::Value::Null);
    let requested = args
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| Error::invalid("The 'name' argument is required".to_string()))?;

    let skills = skill::discover_skills();
    let skill = skills.iter().find(|s| s.name == requested).ok_or_else(|| {
        // Names listed back: the model chose from an index, so the useful
        // reply to a miss is what it could have chosen instead.
        let available = skills
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        Error::invalid(format!(
            "Unknown skill: {requested} (available: {available})"
        ))
    })?;

    let extra = args
        .get("args")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();

    // Returned directly. The extension had to escape this for a shell.
    Ok(CallRes {
        content: skill::substitute_args(skill.body.clone(), extra),
    })
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "skills",
    version: "0.1.0",
    methods: {
        "skills.tools.list" => rad_sdk::infallible(list),
        "skills.tools.call" => call,
    }
}
