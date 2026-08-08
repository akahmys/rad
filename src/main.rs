#![deny(clippy::pedantic)]

use clap::Parser;
use rad::command::{CommandHelper, CommandManager, CommandParser, CommandResult};
use rustyline::history::DefaultHistory;
use rustyline::{Editor, error::ReadlineError};

mod startup;
use startup::load_config_and_session;

#[derive(Parser, Debug)]
#[command(name = "rad", version, about = "Rust Agent Dispatcher")]
struct Args {
    #[arg(short, long, help = "Path to config file")]
    config: Option<String>,

    #[arg(short, long, help = "Session ID to reload or resume")]
    session: Option<String>,

    #[arg(long, help = "Override LLM Base URL")]
    base_url: Option<String>,

    #[arg(long, help = "Override LLM API Key")]
    api_key: Option<String>,

    #[arg(long, help = "Override LLM Model")]
    model: Option<String>,

    #[arg(short, long, help = "Override workspace directory")]
    workspace: Option<String>,
}

fn init_editor(
    workspace: &str,
) -> Result<(Editor<CommandHelper, DefaultHistory>, std::path::PathBuf), String> {
    let mut rl = Editor::<CommandHelper, DefaultHistory>::new()
        .map_err(|e| format!("Failed to initialize shell editor: {e}"))?;

    rl.set_helper(Some(rad::command::CommandHelper::new()));

    let history_path = std::path::PathBuf::from(workspace).join(".rad/history");
    if history_path.exists() {
        let _ = rl.load_history(&history_path);
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

    // Eager-load all configured extensions now rather than waiting for the
    // first task (or `/tools`) to trigger it lazily: measured cost is
    // ~150ms in a release build for this project's 5 extensions —
    // negligible against a one-time CLI startup, and it surfaces a broken
    // extension immediately instead of mid-task. `get_or_init_runtimes` is
    // idempotent (skips already-loaded extensions), so this doesn't change
    // behavior for the real per-task load that follows later, just moves
    // the one-time cost earlier.
    let (throwaway_tx, _throwaway_rx) = std::sync::mpsc::channel();
    if let Err(e) = orchestrator.get_or_init_runtimes(&throwaway_tx) {
        eprintln!("\x1b[33mWarning: failed to initialize extensions at startup: {e}\x1b[0m");
    }

    // The kernel comes up inside `Orchestrator::new` (it is part of the same
    // config), so there is nothing to boot or hand over here — only something
    // to report.
    let loaded_modules = orchestrator
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.modules())
        .unwrap_or_default();
    if !loaded_modules.is_empty() {
        println!(
            "\x1b[32m[OK] Loaded {} kernel module(s): {}\x1b[0m",
            loaded_modules.len(),
            loaded_modules.join(", ")
        );
    }

    println!("\x1b[1;36mStarting rad agent shell. Type '/quit' to end the session.\x1b[0m");

    let (rl, history_path) = match init_editor(&cfg.core.workspace) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("\x1b[1;31m{e}\x1b[0m");
            std::process::exit(1);
        }
    };

    run_repl(
        rl,
        &history_path,
        &orchestrator,
        &cfg,
        &session_id,
        &dag_arc,
    );
}

fn run_agent_task(
    task: &str,
    orchestrator: &std::sync::Arc<rad::orchestrator::Orchestrator>,
) -> Result<(), String> {
    rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Thinking);

    if let Err(e) = orchestrator.run_task(task.to_string()) {
        rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Idle);
        return Err(format!("Execution error: {e}"));
    }

    // Esc aborts the running task without needing Enter first. Only
    // possible while stdin can be put into raw mode (a real terminal);
    // falls back to the plain poll loop otherwise (e.g. piped stdin).
    let raw_guard = rad::esc_abort::RawInputGuard::enable();
    let mut aborted = false;
    while orchestrator.is_running() {
        if let Some(ref guard) = raw_guard
            && !aborted
            && rad::esc_abort::esc_pressed(guard)
        {
            aborted = true;
            orchestrator.abort();
            rad::terminal::get_terminal().write_log("\x1b[33m[Aborted by user]\x1b[0m".to_string());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Idle);
    Ok(())
}

fn process_input(
    line: &str,
    rl: &mut Editor<CommandHelper, DefaultHistory>,
    orchestrator: &std::sync::Arc<rad::orchestrator::Orchestrator>,
    cfg: &rad::config::Config,
    session_id: &str,
    dag_arc: &std::sync::Arc<parking_lot::Mutex<rad::dag::Dag>>,
) -> Result<bool, String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Ok(true);
    }

    let _ = rl.add_history_entry(trimmed);

    if let Some(stripped) = trimmed.strip_prefix('!') {
        let cmd_to_run = stripped.trim();
        if !cmd_to_run.is_empty() {
            rad::command::execute_shell(cmd_to_run);
        }
        return Ok(true);
    }

    if let Some(command) = CommandParser::parse(trimmed) {
        match CommandManager::execute(&command, orchestrator) {
            CommandResult::Continue => {}
            CommandResult::Quit => {
                println!("\x1b[32mGoodbye!\x1b[0m");
                return Ok(false);
            }
            CommandResult::StatusInfo(info) => {
                println!("{info}");
            }
            CommandResult::RunTask(task) => {
                return run_task_and_save(&task, orchestrator, cfg, session_id, dag_arc);
            }
        }
        return Ok(true);
    }

    // Not a built-in command — check markdown-template commands
    // (`.agents/commands/<name>.md` / `~/.rad/commands/<name>.md`) before
    // falling through to sending the raw input as a task. An unrecognized
    // `/whatever` with no matching template still falls through
    // deliberately, matching `CommandParser::parse`'s existing contract.
    if let Some((name, args)) = rad::command::split_slash(trimmed)
        && let Some(expanded) = rad::command::templates::expand(&cfg.core.workspace, name, args)
    {
        println!("\x1b[36mExpanded /{name} into a task:\x1b[0m\n{expanded}");
        return run_task_and_save(&expanded, orchestrator, cfg, session_id, dag_arc);
    }

    println!("\x1b[36mTask received: \x1b[1m{trimmed}\x1b[0m");
    run_task_and_save(trimmed, orchestrator, cfg, session_id, dag_arc)
}

fn run_task_and_save(
    task: &str,
    orchestrator: &std::sync::Arc<rad::orchestrator::Orchestrator>,
    cfg: &rad::config::Config,
    session_id: &str,
    dag_arc: &std::sync::Arc<parking_lot::Mutex<rad::dag::Dag>>,
) -> Result<bool, String> {
    run_agent_task(task, orchestrator)?;
    let res = rad::session::save_session(&cfg.core.workspace, session_id, &dag_arc.lock());
    if let Err(e) = res {
        eprintln!("Failed to auto-save session: {e}");
    }
    Ok(true)
}

fn run_repl(
    mut rl: Editor<CommandHelper, DefaultHistory>,
    history_path: &std::path::Path,
    orchestrator: &std::sync::Arc<rad::orchestrator::Orchestrator>,
    cfg: &rad::config::Config,
    session_id: &str,
    dag_arc: &std::sync::Arc<parking_lot::Mutex<rad::dag::Dag>>,
) {
    loop {
        let readline = rl.readline("\x1b[1;32mrad > \x1b[0m");
        match readline {
            Ok(line) => {
                match process_input(&line, &mut rl, orchestrator, cfg, session_id, dag_arc) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(e) => eprintln!("\x1b[1;31m{e}\x1b[0m"),
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
    let _ = rl.save_history(history_path);
}
