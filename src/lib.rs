pub mod command;
pub mod config;
pub mod dag;
pub mod error;
pub mod fs;
pub mod git;
pub mod http;
pub mod ipc;
pub mod mcp;
pub mod orchestrator;
pub mod process;
pub mod repo_map;
pub mod session;
pub mod subsystems;
pub mod sys;
pub mod terminal;
pub mod wasm;

#[macro_export]
macro_rules! log_host {
    ($($arg:tt)*) => {
        if ::std::env::var("RAD_DEBUG").is_ok() {
            println!($($arg)*);
        }
    };
}
