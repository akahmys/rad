use std::collections::{HashMap, hash_map::RandomState};
use std::sync::{Arc, Mutex};

use crate::dag::Dag;
use crate::fs::FsSandbox;
use crate::ipc::RasRpcCommand;
use crate::process::{ProcessManager, RunningProcess};

/// Executes the given RPC command against physical systems.
///
/// # Errors
///
/// Returns an error if filesystem operations, process spawning, or DAG operations fail.
pub fn execute_rpc_command(
    cmd: &RasRpcCommand,
    sandbox: &FsSandbox,
    process_manager: &ProcessManager,
    dag: &Arc<Mutex<Dag>>,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    llm_timeout_policy: &Arc<Mutex<crate::ipc::TimeoutPolicy>>,
) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::FileRead { path } => {
            let data = sandbox.file_read(path)?;
            serde_json::to_value(serde_bytes::Bytes::new(&data))
                .map_err(|e| format!("Failed to serialize file_read result: {e}"))
        }
        RasRpcCommand::FileWrite { path, data } => {
            sandbox.file_write(path, data)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::FileEditPatch { path, diff } => {
            sandbox.file_edit_patch(path, diff)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SpawnBashProcess { command } => {
            spawn_bash_process_rpc(command, sandbox, process_manager, active_processes, event_tx)
        }
        RasRpcCommand::CreateNode { parent_id, node_type } => {
            let mut dag = dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
            let node_id = dag.create_node(parent_id, node_type)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetNodeText { node_id, text } => {
            let mut dag = dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
            dag.set_node_text(node_id, text)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::MergeNodes { node_ids, summary_text } => {
            let mut dag = dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
            let node_id = dag.merge_nodes(node_ids, summary_text)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::DeleteNode { node_id } => {
            let mut dag = dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
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
            let stream_id = crate::http::open_http_stream(
                url,
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
    }
}

fn spawn_bash_process_rpc(
    command: &str,
    sandbox: &FsSandbox,
    process_manager: &ProcessManager,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
) -> Result<serde_json::Value, String> {
    let running = process_manager.spawn_bash_process(command, Some(sandbox.workspace_dir()))?;
    let pgid = running.pgid().as_raw();

    let mut processes = active_processes.lock().map_err(|e| format!("Process lock error: {e}"))?;
    processes.insert(pgid, running);

    let event_tx_clone = event_tx.clone();
    let active_processes_clone = active_processes.clone();

    std::thread::spawn(move || {
        let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::ProcessSpawned { pgid, pid: pgid });
        loop {
            let done = monitor_single_process_tick(pgid, &active_processes_clone, &event_tx_clone);
            if done {
                if let Ok(mut procs) = active_processes_clone.lock() {
                    procs.remove(&pgid);
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    serde_json::to_value(pgid).map_err(|e| format!("Serialization error: {e}"))
}

fn monitor_single_process_tick(
    pgid: i32,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
) -> bool {
    let Ok(mut procs) = active_processes.lock() else { return true };
    let Some(proc) = procs.get_mut(&pgid) else { return true };

    let (stdout, stderr) = proc.read_available();
    if !stdout.is_empty() {
        let _ = event_tx.send(crate::ipc::RasCoreEvent::ProcessStdout { pgid, data: stdout });
    }
    if !stderr.is_empty() {
        let _ = event_tx.send(crate::ipc::RasCoreEvent::ProcessStderr { pgid, data: stderr });
    }

    match proc.child.try_wait() {
        Ok(Some(status)) => {
            let code = i32::try_from(status.exit_code()).ok();
            let _ = event_tx.send(crate::ipc::RasCoreEvent::ProcessExited { pgid, exit_code: code });
            proc.unregister_pgid();
            true
        }
        Ok(None) => {
            let policy = proc.timeout_policy.lock().map(|g| g.clone()).unwrap_or(crate::ipc::TimeoutPolicy::Infinite);
            let is_timeout = match policy {
                crate::ipc::TimeoutPolicy::Dynamic { heartbeat_timeout_ms, .. } => {
                    proc.last_activity.elapsed() > std::time::Duration::from_millis(heartbeat_timeout_ms)
                }
                crate::ipc::TimeoutPolicy::Infinite => false,
            };
            if is_timeout {
                proc.kill_group();
                let _ = event_tx.send(crate::ipc::RasCoreEvent::StreamTimeout {
                    target: format!("process_{pgid}"),
                    duration_ms: match policy {
                        crate::ipc::TimeoutPolicy::Dynamic { heartbeat_timeout_ms, .. } => heartbeat_timeout_ms,
                        crate::ipc::TimeoutPolicy::Infinite => 0,
                    },
                });
                let _ = event_tx.send(crate::ipc::RasCoreEvent::ProcessExited { pgid, exit_code: None });
                true
            } else {
                false
            }
        }
        Err(_) => {
            proc.kill_group();
            let _ = event_tx.send(crate::ipc::RasCoreEvent::ProcessExited { pgid, exit_code: None });
            true
        }
    }
}
