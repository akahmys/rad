// Built-in `CommandSpec` handler implementations, split out of `command.rs`
// to stay under the 300-line file limit.
use super::{CommandResult, command_specs, compact, llm, templates, tools, tree};
use crate::orchestrator::Orchestrator;
use std::fmt::Write as _;
use std::sync::Arc;

pub(super) fn cmd_help(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    let mut out = String::from("Available Slash Commands:\n");
    for spec in command_specs() {
        let _ = writeln!(out, "  /{:<10} - {}", spec.name, spec.description);
    }
    let workspace = orchestrator.config.lock().core.workspace.clone();
    let template_names = templates::list_names(&workspace);
    if !template_names.is_empty() {
        let _ = writeln!(out, "\nCustom commands (.agents/commands/, ~/.rad/commands/):");
        for name in template_names {
            let _ = writeln!(out, "  /{name}");
        }
    }
    CommandResult::StatusInfo(out)
}

// `/status` was a strict superset of the old `/session` (which only showed
// the ID) — merged into one command under the `/session` name, matching
// pi-coding-agent's precedent of a single `/session` command covering
// "session file, ID, messages, tokens, and cost" rather than splitting
// that across two commands with overlapping output.
pub(super) fn cmd_session(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    let session_id = orchestrator.session_id.lock().clone();
    let usage_guard = orchestrator.token_usage.lock();
    let (prompt, completion) = (usage_guard.prompt_tokens, usage_guard.completion_tokens);
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

pub(super) fn cmd_rollback(args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    let node_id = args.trim();
    if node_id.is_empty() {
        return CommandResult::StatusInfo(
            "\x1b[1;31mUsage: /rollback <node_id>\x1b[0m".to_string(),
        );
    }
    match orchestrator.rollback(node_id) {
        Ok(()) => println!("Session successfully rolled back to node: {node_id}"),
        Err(e) => eprintln!("Failed to rollback: {e}"),
    }
    CommandResult::Continue
}

pub(super) fn cmd_reload(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    match orchestrator.reload() {
        Ok(()) => CommandResult::StatusInfo(
            "\x1b[32mConfiguration reloaded successfully!\x1b[0m".to_string(),
        ),
        Err(e) => {
            CommandResult::StatusInfo(format!("\x1b[1;31mFailed to reload configuration: {e}\x1b[0m"))
        }
    }
}

pub(super) fn cmd_new(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    match orchestrator.reset_session() {
        Ok(new_id) => CommandResult::StatusInfo(format!(
            "\x1b[32mSession reset successfully. Started new session: \x1b[1;36m{new_id}\x1b[0m"
        )),
        Err(e) => CommandResult::StatusInfo(format!("\x1b[1;31mFailed to reset session: {e}\x1b[0m")),
    }
}

pub(super) fn cmd_tree(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    let dag_guard = orchestrator.dag.lock();
    let tree_str = tree::render_dag_tree(&dag_guard);
    CommandResult::StatusInfo(tree_str)
}

pub(super) fn cmd_tools(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    let tools_str = tools::render_tools_and_permissions(orchestrator);
    CommandResult::StatusInfo(tools_str)
}

pub(super) fn cmd_llm(args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    CommandResult::StatusInfo(llm::execute_llm(args, orchestrator))
}

pub(super) fn cmd_compact(_args: &str, orchestrator: &Arc<Orchestrator>) -> CommandResult {
    CommandResult::StatusInfo(compact::run_compact(orchestrator))
}
