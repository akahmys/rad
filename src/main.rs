#![deny(clippy::pedantic)]

use rad::config;
use std::io::{self, Write};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rad", version = "0.2.0", about = "Rust Agent Dispatcher")]
struct Args {
    #[arg(short, long, help = "Path to config file")]
    config: Option<String>,

    #[arg(short, long, help = "Session ID to reload or resume")]
    session: Option<String>,
}

fn main() {
    let args = Args::parse();

    let cfg = match config::load_config(args.config.as_deref()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading configuration: {e}");
            std::process::exit(1);
        }
    };

    println!("Configuration loaded successfully!");
    println!("Workspace Dir: {}", cfg.core.workspace);
    println!("Snapshot Dir: {}", cfg.core.snapshot);
    println!("Log Dir: {}", cfg.core.log);
    println!("Extensions loaded: {}", cfg.extensions.len());

    let session_id = args.session.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string()
    });

    let dag = if let Ok(loaded) = rad::session::load_session(&cfg.core.workspace, &session_id) {
        println!("Resumed session: {session_id}");
        loaded
    } else {
        println!("Started new session: {session_id}");
        rad::dag::Dag::new()
    };
    let dag_arc = std::sync::Arc::new(std::sync::Mutex::new(dag));

    let orchestrator = rad::orchestrator::Orchestrator::new(cfg.clone(), session_id.clone(), dag_arc.clone());

    println!("Starting rad agent shell. Type 'exit' or 'quit' to end the session.");

    loop {
        print!("rad > ");
        if io::stdout().flush().is_err() {
            eprintln!("Failed to flush stdout");
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!();
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed == "exit" || trimmed == "quit" {
                    println!("Goodbye!");
                    break;
                }
                if trimmed.is_empty() {
                    continue;
                }
                println!("Task received: {trimmed}");
                
                if let Err(e) = orchestrator.run_task(trimmed.to_string()) {
                    eprintln!("Execution error: {e}");
                }
                
                if let Ok(dag_guard) = dag_arc.lock() {
                    let res = rad::session::save_session(&cfg.core.workspace, &session_id, &dag_guard);
                    if let Err(e) = res {
                        eprintln!("Failed to auto-save session: {e}");
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        }
    }
}
