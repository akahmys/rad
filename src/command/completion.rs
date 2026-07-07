use std::borrow::Cow;
use rustyline::error::ReadlineError;
use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Context, Helper};

/// A helper that implements slash command completion, shell command completion, and file path completion.
pub struct CommandHelper {
    file_completer: FilenameCompleter,
}

impl CommandHelper {
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_completer: FilenameCompleter::new(),
        }
    }
}

impl Default for CommandHelper {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for CommandHelper {
    type Candidate = String;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>), ReadlineError> {
        if line.starts_with('/') {
            let mut candidates = Vec::new();
            let commands = ["/help", "/exit", "/status", "/clear", "/session", "/rollback", "/reload", "/reset"];
            for cmd in commands {
                if cmd.starts_with(line) {
                    candidates.push(cmd.to_string());
                }
            }
            Ok((pos - line.len(), candidates))
        } else if line.starts_with('!') {
            let cur_line = &line[..pos];
            let words: Vec<&str> = cur_line.split_whitespace().collect();
            let is_command_name = !cur_line.contains(' ') || (words.len() == 1 && !cur_line.ends_with(' '));
            
            if is_command_name {
                let current_word = words.first().copied().unwrap_or("");
                if let Some(prefix) = current_word.strip_prefix('!') {
                    let candidates = complete_system_commands(prefix);
                    let pairs = candidates.into_iter().map(|cmd| format!("!{cmd}")).collect();
                    Ok((pos - current_word.len(), pairs))
                } else {
                    Ok((pos, Vec::new()))
                }
            } else if let Ok((pos_out, candidates)) = self.file_completer.complete(line, pos, ctx) {
                let replacement_strings = candidates.into_iter().map(|c| c.replacement).collect();
                Ok((pos_out, replacement_strings))
            } else {
                Ok((pos, Vec::new()))
            }
        } else if let Ok((pos_out, candidates)) = self.file_completer.complete(line, pos, ctx) {
            let replacement_strings = candidates.into_iter().map(|c| c.replacement).collect();
            Ok((pos_out, replacement_strings))
        } else {
            Ok((pos, Vec::new()))
        }
    }
}

fn complete_system_commands(prefix: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let Ok(path_var) = std::env::var("PATH") else { return commands; };
    for path_dir in std::env::split_paths(&path_var) {
        let Ok(entries) = std::fs::read_dir(path_dir) else { continue; };
        for entry in entries.flatten() {
            let file_name = entry.file_name().to_string_lossy().into_owned();
            if file_name.starts_with(prefix) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if let Ok(meta) = entry.metadata() {
                        let is_exec = meta.mode() & 0o111 != 0;
                        if is_exec && meta.is_file() {
                            commands.push(file_name);
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            commands.push(file_name);
                        }
                    }
                }
            }
        }
    }
    commands.sort();
    commands.dedup();
    commands.truncate(20);
    commands
}

impl Hinter for CommandHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _ctx: &Context<'_>) -> Option<Self::Hint> {
        None
    }
}

impl Highlighter for CommandHelper {
    fn highlight<'a>(&self, line: &'a str, _pos: usize) -> Cow<'a, str> {
        Cow::Borrowed(line)
    }
}

impl Validator for CommandHelper {
    fn validate(&self, _ctx: &mut ValidationContext<'_>) -> Result<ValidationResult, ReadlineError> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for CommandHelper {}
