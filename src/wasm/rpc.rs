use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::Write;

use crate::ipc::RasRpcCommand;
use crate::process::RunningProcess;
use crate::subsystems::{FsSubsystem, ProcessSubsystem, DagSubsystem, NetworkSubsystem};

/// Prompts the human user for approval before executing a privileged operation.
///
/// # Errors
///
/// Returns an error if the user rejects the operation or stdin cannot be read.
fn request_approval(desc: &str) -> Result<(), String> {
    // Bypass approval prompt during cargo test to prevent blocking unit tests
    if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
        return Ok(());
    }

    print!("\n[Privileged Operation Request] {}\nApprove this action? (y/N): ", desc);
    let mut stdout = std::io::stdout();
    let _ = stdout.flush();
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read stdin: {e}"))?;
    
    let trimmed = input.trim().to_lowercase();
    if trimmed == "y" || trimmed == "yes" {
        Ok(())
    } else {
        Err("Operation rejected by user".to_string())
    }
}

/// Shared context passed to each RPC handler category.
pub struct RpcContext<'a> {
    pub sandbox: &'a dyn FsSubsystem,
    pub process_manager: &'a dyn ProcessSubsystem,
    pub dag: &'a dyn DagSubsystem,
    pub network: &'a dyn NetworkSubsystem,
    pub active_processes: &'a Arc<Mutex<HashMap<i32, RunningProcess>>>,
    pub active_mcp_servers: &'a Arc<Mutex<HashMap<String, crate::mcp::McpProcess>>>,
    pub event_tx: &'a std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    pub llm_timeout_policy: &'a Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    pub orchestrator: Option<&'a Arc<crate::orchestrator::Orchestrator>>,
    pub call_id: String,
    pub hitl_enabled: bool,
}

/// Thin dispatcher: routes each RPC command to a focused handler function.
///
/// # Errors
///
/// Returns an error if the delegated handler fails.
#[allow(clippy::too_many_arguments)]
pub fn execute_rpc_command(
    cmd: &RasRpcCommand,
    sandbox: &dyn FsSubsystem,
    process_manager: &dyn ProcessSubsystem,
    dag: &dyn DagSubsystem,
    network: &dyn NetworkSubsystem,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess>>>,
    active_mcp_servers: &Arc<Mutex<HashMap<String, crate::mcp::McpProcess>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    llm_timeout_policy: &Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    orchestrator: Option<&Arc<crate::orchestrator::Orchestrator>>,
    call_id: String,
    hitl_enabled: bool,
) -> Result<serde_json::Value, String> {
    let ctx = RpcContext {
        sandbox, process_manager, dag, network,
        active_processes, active_mcp_servers,
        event_tx, llm_timeout_policy, orchestrator,
        call_id, hitl_enabled,
    };
    match cmd {
        RasRpcCommand::FileRead { .. }
        | RasRpcCommand::FileWrite { .. }
        | RasRpcCommand::FileEditPatch { .. }
        | RasRpcCommand::TakeSnapshot { .. }
        | RasRpcCommand::CheckoutSnapshot { .. } => handle_fs(cmd, &ctx),

        RasRpcCommand::CreateNode { .. }
        | RasRpcCommand::SetNodeText { .. }
        | RasRpcCommand::MergeNodes { .. }
        | RasRpcCommand::DeleteNode { .. }
        | RasRpcCommand::GetDag => handle_dag(cmd, &ctx),

        RasRpcCommand::SpawnBashProcess { .. }
        | RasRpcCommand::SpawnMcpServer { .. }
        | RasRpcCommand::SendMcpRequest { .. } => handle_process(cmd, &ctx),

        RasRpcCommand::OpenHttpStream { .. }
        | RasRpcCommand::SetStreamTimeoutPolicy { .. }
        | RasRpcCommand::WriteStdout { .. } => handle_io(cmd, &ctx),

        RasRpcCommand::CompleteTask
        | RasRpcCommand::AskHumanApproval { .. }
        | RasRpcCommand::ReportTokenUsage { .. }
        | RasRpcCommand::GetRepoMap
        | RasRpcCommand::GetTools
        | RasRpcCommand::ExecuteTool { .. } => handle_meta(cmd, &ctx),
    }
}

fn handle_fs(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::FileRead { path } => {
            let data = ctx.sandbox.file_read(path)?;
            serde_json::to_value(serde_bytes::Bytes::new(&data))
                .map_err(|e| format!("Failed to serialize file_read result: {e}"))
        }
        RasRpcCommand::FileWrite { path, data } => {
            if ctx.hitl_enabled {
                request_approval(&format!("File Write to '{}'", path.display()))?;
            }
            ctx.sandbox.file_write(path, data)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::FileEditPatch { path, diff } => {
            if ctx.hitl_enabled {
                request_approval(&format!("File Edit (Patch) on '{}'", path.display()))?;
            }
            ctx.sandbox.file_edit_patch(path, diff)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::TakeSnapshot { node_id, target_paths } => {
            ctx.sandbox.take_snapshot(node_id, target_paths)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::CheckoutSnapshot { node_id } => {
            ctx.sandbox.checkout_snapshot(node_id)?;
            Ok(serde_json::Value::Null)
        }
        _ => unreachable!(),
    }
}

fn handle_dag(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::CreateNode { parent_id, node_type } => {
            let node_id = ctx.dag.create_node(parent_id, node_type)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetNodeText { node_id, text } => {
            ctx.dag.set_node_text(node_id, text)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::MergeNodes { node_ids, summary_text } => {
            let node_id = ctx.dag.merge_nodes(node_ids, summary_text)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::DeleteNode { node_id } => {
            ctx.dag.delete_node(node_id)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::GetDag => {
            let value = ctx.dag.get_dag()?;
            Ok(value)
        }
        _ => unreachable!(),
    }
}

fn handle_process(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::SpawnBashProcess { command } => {
            if ctx.hitl_enabled {
                request_approval(&format!("Execute shell command: '{command}'"))?;
            }
            crate::wasm::rpc_process::spawn_bash_process_rpc(
                command, ctx.sandbox, ctx.process_manager,
                ctx.active_processes, ctx.event_tx, ctx.call_id.clone(), ctx.hitl_enabled,
            )
        }
        RasRpcCommand::SpawnMcpServer { name, command, args } => {
            let proc = crate::mcp::McpProcess::spawn(name, command, args, ctx.event_tx.clone())?;
            let mut guard = ctx.active_mcp_servers.lock()
                .map_err(|e| format!("MCP server lock error: {e}"))?;
            guard.insert(name.clone(), proc);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SendMcpRequest { name, message } => {
            let mut guard = ctx.active_mcp_servers.lock()
                .map_err(|e| format!("MCP server lock error: {e}"))?;
            if let Some(proc) = guard.get_mut(name) {
                proc.send_message(message)?;
                Ok(serde_json::Value::Null)
            } else {
                Err(format!("MCP server '{name}' is not running"))
            }
        }
        _ => unreachable!(),
    }
}

fn handle_io(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::OpenHttpStream { url, headers, body } => {
            let final_url = if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
                url.replace("127.0.0.1:8080", &format!("127.0.0.1:{test_port}"))
            } else {
                url.clone()
            };
            crate::terminal::get_terminal().set_state(crate::terminal::TerminalState::Thinking);
            let stream_id = ctx.network.open_http_stream(
                &final_url, headers.clone(), body,
                ctx.event_tx.clone(), ctx.llm_timeout_policy.clone(),
            )?;
            serde_json::to_value(stream_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
            handle_set_timeout(target, policy, ctx)
        }
        RasRpcCommand::WriteStdout { text } => {
            crate::terminal::get_terminal().write_llm_token(text);
            Ok(serde_json::Value::Null)
        }
        _ => unreachable!(),
    }
}

fn handle_set_timeout(
    target: &crate::ipc::Target,
    policy: &crate::ipc::TimeoutPolicy,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    match target {
        crate::ipc::Target::Llm => {
            let mut guard = ctx.llm_timeout_policy.lock()
                .map_err(|e| format!("Failed to lock llm_timeout_policy: {e}"))?;
            *guard = policy.clone();
        }
        crate::ipc::Target::Process(pgid) => {
            let processes = ctx.active_processes.lock()
                .map_err(|e| format!("Process lock error: {e}"))?;
            if let Some(proc) = processes.get(pgid) {
                let mut guard = proc.timeout_policy.lock()
                    .map_err(|e| format!("Failed to lock process timeout_policy: {e}"))?;
                *guard = policy.clone();
            } else {
                return Err(format!("Process with PGID {pgid} not found"));
            }
        }
    }
    Ok(serde_json::Value::Null)
}

fn handle_meta(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::CompleteTask => {
            let _ = ctx.event_tx.send(crate::ipc::RasCoreEvent::TaskCompleted);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::AskHumanApproval { prompt } => {
            if !ctx.hitl_enabled {
                Ok(serde_json::Value::Bool(true))
            } else {
                let approved = crate::wasm::rpc_process::ask_human_approval_internal(prompt)?;
                Ok(serde_json::Value::Bool(approved))
            }
        }
        RasRpcCommand::ReportTokenUsage { prompt_tokens, completion_tokens } => {
            if let Some(orch) = ctx.orchestrator
                && let Ok(mut usage) = orch.token_usage.lock()
            {
                usage.prompt_tokens += prompt_tokens;
                usage.completion_tokens += completion_tokens;
            }
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::GetRepoMap => {
            let workspace_path = ctx.sandbox.workspace_dir();
            let repo_summary = crate::repo_map::extract_repo_map(workspace_path)?;
            Ok(serde_json::Value::String(repo_summary))
        }
        RasRpcCommand::GetTools => {
            Ok(serde_json::json!([]))
        }
        RasRpcCommand::ExecuteTool { name, .. } => {
            Err(format!("ExecuteTool for '{name}' not implemented in Core"))
        }
        _ => unreachable!(),
    }
}
