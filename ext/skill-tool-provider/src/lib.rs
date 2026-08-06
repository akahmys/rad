#![deny(clippy::pedantic)]

#[allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::pedantic
)]
mod bindings {
    wit_bindgen::generate!({
        path: "../../wit/rad.wit",
        world: "rad-tool-provider",
    });

    use super::SkillToolProviderImpl;
    export!(SkillToolProviderImpl);
}

pub use bindings::*;

use self::radcomp::extension::types as wit;

struct SkillToolProviderImpl;

mod skill;

use skill::{SkillMode, discover_skills, substitute_args};

impl Guest for SkillToolProviderImpl {
    fn get_tools() -> Result<String, String> {
        let tools: Vec<serde_json::Value> = discover_skills()
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
        serde_json::to_string(&tools).map_err(|e| format!("Failed to serialize tools: {e}"))
    }

    fn execute_tool(name: String, arguments: String) -> Result<wit::ExecutionHandle, String> {
        let skill = discover_skills()
            .into_iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("Unknown skill: {name}"))?;

        if skill.mode != SkillMode::Inline {
            return Err(format!(
                "Skill '{name}' declares a non-inline mode, which is not yet implemented (only inline execution is supported)"
            ));
        }

        let args = serde_json::from_str::<serde_json::Value>(&arguments)
            .ok()
            .and_then(|v| {
                v.get("args")
                    .and_then(|a| a.as_str())
                    .map(ToString::to_string)
            })
            .unwrap_or_default();
        let result = substitute_args(skill.body, &args);

        let escaped = result.replace('\'', "'\\''");
        open_process(&format!("echo -n '{escaped}'"))
    }
}
