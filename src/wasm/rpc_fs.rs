use super::rpc::RpcContext;
use super::rpc_process::request_approval;
use crate::ipc::RasRpcCommand;

/// Handles all file system RPC commands.
///
/// # Errors
///
/// Returns an error if the underlying filesystem operation fails.
pub fn handle_fs(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::FileRead { path } => {
            let expanded = crate::config::expand_tilde(&path.to_string_lossy());
            let data = ctx.sandbox.file_read(&expanded)?;
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
        RasRpcCommand::TakeSnapshot {
            node_id,
            target_paths,
        } => {
            ctx.sandbox.take_snapshot(node_id, target_paths)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::CheckoutSnapshot { node_id } => {
            ctx.sandbox.checkout_snapshot(node_id)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::GetRepoMap => {
            let workspace_path = ctx.sandbox.workspace_dir();
            let repo_summary = crate::repo_map::extract_repo_map(workspace_path)?;
            Ok(serde_json::Value::String(repo_summary))
        }
        _ => unreachable!(),
    }
}
