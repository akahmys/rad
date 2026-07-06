#![deny(clippy::pedantic)]

use rad::config;
use rad::command::{CommandHelper, CommandManager, CommandParser, CommandResult};
use clap::Parser;
use rustyline::{error::ReadlineError, Editor};
use rustyline::Config;
use rustyline::history::{History, MemHistory};

#[derive(Parser, Debug)]
#[command(name = "rad", version = "0.2.0", about = "Rust Agent Dispatcher")]
struct Args {
    #[arg(short, long, help = "Path to config file")]
    config: Option<String>,

    #[arg(short, long, help = "Session ID to reload or resume")]
    session: Option<String>,
}

fn load_config_and_session(
    args: &Args,
) -> Result<(rad::config::Config, String, std::sync::Arc<std::sync::Mutex<rad::dag::Dag>>), String> {
    let cfg = config::load_config(args.config.as_deref())
        .map_err(|e| format!("Error loading configuration: {e}"))?;

    println!("\x1b[32mConfiguration loaded successfully!\x1b[0m");
    println!("Workspace Dir: {}", cfg.core.workspace);
    println!("Snapshot Dir: {}", cfg.core.snapshot);
    println!("Log Dir: {}", cfg.core.log);
    println!("Extensions loaded: {}", cfg.extensions.len());

    let session_id = args.session.clone().unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string()
    });

    let dag = if let Ok(loaded) = rad::session::load_session(&cfg.core.workspace, &session_id) {
        println!("\x1b[36mResumed session: {session_id}\x1b[0m");
        loaded
    } else {
        println!("\x1b[36mStarted new session: {session_id}\x1b[0m");
        rad::dag::Dag::new()
    };

    Ok((cfg, session_id, std::sync::Arc::new(std::sync::Mutex::new(dag))))
}

fn init_editor(workspace: &str) -> Result<(Editor<CommandHelper, MemHistory>, std::path::PathBuf), String> {
    let mut rl = Editor::<CommandHelper, MemHistory>::with_history(
        Config::default(),
        MemHistory::new(),
    )
    .map_err(|e| format!("Failed to initialize shell editor: {e}"))?;

    rl.set_helper(Some(rad::command::CommandHelper::new()));

    let history_path = std::path::PathBuf::from(workspace).join(".rad/history");
    if history_path.exists() {
        let _ = rl.history_mut().load(&history_path);
    }

    Ok((rl, history_path))
}

fn main() {
    let args = Args::parse();

    let (cfg, session_id, dag_arc) = match load_config_and_session(&args) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    let orchestrator = std::sync::Arc::new(rad::orchestrator::Orchestrator::new(
        cfg.clone(),
        session_id.clone(),
        dag_arc.clone(),
        args.config.clone(),
    ));

    println!("\x1b[1;36mStarting rad agent shell. Type 'exit' or 'quit' to end the session.\x1b[0m");

    let (mut rl, history_path) = match init_editor(&cfg.core.workspace) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("\x1b[1;31m{e}\x1b[0m");
            std::process::exit(1);
        }
    };

    loop {
        let readline = rl.readline("\x1b[1;32mrad > \x1b[0m");
        match readline {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                
                let _ = rl.add_history_entry(trimmed);

                if let Some(stripped) = trimmed.strip_prefix('!') {
                    let cmd_to_run = stripped.trim();
                    if !cmd_to_run.is_empty() {
                        rad::command::execute_shell(cmd_to_run);
                    }
                    continue;
                }

                if let Some(command) = CommandParser::parse(trimmed) {
                    match CommandManager::execute(command, &orchestrator) {
                        CommandResult::Continue => {}
                        CommandResult::Exit => {
                            println!("\x1b[32mGoodbye!\x1b[0m");
                            break;
                        }
                        CommandResult::StatusInfo(info) => {
                            println!("{info}");
                        }
                    }
                    continue;
                }

                if trimmed == "exit" || trimmed == "quit" {
                    println!("\x1b[32mGoodbye!\x1b[0m");
                    break;
                }

                println!("\x1b[36mTask received: \x1b[1m{trimmed}\x1b[0m");
                
                rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Thinking);

                if let Err(e) = orchestrator.run_task(trimmed.to_string()) {
                    rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Idle);
                    eprintln!("\x1b[1;31mExecution error: {e}\x1b[0m");
                } else {
                    while orchestrator.is_running() {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Idle);
                }
                
                if let Ok(dag_guard) = dag_arc.lock() {
                    let res = rad::session::save_session(&cfg.core.workspace, &session_id, &dag_guard);
                    if let Err(e) = res {
                        eprintln!("Failed to auto-save session: {e}");
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }

    if let Some(parent) = history_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = rl.history_mut().save(&history_path);
}
