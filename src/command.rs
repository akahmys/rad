#![deny(clippy::pedantic)]

use std::fmt::{self, Write as _};

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
    /// Roll back the session state to a specific node ID.
    Rollback(String),
    /// Reload configuration file dynamically.
    Reload,
    /// Reset the current session (rotates session ID and clears DAG).
    Reset,
    /// Show history DAG visually as a tree.
    Tree,
    /// List active permissions and registered tools.
    Tools,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Help => write!(f, "/help"),
            Command::Exit => write!(f, "/exit"),
            Command::Status => write!(f, "/status"),
            Command::Clear => write!(f, "/clear"),
            Command::Session(id) => write!(f, "/session {id}"),
            Command::Rollback(id) => write!(f, "/rollback {id}"),
            Command::Reload => write!(f, "/reload"),
            Command::Reset => write!(f, "/reset"),
            Command::Tree => write!(f, "/tree"),
            Command::Tools => write!(f, "/tools"),
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
            "/rollback" => {
                if parts.len() > 1 {
                    Some(Command::Rollback(parts[1].to_string()))
                } else {
                    None
                }
            }
            "/reload" => Some(Command::Reload),
            "/reset" => Some(Command::Reset),
            "/tree" => Some(Command::Tree),
            "/tools" => Some(Command::Tools),
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
    pub fn execute(
        command: Command,
        orchestrator: &crate::orchestrator::Orchestrator,
    ) -> CommandResult {
        match command {
            Command::Help => {
                println!("Available Slash Commands:");
                println!("  /help           - Show this help message");
                println!("  /exit           - Exit the session");
                println!("  /status         - Show current session status and DAG info");
                println!("  /clear          - Clear the terminal screen");
                println!("  /session <id>   - Show the current session ID");
                println!("  /rollback <id>  - Roll back session to a specific DAG node");
                println!("  /reload         - Reload configuration file dynamically");
                println!("  /reset          - Save current session and start a new clean session");
                println!("  /tree           - Show history DAG visually as a tree");
                println!("  /tools          - List permissions and registered tools");
                CommandResult::Continue
            }
            Command::Exit => CommandResult::Exit,
            Command::Status => {
                let session_id = orchestrator.session_id.lock().clone();
                let usage_guard = orchestrator.token_usage.lock();
                let (prompt, completion) =
                    (usage_guard.prompt_tokens, usage_guard.completion_tokens);
                let total = prompt + completion;
                let mut status_msg = format!("Session ID: {session_id}\n");

                {
                    let dag_guard = orchestrator.dag.lock();
                    let total_nodes = dag_guard.nodes.len();
                    let current_node = dag_guard.current_node_id.as_deref().unwrap_or("None");
                    let _ = write!(
                        status_msg,
                        "Total DAG Nodes: {total_nodes}\nCurrent DAG Node: {current_node}\n"
                    );
                }

                let _ = write!(
                    status_msg,
                    "Token Usage: Prompt: {prompt}, Completion: {completion}, Total: {total}"
                );

                CommandResult::StatusInfo(status_msg)
            }
            Command::Clear => {
                // ANSI escape sequences to clear screen and reset cursor to top-left
                print!("{}[2J{}[1;1H", 27 as char, 27 as char);
                CommandResult::Continue
            }
            Command::Session(id) => CommandResult::StatusInfo(format!("Current session: {id}")),
            Command::Rollback(node_id) => {
                match orchestrator.rollback(&node_id) {
                    Ok(()) => {
                        println!("Session successfully rolled back to node: {node_id}");
                    }
                    Err(e) => {
                        eprintln!("Failed to rollback: {e}");
                    }
                }
                CommandResult::Continue
            }
            Command::Reload => match orchestrator.reload() {
                Ok(()) => CommandResult::StatusInfo(
                    "\x1b[32mConfiguration reloaded successfully!\x1b[0m".to_string(),
                ),
                Err(e) => CommandResult::StatusInfo(format!(
                    "\x1b[1;31mFailed to reload configuration: {e}\x1b[0m"
                )),
            },
            Command::Reset => match orchestrator.reset_session() {
                Ok(new_id) => CommandResult::StatusInfo(format!(
                    "\x1b[32mSession reset successfully. Started new session: \x1b[1;36m{new_id}\x1b[0m"
                )),
                Err(e) => CommandResult::StatusInfo(format!(
                    "\x1b[1;31mFailed to reset session: {e}\x1b[0m"
                )),
            },
            Command::Tree => {
                let dag_guard = orchestrator.dag.lock();
                let tree_str = tree::render_dag_tree(&dag_guard);
                CommandResult::StatusInfo(tree_str)
            }
            Command::Tools => {
                let tools_str = tools::render_tools_and_permissions(orchestrator);
                CommandResult::StatusInfo(tools_str)
            }
        }
    }
}

pub mod completion;
pub use completion::CommandHelper;
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
