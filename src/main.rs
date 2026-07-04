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

    let mut dag = if let Ok(loaded) = rad::session::load_session(&cfg.core.workspace, &session_id) {
        println!("Resumed session: {session_id}");
        loaded
    } else {
        println!("Started new session: {session_id}");
        rad::dag::Dag::new()
    };

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
                
                // For demonstration, create a task node and save DAG
                if let Ok(node_id) = dag.create_node("", "task") {
                    let _ = dag.set_node_text(&node_id, trimmed);
                }
                
                if let Err(e) = rad::session::save_session(&cfg.core.workspace, &session_id, &dag) {
                    eprintln!("Failed to auto-save session: {e}");
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {e}");
                break;
            }
        }
    }
}
