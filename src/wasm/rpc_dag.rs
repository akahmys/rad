use super::rpc::RpcContext;
use crate::ipc::RasRpcCommand;

/// Handles all DAG (Directed Acyclic Graph) RPC commands.
///
/// # Errors
///
/// Returns an error if the underlying DAG operation fails.
pub fn handle_dag(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::CreateNode {
            parent_id,
            node_type,
        } => {
            let node_id = ctx.dag.create_node(parent_id, node_type)?;
            serde_json::to_value(node_id).map_err(|e| format!("Serialization error: {e}"))
        }
        RasRpcCommand::SetNodeText { node_id, text } => {
            ctx.dag.set_node_text(node_id, text)?;
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::MergeNodes {
            node_ids,
            summary_text,
        } => {
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
