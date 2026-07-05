#![deny(clippy::pedantic)]

use std::fmt;
use std::borrow::Cow;
use rustyline::error::ReadlineError;
use rustyline::completion::Completer;
use rustyline::hint::Hinter;
use rustyline::highlight::Highlighter;
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use rustyline::{Helper, Context};

/// Represents the available slash commands in the REPL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Show the help menu.
    Help,
    /// Exit the session.
    Exit,
    /// Show current session status.
    Status,
    /// Clear the terminal screen.
    Clear,
    /// Display information about the current session ID.
    Session(String),
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Help => write!(f, "/help"),
            Command::Exit => write!(f, "/exit"),
            Command::Status => write!(f, "/status"),
            Command::Clear => write!(f, "/clear"),
            Command::Session(id) => write!(f, "/session {id}"),
        }
    }
}

/// Parser for identifying slash commands in user input.
pub struct CommandParser;

impl CommandParser {
    /// Parses the input string. Returns `Some(Command)` if it's a valid slash command,
    /// otherwise returns `None` (indicating it's a regular task).
    #[must_use]
    pub fn parse(input: &str) -> Option<Command> {
        let trimmed = input.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "/help" => Some(Command::Help),
            "/exit" => Some(Command::Exit),
            "/status" => Some(Command::Status),
            "/clear" => Some(Command::Clear),
            "/session" => {
                if parts.len() > 1 {
                    Some(Command::Session(parts[1].to_string()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Result of executing a command.
pub enum CommandResult {
    /// The command was executed, continue the loop.
    Continue,
    /// The command requested exiting the application.
    Exit,
    /// The command produced status information to be printed.
    StatusInfo(String),
}

/// Manages the execution of slash commands.
pub struct CommandManager;

impl CommandManager {
    /// Executes the given command and returns the result.
    #[must_use]
    pub fn execute(command: Command, session_id: &str) -> CommandResult {
        match command {
            Command::Help => {
                println!("Available Slash Commands:");
                println!("  /help       - Show this help message");
                println!("  /exit       - Exit the session");
                println!("  /status     - Show current session status");
                println!("  /clear      - Clear the terminal screen");
                println!("  /session <id> - Show the current session ID");
                CommandResult::Continue
            }
            Command::Exit => CommandResult::Exit,
            Command::Status => CommandResult::StatusInfo(format!("Current session: {session_id}")),
            Command::Clear => {
                // ANSI escape sequences to clear screen and reset cursor to top-left
                print!("{}[2J{}[1;1H", 27 as char, 27 as char);
                CommandResult::Continue
            }
            Command::Session(id) => CommandResult::StatusInfo(format!("Current session: {id}")),
        }
    }
}

/// A helper that implements slash command completion and provides default behaviors for other traits.
pub struct CommandHelper;

impl CommandHelper {
    #[must_use]
    pub fn new() -> Self {
        Self
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
        word: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<String>), ReadlineError> {
        if word.starts_with('/') {
            let mut candidates = Vec::new();
            let commands = ["/help", "/exit", "/status", "/clear", "/session"];
            for cmd in commands {
                if cmd.starts_with(word) {
                    candidates.push(cmd.to_string());
                }
            }
            Ok((pos + word.len(), candidates))
        } else {
            Ok((pos, Vec::new()))
        }
    }
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
