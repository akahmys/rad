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
            let running = process_manager.spawn_bash_process(command)?;
            let pgid = running.pgid().as_raw();

            let mut processes = active_processes.lock().map_err(|e| format!("Process lock error: {e}"))?;
            processes.insert(pgid, running);

            serde_json::to_value(pgid).map_err(|e| format!("Serialization error: {e}"))
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
        RasRpcCommand::OpenHttpStream { .. } => {
            serde_json::to_value("stream_dummy_id").map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetStreamTimeoutPolicy { .. } => {
            Ok(serde_json::Value::Null)
        }
    }
}
