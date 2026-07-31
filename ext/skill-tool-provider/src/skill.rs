//! Discovery of `.agents/skills/<name>/SKILL.md` (project-local, checked
//! first) and `~/.rad/skills/<name>/SKILL.md` (user-global) — same
//! precedence direction as `src/command/templates.rs`'s custom slash
//! commands, but surfaced as LLM-discoverable tools instead of user-typed
//! `/name` commands.
use crate::{host_rpc, radcomp::extension::types as wit};

#[cfg(test)]
mod tests;

const ARGS_PLACEHOLDER: &str = "$ARGUMENTS";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillMode {
    Inline,
    /// Reserved for future nested-task execution — not implemented yet.
    Subagent,
}

pub struct Skill {
    pub name: String,
    pub description: String,
    pub mode: SkillMode,
    pub body: String,
}

fn skill_dirs() -> [&'static str; 2] {
    [".agents/skills", "~/.rad/skills"]
}

fn list_dir(path: &str) -> Vec<String> {
    let Ok(res_str) = host_rpc(&wit::RasRpcCommand::ListDir(path.to_string())) else {
        return Vec::new();
    };
    serde_json::from_str(&res_str).unwrap_or_default()
}

fn read_file(path: &str) -> Option<String> {
    let res_str = host_rpc(&wit::RasRpcCommand::FileRead(path.to_string())).ok()?;
    if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&res_str) {
        return String::from_utf8(bytes).ok();
    }
    serde_json::from_str::<String>(&res_str).ok()
}

/// Parses a `SKILL.md`'s `---`-delimited frontmatter (simple `key: value`
/// lines, not full YAML — the schema is a handful of scalar fields, not
/// worth a new dependency) plus the body that follows it. `None` if the
/// frontmatter delimiters are missing/malformed or `description` (the only
/// required field) is absent.
fn parse_skill_md(name: &str, content: &str) -> Option<Skill> {
    let rest = content.strip_prefix("---")?;
    let (frontmatter, body) = rest.split_once("---")?;

    let mut description = None;
    let mut mode = SkillMode::Inline;
    for line in frontmatter.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "description" if !value.is_empty() => description = Some(value.to_string()),
            "mode" if value == "subagent" => mode = SkillMode::Subagent,
            // "allowed_tools" and unknown fields are reserved/ignored for now.
            _ => {}
        }
    }

    Some(Skill {
        name: name.to_string(),
        description: description?,
        mode,
        body: body.trim_start_matches('\n').to_string(),
    })
}

/// Discovers all skills, project-local first, deduplicated by directory
/// name (first-seen wins, matching `templates.rs::list_names`'s
/// precedence). A name whose `SKILL.md` fails to read or parse is skipped
/// entirely rather than falling back to a same-named skill in the other
/// directory, mirroring how a project-local template shadows a
/// same-named global one for custom commands.
pub fn discover_skills() -> Vec<Skill> {
    let mut skills = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for dir in skill_dirs() {
        for name in list_dir(dir) {
            if !seen.insert(name.clone()) {
                continue;
            }
            let Some(content) = read_file(&format!("{dir}/{name}/SKILL.md")) else {
                continue;
            };
            if let Some(skill) = parse_skill_md(&name, &content) {
                skills.push(skill);
            }
        }
    }
    skills
}

/// Substitutes `$ARGUMENTS` in a skill's body with `args` (appended on its
/// own line if the placeholder isn't present and `args` is non-empty),
/// same logic as `templates.rs::expand`.
pub fn substitute_args(body: String, args: &str) -> String {
    if body.contains(ARGS_PLACEHOLDER) {
        body.replace(ARGS_PLACEHOLDER, args)
    } else if args.is_empty() {
        body
    } else {
        format!("{body}\n\n{args}")
    }
}
