//! Discovery of `.agents/skills/<name>/SKILL.md` (project-local, checked
//! first) and `~/.rad/skills/<name>/SKILL.md` (user-global) — same
//! precedence direction as `src/command/templates.rs`'s custom slash
//! commands, but surfaced as LLM-discoverable tools instead of user-typed
//! `/name` commands.
//!
//! Ported from the `skill-tool-provider` extension. Discovery went through
//! `ListDir`/`FileRead` host RPCs there; here it is `std::fs`, because the
//! kernel deliberately does not reimplement what WASI already provides
//! (`ARCHITECTURE-NEXT.md` §3.4.1). Reachability is decided by the host's
//! preopens rather than by a permission mask, which is what that section
//! establishes was already true in practice.

#[cfg(test)]
mod tests;

const ARGS_PLACEHOLDER: &str = "$ARGUMENTS";

pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

fn skill_dirs() -> [&'static str; 2] {
    [".agents/skills", "~/.rad/skills"]
}

fn list_dir(path: &str) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(expand_tilde(path)) else {
        return Vec::new();
    };
    entries
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .collect()
}

fn read_file(path: &str) -> Option<String> {
    std::fs::read_to_string(expand_tilde(path)).ok()
}

/// `~` expansion, which the host RPC used to do on the extension's behalf.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(rest);
    }
    std::path::PathBuf::from(path)
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
    for line in frontmatter.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "description" if !value.is_empty() => description = Some(value.to_string()),
            // `mode` is gone: `subagent` only ever returned "not implemented",
            // and subagents were dropped as a goal (ARCHITECTURE-NEXT.md §1.2).
            // An unknown key is ignored rather than rejected, so an old
            // `mode: subagent` line does not stop a skill from loading.
            //
            // "allowed_tools" is still reserved — enforcement belongs to the
            // `policy` module (§4.5.2), not here.
            _ => {}
        }
    }

    Some(Skill {
        name: name.to_string(),
        description: description?,
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
