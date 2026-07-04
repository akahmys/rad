#![deny(clippy::pedantic)]

mod config;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rad", version = "0.1.0", about = "Rust Agent Dispatcher")]
struct Args {
    #[arg(short, long, help = "Path to config file")]
    config: Option<String>,
}

fn main() {
    let args = Args::parse();

    match config::load_config(args.config.as_deref()) {
        Ok(cfg) => {
            println!("Configuration loaded successfully!");
            println!("Workspace Dir: {}", cfg.core.workspace);
            println!("Snapshot Dir: {}", cfg.core.snapshot);
            println!("Log Dir: {}", cfg.core.log);
            println!("Extensions loaded: {}", cfg.extensions.len());
            for ext in &cfg.extensions {
                println!(
                    " - Name: {}, Source: {}, Enabled: {}",
                    ext.name, ext.source, ext.enabled
                );
            }
        }
        Err(e) => {
            eprintln!("Error loading configuration: {e}");
            std::process::exit(1);
        }
    }
}
