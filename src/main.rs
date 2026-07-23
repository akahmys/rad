#![deny(clippy::pedantic)]

use clap::Parser;
use rad::command::{CommandHelper, CommandManager, CommandParser, CommandResult};
use rad::config;
use rustyline::Config;
use rustyline::history::{History, MemHistory};
use rustyline::{Editor, error::ReadlineError};

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

fn load_config_and_session(
    args: &Args,
) -> Result<
    (
        rad::config::Config,
        String,
        std::sync::Arc<parking_lot::Mutex<rad::dag::Dag>>,
    ),
    String,
> {
    let mut cfg = config::load_config(args.config.as_deref())
        .map_err(|e| format!("Error loading configuration: {e}"))?;

    // Apply CLI overrides (Tier 1 Priority)
    if let Some(ref ws) = args.workspace {
        cfg.core.workspace.clone_from(ws);
    }
    let active_name = cfg.llm.active.clone().unwrap_or_else(|| "default".to_string());
    let profile = cfg.llm.endpoints.entry(active_name).or_default();
    if let Some(ref url) = args.base_url {
        profile.base_url.clone_from(url);
    }
    if let Some(ref key) = args.api_key {
        profile.api_key = Some(key.clone());
    }
    if let Some(ref model) = args.model {
        profile.model = Some(model.clone());
    }

    println!("\x1b[32mConfiguration loaded successfully!\x1b[0m");
    println!("Workspace Dir: {}", cfg.core.workspace);
    println!("Snapshot Dir: {}", cfg.core.snapshot);
    println!("Log Dir: {}", cfg.core.log);
    let enabled_exts: Vec<_> = cfg
        .extensions
        .iter()
        .filter(|ext| ext.enabled && rad::config::expand_tilde(&ext.source).exists())
        .collect();
    println!("Extensions loaded ({}):", enabled_exts.len());
    for ext in &enabled_exts {
        let mcp_names: Vec<String> = ext
            .config
            .get("mcp_servers")
            .and_then(serde_json::Value::as_object)
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default();

        if mcp_names.is_empty() {
            println!("  - {}", ext.name);
        } else {
            println!("  - {} (MCP: {})", ext.name, mcp_names.join(", "));
        }
    }

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

    Ok((
        cfg,
        session_id,
        std::sync::Arc::new(parking_lot::Mutex::new(dag)),
    ))
}

fn init_editor(
    workspace: &str,
) -> Result<(Editor<CommandHelper, MemHistory>, std::path::PathBuf), String> {
    let mut rl =
        Editor::<CommandHelper, MemHistory>::with_history(Config::default(), MemHistory::new())
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

#[cfg(unix)]
struct RawModeGuard {
    orig_termios: nix::sys::termios::Termios,
}

#[cfg(unix)]
impl RawModeGuard {
    fn enable() -> Result<Self, String> {
        use nix::sys::termios::{LocalFlags, SetArg, tcgetattr, tcsetattr};

        let fd = std::io::stdin();
        let orig_termios = tcgetattr(&fd).map_err(|e| format!("Failed to get termios: {e}"))?;
        let mut raw_termios = orig_termios.clone();

        raw_termios.local_flags.remove(LocalFlags::ICANON);
        raw_termios.local_flags.remove(LocalFlags::ECHO);

        tcsetattr(&fd, SetArg::TCSADRAIN, &raw_termios)
            .map_err(|e| format!("Failed to set termios: {e}"))?;

        Ok(Self { orig_termios })
    }
}

#[cfg(unix)]
impl Drop for RawModeGuard {
    fn drop(&mut self) {
        use nix::sys::termios::{SetArg, tcsetattr};
        let fd = std::io::stdin();
        let _ = tcsetattr(&fd, SetArg::TCSADRAIN, &self.orig_termios);
    }
}

#[cfg(not(unix))]
struct RawModeGuard;

#[cfg(not(unix))]
impl RawModeGuard {
    fn enable() -> Result<Self, String> {
        Ok(Self)
    }
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

    let guard = RawModeGuard::enable()?;
    while orchestrator.is_running() {
        if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(50)) {
            let ev = crossterm::event::read();
            if let Ok(crossterm::event::Event::Key(crossterm::event::KeyEvent {
                code: crossterm::event::KeyCode::Esc,
                ..
            })) = ev
            {
                std::mem::drop(guard);
                println!("\n\x1b[1;33mTask execution aborted by user (Esc).\x1b[0m");
                orchestrator.abort();
                break;
            }
        }
    }
    rad::terminal::get_terminal().set_state(rad::terminal::TerminalState::Idle);
    Ok(())
}

fn process_input(
    line: &str,
    rl: &mut Editor<CommandHelper, MemHistory>,
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
        match CommandManager::execute(command, orchestrator) {
            CommandResult::Continue => {}
            CommandResult::Quit => {
                println!("\x1b[32mGoodbye!\x1b[0m");
                return Ok(false);
            }
            CommandResult::StatusInfo(info) => {
                println!("{info}");
            }
        }
        return Ok(true);
    }

    println!("\x1b[36mTask received: \x1b[1m{trimmed}\x1b[0m");

    run_agent_task(trimmed, orchestrator)?;

    let res = rad::session::save_session(&cfg.core.workspace, session_id, &dag_arc.lock());
    if let Err(e) = res {
        eprintln!("Failed to auto-save session: {e}");
    }
    Ok(true)
}

fn run_repl(
    mut rl: Editor<CommandHelper, MemHistory>,
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
    let _ = rl.history_mut().save(history_path);
}
