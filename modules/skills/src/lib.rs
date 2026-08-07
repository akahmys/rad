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

/// One tool per skill, matching the extension exactly. Collapsing these into a
/// single `skill(name)` tool is AWU 962 — kept separate so a change in what the
/// model sees cannot be mistaken for a porting error.
/// Cannot fail — discovery returns whatever is on disk, and a malformed
/// `SKILL.md` is skipped rather than raised. Wrapped at the call site with
/// `rad_sdk::infallible` instead of returning a `Result` nothing ever fills.
fn list(_req: ListReq) -> ListRes {
    let tools = skill::discover_skills()
        .into_iter()
        .map(|s| {
            serde_json::json!({
                "type": "function",
                "function": {
                    "name": s.name,
                    "description": s.description,
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "args": {
                                "type": "string",
                                "description": "Optional context or arguments to pass to the skill"
                            }
                        }
                    }
                }
            })
        })
        .collect();
    ListRes { tools }
}

fn call(req: CallReq) -> Result<CallRes, Error> {
    let CallReq { name, arguments } = req;
    let skill = skill::discover_skills()
        .into_iter()
        .find(|s| s.name == name)
        .ok_or_else(|| Error::invalid(format!("Unknown skill: {name}")))?;

    let args = serde_json::from_str::<serde_json::Value>(&arguments)
        .ok()
        .and_then(|v| {
            v.get("args")
                .and_then(|a| a.as_str())
                .map(ToString::to_string)
        })
        .unwrap_or_default();

    // Returned directly. The extension had to escape this for a shell.
    Ok(CallRes {
        content: skill::substitute_args(skill.body, &args),
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
