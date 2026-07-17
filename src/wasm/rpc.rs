use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::ipc::RasRpcCommand;
use crate::process::RunningProcess;
use crate::subsystems::{DagSubsystem, FsSubsystem, NetworkSubsystem, ProcessSubsystem};

/// Shared context passed to each RPC handler category.
pub struct RpcContext<'a> {
    pub sandbox: &'a dyn FsSubsystem,
    pub process_manager: &'a dyn ProcessSubsystem,
    pub dag: &'a dyn DagSubsystem,
    pub network: &'a dyn NetworkSubsystem,
    pub active_processes: &'a Arc<Mutex<HashMap<String, RunningProcess>>>,
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
    active_processes: &Arc<Mutex<HashMap<String, RunningProcess>>>,
    active_mcp_servers: &Arc<Mutex<HashMap<String, crate::mcp::McpProcess>>>,
    event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
    llm_timeout_policy: &Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    orchestrator: Option<&Arc<crate::orchestrator::Orchestrator>>,
    call_id: String,
    hitl_enabled: bool,
) -> Result<serde_json::Value, String> {
    let ctx = RpcContext {
        sandbox,
        process_manager,
        dag,
        network,
        active_processes,
        active_mcp_servers,
        event_tx,
        llm_timeout_policy,
        orchestrator,
        call_id,
        hitl_enabled,
    };
    match cmd {
        RasRpcCommand::FileRead { .. }
        | RasRpcCommand::FileWrite { .. }
        | RasRpcCommand::FileEditPatch { .. }
        | RasRpcCommand::TakeSnapshot { .. }
        | RasRpcCommand::CheckoutSnapshot { .. }
        | RasRpcCommand::OpenFile { .. }
        | RasRpcCommand::GetRepoMap => super::rpc_fs::handle_fs(cmd, &ctx),

        RasRpcCommand::CreateNode { .. }
        | RasRpcCommand::SetNodeText { .. }
        | RasRpcCommand::MergeNodes { .. }
        | RasRpcCommand::DeleteNode { .. }
        | RasRpcCommand::GetDag => super::rpc_dag::handle_dag(cmd, &ctx),

        RasRpcCommand::SpawnBashProcess { .. }
        | RasRpcCommand::SpawnMcpServer { .. }
        | RasRpcCommand::OpenProcess { .. }
        | RasRpcCommand::SendMcpRequest { .. } => super::rpc_process::handle_process(cmd, &ctx),

        RasRpcCommand::OpenHttpStream { .. } | RasRpcCommand::SetStreamTimeoutPolicy { .. } => {
            super::rpc_network::handle_network(cmd, &ctx)
        }
        RasRpcCommand::WriteStdout { .. } => super::rpc_terminal::handle_terminal(cmd, &ctx),

        RasRpcCommand::CompleteTask
        | RasRpcCommand::AskHumanApproval { .. }
        | RasRpcCommand::ReportTokenUsage { .. }
        | RasRpcCommand::GetTools
        | RasRpcCommand::ExecuteTool { .. }
        | RasRpcCommand::GenerateLlmStream { .. }
        | RasRpcCommand::CallExtension { .. } => super::rpc_meta::handle_meta(cmd, &ctx),
    }
}
