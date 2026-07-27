#![deny(clippy::pedantic)]

use std::sync::Arc;

use crate::orchestrator::Orchestrator;
use handlers::{
    cmd_compact, cmd_help, cmd_llm, cmd_new, cmd_reload, cmd_rollback, cmd_session, cmd_tools,
    cmd_tree,
};

/// A slash command matched against the registry (`command_specs`), plus
/// whatever trailing text followed its name. `name` is the command's
/// canonical registered name even when the input used an alias (e.g.
/// `/models` resolves to `name: "llm"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub name: &'static str,
    pub args: String,
}

/// Result of executing a command.
pub enum CommandResult {
    /// The command was executed, continue the loop.
    Continue,
    /// The command requested exiting the application.
    Quit,
    /// The command produced status information to be printed.
    StatusInfo(String),
    /// The command expanded into a task prompt to run through the agent
    /// (currently only produced by markdown-template commands — see
    /// `templates` — but kept as a first-class `CommandResult` variant
    /// rather than a template-specific side channel, since "run this text
    /// as a task" is a generic outcome any future command could produce).
    RunTask(String),
}

/// One entry in the command registry: name, aliases, help text, and
/// handler are declared together so they can never drift out of sync with
/// each other (previously the command enum, parser match, dispatcher
/// match, hand-written `/help` text, and `CommandHelper`'s tab-completion
/// list were 5 independently-maintained sources of truth — `CommandHelper`
/// didn't even know `/llm` existed). Every handler takes the raw trailing
/// text after the command name and parses whatever structure it needs
/// itself, the same way `/llm`'s subcommands already worked.
pub struct CommandSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
    pub handler: fn(&str, &Arc<Orchestrator>) -> CommandResult,
}

#[must_use]
pub fn command_specs() -> &'static [CommandSpec] {
    &[
        CommandSpec {
            name: "help",
            aliases: &[],
            description: "Show this help message",
            handler: cmd_help,
        },
        CommandSpec {
            name: "quit",
            aliases: &[],
            description: "Exit the session",
            handler: |_, _| CommandResult::Quit,
        },
        CommandSpec {
            name: "session",
            aliases: &[],
            description: "Show session ID, DAG info, and token usage",
            handler: cmd_session,
        },
        CommandSpec {
            name: "rollback",
            aliases: &[],
            description: "Roll back session to a specific DAG node (usage: /rollback <node_id>)",
            handler: cmd_rollback,
        },
        CommandSpec {
            name: "reload",
            aliases: &[],
            description: "Reload configuration file dynamically",
            handler: cmd_reload,
        },
        CommandSpec {
            name: "new",
            aliases: &[],
            description: "Save current session and start a new clean session",
            handler: cmd_new,
        },
        CommandSpec {
            name: "tree",
            aliases: &[],
            description: "Show history DAG visually as a tree",
            handler: cmd_tree,
        },
        CommandSpec {
            name: "tools",
            aliases: &[],
            description: "List permissions and registered tools",
            handler: cmd_tools,
        },
        CommandSpec {
            name: "llm",
            aliases: &["models"],
            description: "Manage LLM endpoints (list, switch, test, add, model, context)",
            handler: cmd_llm,
        },
        CommandSpec {
            name: "compact",
            aliases: &[],
            description: "Manually compact and persist session history now",
            handler: cmd_compact,
        },
    ]
}

/// Splits `/name rest` into `(name, rest)`, trimming `rest`. `None` if
/// `input` isn't `/`-prefixed or the name portion is empty. Shared between
/// `CommandParser::parse` (built-in registry lookup) and the markdown
/// template lookup in `main.rs` (`templates` module) — both need to
/// extract the same `name`/`args` shape from raw input, just against
/// different sources of "known names."
#[must_use]
pub fn split_slash(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim();
    let rest = trimmed.strip_prefix('/')?;
    let (name, args) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    if name.is_empty() {
        return None;
    }
    Some((name, args.trim()))
}

/// Parser for identifying slash commands in user input.
pub struct CommandParser;

impl CommandParser {
    /// Parses the input string against the command registry. Returns
    /// `Some(ParsedCommand)` for a recognized `/name ...` command,
    /// otherwise `None` (indicating it's a regular task, a markdown
    /// template command — see `templates` — or an unrecognized
    /// `/whatever` sent to the LLM as-is).
    #[must_use]
    pub fn parse(input: &str) -> Option<ParsedCommand> {
        let (name, args) = split_slash(input)?;
        let spec = command_specs()
            .iter()
            .find(|spec| spec.name == name || spec.aliases.contains(&name))?;
        Some(ParsedCommand {
            name: spec.name,
            args: args.to_string(),
        })
    }
}

/// Manages the execution of slash commands.
pub struct CommandManager;

impl CommandManager {
    /// Executes the given parsed command and returns the result.
    #[must_use]
    pub fn execute(command: &ParsedCommand, orchestrator: &Arc<Orchestrator>) -> CommandResult {
        match command_specs().iter().find(|spec| spec.name == command.name) {
            Some(spec) => (spec.handler)(&command.args, orchestrator),
            None => CommandResult::StatusInfo(format!("Unknown command: /{}", command.name)),
        }
    }
}

pub mod compact;
pub mod completion;
pub use completion::CommandHelper;
mod handlers;
pub mod llm;
pub mod templates;
pub mod tools;
pub mod tree;

/// Executes a command directly on the host shell.
pub fn execute_shell(cmd_str: &str) {
    let mut cmd = std::process::Command::new("sh");
    cmd.arg("-c").arg(cmd_str);
    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                if let Some(code) = status.code() {
                    eprintln!("Command exited with status code: {code}");
                } else {
                    eprintln!("Command terminated by signal");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to execute command: {e}");
        }
    }
}
