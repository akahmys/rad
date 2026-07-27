//! Markdown-template-based lightweight slash commands (Phase 47-2):
//! user-defined shortcuts that expand into a task prompt with no code
//! required, pi-coding-agent style. Checked as a second tier after the
//! built-in `command_specs()` registry in `process_input` (`main.rs`) —
//! Rust's static function-pointer table can't hold entries discovered at
//! runtime from the filesystem, so this is a separate lookup rather than a
//! literal single array, but the user-facing behavior is unified: `/foo`
//! resolves the same way whether `foo` is a built-in or a template.
use std::path::PathBuf;

#[cfg(test)]
mod tests;

const PLACEHOLDER: &str = "$ARGUMENTS";

/// Project-local `.agents/commands/` takes priority over user-global
/// `~/.rad/commands/` (same precedence direction as `AGENTS.md`'s
/// project-before-global convention).
fn template_dirs(workspace: &str) -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from(workspace).join(".agents/commands")];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".rad/commands"));
    }
    dirs
}

/// Looks up `name` among markdown-template commands, substitutes
/// `$ARGUMENTS` with `args` (appended on its own line if the placeholder
/// isn't present and `args` is non-empty), and returns the expanded task
/// text. `None` if no template named `name` exists in any template dir.
#[must_use]
pub fn expand(workspace: &str, name: &str, args: &str) -> Option<String> {
    for dir in template_dirs(workspace) {
        let Ok(content) = std::fs::read_to_string(dir.join(format!("{name}.md"))) else {
            continue;
        };
        return Some(if content.contains(PLACEHOLDER) {
            content.replace(PLACEHOLDER, args)
        } else if args.is_empty() {
            content
        } else {
            format!("{content}\n\n{args}")
        });
    }
    None
}

/// Lists available template command names (for `/help`), project-local
/// first, deduplicated and sorted.
#[must_use]
pub fn list_names(workspace: &str) -> Vec<String> {
    let mut names = Vec::new();
    for dir in template_dirs(workspace) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "md") {
                continue;
            }
            if let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && !names.iter().any(|n: &String| n == name)
            {
                names.push(name.to_string());
            }
        }
    }
    names.sort();
    names
}
