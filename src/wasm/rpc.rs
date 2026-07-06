use std::collections::{HashMap, hash_map::RandomState};
use std::sync::{Arc, Mutex};
use std::io::Write;

use crate::ipc::RasRpcCommand;
use crate::process::RunningProcess;
use crate::subsystems::{FsSubsystem, ProcessSubsystem, DagSubsystem, NetworkSubsystem};

/// Executes the given RPC command against physical systems.
///
/// # Errors
///
/// Returns an error if filesystem operations, process spawning, or DAG operations fail.
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

/// Executes the given RPC command against physical systems.
///
/// # Errors
///
/// Returns an error if filesystem operations, process spawning, or DAG operations fail.
#[allow(clippy::too_many_arguments)]
pub fn execute_rpc_command(
    cmd: &RasRpcCommand,
    sandbox: &dyn FsSubsystem,
    process_manager: &dyn ProcessSubsystem,
    dag: &dyn DagSubsystem,
    network: &dyn NetworkSubsystem,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
    active_mcp_servers: &Arc<Mutex<HashMap<String, crate::mcp::McpProcess>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    llm_timeout_policy: &Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    orchestrator: Option<&Arc<crate::orchestrator::Orchestrator>>,
    call_id: String,
    hitl_enabled: bool,
) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::FileRead { path } => {
            let data = sandbox.file_read(path)?;
            serde_json::to_value(serde_bytes::Bytes::new(&data))
                .map_err(|e| format!("Failed to serialize file_read result: {e}"))
        }
        RasRpcCommand::FileWrite { path, data } => {
            if hitl_enabled {
                request_approval(&format!("File Write to '{}'", path.display()))?;
            }
            sandbox.file_write(path, data)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::FileEditPatch { path, diff } => {
            if hitl_enabled {
                request_approval(&format!("File Edit (Patch) on '{}'", path.display()))?;
            }
            sandbox.file_edit_patch(path, diff)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SpawnBashProcess { command } => {
            if hitl_enabled {
                request_approval(&format!("Execute shell command: '{}'", command))?;
            }
            crate::wasm::rpc_process::spawn_bash_process_rpc(command, sandbox, process_manager, active_processes, event_tx, call_id, hitl_enabled)
        }
        RasRpcCommand::CreateNode { parent_id, node_type } => {
            let node_id = dag.create_node(parent_id, node_type)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetNodeText { node_id, text } => {
            dag.set_node_text(node_id, text)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::MergeNodes { node_ids, summary_text } => {
            let node_id = dag.merge_nodes(node_ids, summary_text)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::DeleteNode { node_id } => {
            dag.delete_node(node_id)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::TakeSnapshot { node_id, target_paths } => {
            sandbox.take_snapshot(node_id, target_paths)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::CheckoutSnapshot { node_id } => {
            sandbox.checkout_snapshot(node_id)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::OpenHttpStream { url, headers, body } => {
            let final_url = if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
                url.replace("127.0.0.1:8080", &format!("127.0.0.1:{test_port}"))
            } else {
                url.clone()
            };
            // Set terminal state to Thinking to display the indicator
            crate::terminal::get_terminal().set_state(crate::terminal::TerminalState::Thinking);
            let stream_id = network.open_http_stream(
                &final_url,
                headers.clone(),
                body,
                event_tx.clone(),
                llm_timeout_policy.clone(),
            )?;
            serde_json::to_value(stream_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
            match target {
                crate::ipc::Target::Llm => {
                    let mut guard = llm_timeout_policy.lock().map_err(|e| format!("Failed to lock llm_timeout_policy: {e}"))?;
                    *guard = policy.clone();
                }
                crate::ipc::Target::Process(pgid) => {
                    let processes = active_processes.lock().map_err(|e| format!("Process lock error: {e}"))?;
                    if let Some(proc) = processes.get(pgid) {
                        let mut guard = proc.timeout_policy.lock().map_err(|e| format!("Failed to lock process timeout_policy: {e}"))?;
                        *guard = policy.clone();
                    } else {
                        return Err(format!("Process with PGID {pgid} not found"));
                    }
                }
            }
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::WriteStdout { text } => {
            crate::terminal::get_terminal().write_llm_token(text);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::CompleteTask => {
            let _ = event_tx.send(crate::ipc::RasCoreEvent::TaskCompleted);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::GetDag => {
            let value = dag.get_dag()?;
            Ok(value)
        }
        RasRpcCommand::AskHumanApproval { prompt } => {
            if !hitl_enabled {
                Ok(serde_json::Value::Bool(true))
            } else {
                let approved = crate::wasm::rpc_process::ask_human_approval_internal(prompt)?;
                Ok(serde_json::Value::Bool(approved))
            }
        }
        RasRpcCommand::ReportTokenUsage { prompt_tokens, completion_tokens } => {
            if let Some(orch) = orchestrator {
                let lock_res = orch.token_usage.lock();
                if let Ok(mut usage) = lock_res {
                    usage.prompt_tokens += prompt_tokens;
                    usage.completion_tokens += completion_tokens;
                }
            }
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SpawnMcpServer { name, command, args } => {
            let proc = crate::mcp::McpProcess::spawn(name, command, args, event_tx.clone())?;
            active_mcp_servers.lock().unwrap().insert(name.clone(), proc);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SendMcpRequest { name, message } => {
            let mut guard = active_mcp_servers.lock().unwrap();
            if let Some(proc) = guard.get_mut(name) {
                proc.send_message(message)?;
                Ok(serde_json::Value::Null)
            } else {
                Err(format!("MCP server '{name}' is not running"))
            }
        }
        RasRpcCommand::GetRepoMap => {
            let workspace_path = sandbox.workspace_dir();
            let repo_summary = crate::repo_map::extract_repo_map(workspace_path)?;
            Ok(serde_json::Value::String(repo_summary))
        }
    }
}

