pub mod rad_extension {
    wasmtime::component::bindgen!({
        path: "wit/rad.wit",
        world: "rad-extension",
    });
}

pub mod rad_orchestrator {
    wasmtime::component::bindgen!({
        path: "wit/rad.wit",
        world: "rad-orchestrator",
        with: {
            "radcomp:extension/types": crate::wasm::bindings::rad_extension::radcomp::extension::types,
        }
    });
}

pub mod rad_security_guard {
    wasmtime::component::bindgen!({
        path: "wit/rad.wit",
        world: "rad-security-guard",
        with: {
            "radcomp:extension/types": crate::wasm::bindings::rad_extension::radcomp::extension::types,
        }
    });
}

pub mod rad_tool_provider {
    wasmtime::component::bindgen!({
        path: "wit/rad.wit",
        world: "rad-tool-provider",
        with: {
            "radcomp:extension/types": crate::wasm::bindings::rad_extension::radcomp::extension::types,
        }
    });
}


pub use rad_extension::radcomp::extension::types as wit;
pub use rad_extension::RadExtension;
pub use rad_extension::RadExtensionImports;


use rad_models::{
    Target as CoreTarget,
    TimeoutPolicy as CoreTimeoutPolicy,
    PendingToolCallInfo as CorePendingToolCallInfo,
    RasCoreEvent as CoreRasCoreEvent,
    RasRpcCommand as CoreRasRpcCommand,
};



impl From<wit::Target> for CoreTarget {
    fn from(t: wit::Target) -> Self {
        match t {
            wit::Target::Llm => CoreTarget::Llm,
            wit::Target::Process(p) => CoreTarget::Process(p),
        }
    }
}

impl From<CoreTarget> for wit::Target {
    fn from(t: CoreTarget) -> Self {
        match t {
            CoreTarget::Llm => wit::Target::Llm,
            CoreTarget::Process(p) => wit::Target::Process(p),
        }
    }
}

impl From<wit::TimeoutPolicy> for CoreTimeoutPolicy {
    fn from(tp: wit::TimeoutPolicy) -> Self {
        match tp {
            wit::TimeoutPolicy::Dynamic(p) => CoreTimeoutPolicy::Dynamic {
                heartbeat_timeout_ms: p.heartbeat_timeout_ms,
                max_silent_wait_ms: p.max_silent_wait_ms,
            },
            wit::TimeoutPolicy::Infinite => CoreTimeoutPolicy::Infinite,
        }
    }
}

impl From<CoreTimeoutPolicy> for wit::TimeoutPolicy {
    fn from(tp: CoreTimeoutPolicy) -> Self {
        match tp {
            CoreTimeoutPolicy::Dynamic { heartbeat_timeout_ms, max_silent_wait_ms } => {
                wit::TimeoutPolicy::Dynamic(wit::DynamicPolicy {
                    heartbeat_timeout_ms,
                    max_silent_wait_ms,
                })
            }
            CoreTimeoutPolicy::Infinite => wit::TimeoutPolicy::Infinite,
        }
    }
}

impl From<wit::PendingToolCallInfo> for CorePendingToolCallInfo {
    fn from(info: wit::PendingToolCallInfo) -> Self {
        CorePendingToolCallInfo {
            id: info.id,
            name: info.name,
            arguments: info.arguments,
            pgid: info.pgid,
        }
    }
}

impl From<CorePendingToolCallInfo> for wit::PendingToolCallInfo {
    fn from(info: CorePendingToolCallInfo) -> Self {
        wit::PendingToolCallInfo {
            id: info.id,
            name: info.name,
            arguments: info.arguments,
            pgid: info.pgid,
        }
    }
}

impl From<wit::RasRpcCommand> for CoreRasRpcCommand {
    fn from(cmd: wit::RasRpcCommand) -> Self {
        match cmd {
            wit::RasRpcCommand::FileRead(path) => CoreRasRpcCommand::FileRead { path: std::path::PathBuf::from(path) },
            wit::RasRpcCommand::FileWrite(payload) => CoreRasRpcCommand::FileWrite {
                path: std::path::PathBuf::from(payload.path),
                data: payload.data,
            },
            wit::RasRpcCommand::FileEditPatch(payload) => CoreRasRpcCommand::FileEditPatch {
                path: std::path::PathBuf::from(payload.path),
                diff: payload.diff,
            },
            wit::RasRpcCommand::SpawnBashProcess(cmd_str) => CoreRasRpcCommand::SpawnBashProcess { command: cmd_str },
            wit::RasRpcCommand::CreateNode(payload) => CoreRasRpcCommand::CreateNode {
                parent_id: payload.parent_id,
                node_type: payload.node_type,
            },
            wit::RasRpcCommand::SetNodeText(payload) => CoreRasRpcCommand::SetNodeText {
                node_id: payload.node_id,
                text: payload.text,
            },
            wit::RasRpcCommand::MergeNodes(payload) => CoreRasRpcCommand::MergeNodes {
                node_ids: payload.node_ids,
                summary_text: payload.summary_text,
            },
            wit::RasRpcCommand::DeleteNode(node_id) => CoreRasRpcCommand::DeleteNode { node_id },
            wit::RasRpcCommand::TakeSnapshot(payload) => CoreRasRpcCommand::TakeSnapshot {
                node_id: payload.node_id,
                target_paths: payload.target_paths.into_iter().map(std::path::PathBuf::from).collect(),
            },
            wit::RasRpcCommand::CheckoutSnapshot(node_id) => CoreRasRpcCommand::CheckoutSnapshot { node_id },
            wit::RasRpcCommand::OpenHttpStream(payload) => CoreRasRpcCommand::OpenHttpStream {
                url: payload.url,
                headers: payload.headers.into_iter().collect(),
                body: payload.body,
            },
            wit::RasRpcCommand::SetStreamTimeoutPolicy(payload) => CoreRasRpcCommand::SetStreamTimeoutPolicy {
                target: CoreTarget::from(payload.target),
                policy: CoreTimeoutPolicy::from(payload.policy),
            },
            wit::RasRpcCommand::WriteStdout(text) => CoreRasRpcCommand::WriteStdout { text },
            wit::RasRpcCommand::CompleteTask => CoreRasRpcCommand::CompleteTask,
            wit::RasRpcCommand::GetDag => CoreRasRpcCommand::GetDag,
            wit::RasRpcCommand::AskHumanApproval(prompt) => CoreRasRpcCommand::AskHumanApproval { prompt },
            wit::RasRpcCommand::ReportTokenUsage(payload) => CoreRasRpcCommand::ReportTokenUsage {
                prompt_tokens: payload.prompt_tokens,
                completion_tokens: payload.completion_tokens,
            },
            wit::RasRpcCommand::SpawnMcpServer(payload) => CoreRasRpcCommand::SpawnMcpServer {
                name: payload.name,
                command: payload.command,
                args: payload.args,
            },
            wit::RasRpcCommand::SendMcpRequest(payload) => CoreRasRpcCommand::SendMcpRequest {
                name: payload.name,
                message: payload.message,
            },
            wit::RasRpcCommand::GetRepoMap => CoreRasRpcCommand::GetRepoMap,
        }
    }
}

impl From<CoreRasCoreEvent> for wit::RasCoreEvent {
    fn from(event: CoreRasCoreEvent) -> Self {
        match event {
            CoreRasCoreEvent::HttpChunkReceived { chunk } => wit::RasCoreEvent::HttpChunkReceived(chunk),
            CoreRasCoreEvent::HttpErrorReceived { message } => wit::RasCoreEvent::HttpErrorReceived(message),
            CoreRasCoreEvent::ToolCallRequested { call_id, name, args } => {
                wit::RasCoreEvent::ToolCallRequested(wit::ToolCallRequest {
                    call_id,
                    name,
                    args: args.to_string(),
                })
            }
            CoreRasCoreEvent::ProcessSpawned { pgid, pid } => {
                wit::RasCoreEvent::ProcessSpawned(wit::ProcessSpawnInfo { pgid, pid })
            }
            CoreRasCoreEvent::ProcessStdout { pgid, data } => {
                wit::RasCoreEvent::ProcessStdout(wit::ProcessOutput { pgid, data })
            }
            CoreRasCoreEvent::ProcessStderr { pgid, data } => {
                wit::RasCoreEvent::ProcessStderr(wit::ProcessOutput { pgid, data })
            }
            CoreRasCoreEvent::ProcessExited { pgid, exit_code } => {
                wit::RasCoreEvent::ProcessExited(wit::ProcessExitInfo { pgid, exit_code })
            }
            CoreRasCoreEvent::FileChanged { path, change_type } => {
                wit::RasCoreEvent::FileChanged(wit::FileChangeInfo {
                    path: path.to_string_lossy().into_owned(),
                    change_type,
                })
            }
            CoreRasCoreEvent::StreamTimeout { target, duration_ms } => {
                wit::RasCoreEvent::StreamTimeout(wit::StreamTimeoutInfo { target, duration_ms })
            }
            CoreRasCoreEvent::HumanInputReceived { text } => wit::RasCoreEvent::HumanInputReceived(text),
            CoreRasCoreEvent::TaskCompleted => wit::RasCoreEvent::TaskCompleted,
            CoreRasCoreEvent::Rehydrate { active_calls } => {
                wit::RasCoreEvent::Rehydrate(active_calls.into_iter().map(wit::PendingToolCallInfo::from).collect())
            }
            CoreRasCoreEvent::McpResponse { name, message } => {
                wit::RasCoreEvent::McpResponse(wit::McpResponsePayload { name, message })
            }
        }
    }
}

impl From<CoreRasRpcCommand> for wit::RasRpcCommand {
    fn from(cmd: CoreRasRpcCommand) -> Self {
        match cmd {
            CoreRasRpcCommand::FileRead { path } => wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned()),
            CoreRasRpcCommand::FileWrite { path, data } => wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                path: path.to_string_lossy().into_owned(),
                data,
            }),
            CoreRasRpcCommand::FileEditPatch { path, diff } => wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                path: path.to_string_lossy().into_owned(),
                diff,
            }),
            CoreRasRpcCommand::SpawnBashProcess { command } => wit::RasRpcCommand::SpawnBashProcess(command),
            CoreRasRpcCommand::CreateNode { parent_id, node_type } => wit::RasRpcCommand::CreateNode(wit::CreateNodePayload {
                parent_id,
                node_type,
            }),
            CoreRasRpcCommand::SetNodeText { node_id, text } => wit::RasRpcCommand::SetNodeText(wit::SetNodeTextPayload {
                node_id,
                text,
            }),
            CoreRasRpcCommand::MergeNodes { node_ids, summary_text } => wit::RasRpcCommand::MergeNodes(wit::MergeNodesPayload {
                node_ids,
                summary_text,
            }),
            CoreRasRpcCommand::DeleteNode { node_id } => wit::RasRpcCommand::DeleteNode(node_id),
            CoreRasRpcCommand::TakeSnapshot { node_id, target_paths } => wit::RasRpcCommand::TakeSnapshot(wit::TakeSnapshotPayload {
                node_id,
                target_paths: target_paths.into_iter().map(|p| p.to_string_lossy().into_owned()).collect(),
            }),
            CoreRasRpcCommand::CheckoutSnapshot { node_id } => wit::RasRpcCommand::CheckoutSnapshot(node_id),
            CoreRasRpcCommand::OpenHttpStream { url, headers, body } => wit::RasRpcCommand::OpenHttpStream(wit::OpenHttpStreamPayload {
                url,
                headers: headers.into_iter().collect(),
                body,
            }),
            CoreRasRpcCommand::SetStreamTimeoutPolicy { target, policy } => wit::RasRpcCommand::SetStreamTimeoutPolicy(wit::SetStreamTimeoutPolicyPayload {
                target: wit::Target::from(target),
                policy: wit::TimeoutPolicy::from(policy),
            }),
            CoreRasRpcCommand::WriteStdout { text } => wit::RasRpcCommand::WriteStdout(text),
            CoreRasRpcCommand::CompleteTask => wit::RasRpcCommand::CompleteTask,
            CoreRasRpcCommand::GetDag => wit::RasRpcCommand::GetDag,
            CoreRasRpcCommand::AskHumanApproval { prompt } => wit::RasRpcCommand::AskHumanApproval(prompt),
            CoreRasRpcCommand::ReportTokenUsage { prompt_tokens, completion_tokens } => wit::RasRpcCommand::ReportTokenUsage(wit::ReportTokenUsagePayload {
                prompt_tokens,
                completion_tokens,
            }),
            CoreRasRpcCommand::SpawnMcpServer { name, command, args } => wit::RasRpcCommand::SpawnMcpServer(wit::SpawnMcpServerPayload {
                name,
                command,
                args,
            }),
            CoreRasRpcCommand::SendMcpRequest { name, message } => wit::RasRpcCommand::SendMcpRequest(wit::SendMcpRequestPayload {
                name,
                message,
            }),
            CoreRasRpcCommand::GetRepoMap => wit::RasRpcCommand::GetRepoMap,
        }
    }
}
