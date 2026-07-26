// `RasRpcCommand` <-> WIT conversions, split out of `bindings.rs` to stay
// under the 300-line file limit.
use super::wit;
use rad_models::{
    RasRpcCommand as CoreRasRpcCommand, Target as CoreTarget, TimeoutPolicy as CoreTimeoutPolicy,
};

impl From<wit::RasRpcCommand> for CoreRasRpcCommand {
    fn from(cmd: wit::RasRpcCommand) -> Self {
        match cmd {
            wit::RasRpcCommand::FileRead(path) => CoreRasRpcCommand::FileRead {
                path: std::path::PathBuf::from(path),
            },
            wit::RasRpcCommand::FileWrite(payload) => CoreRasRpcCommand::FileWrite {
                path: std::path::PathBuf::from(payload.path),
                data: payload.data,
            },
            wit::RasRpcCommand::FileEditPatch(payload) => CoreRasRpcCommand::FileEditPatch {
                path: std::path::PathBuf::from(payload.path),
                diff: payload.diff,
            },
            wit::RasRpcCommand::SpawnBashProcess(cmd_str) => {
                CoreRasRpcCommand::SpawnBashProcess { command: cmd_str }
            }
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
                target_paths: payload
                    .target_paths
                    .into_iter()
                    .map(std::path::PathBuf::from)
                    .collect(),
            },
            wit::RasRpcCommand::CheckoutSnapshot(node_id) => {
                CoreRasRpcCommand::CheckoutSnapshot { node_id }
            }
            wit::RasRpcCommand::OpenHttpStream(payload) => CoreRasRpcCommand::OpenHttpStream {
                url: payload.url,
                headers: payload.headers.into_iter().collect(),
                body: payload.body,
            },
            wit::RasRpcCommand::SetStreamTimeoutPolicy(payload) => {
                CoreRasRpcCommand::SetStreamTimeoutPolicy {
                    target: CoreTarget::from(payload.target),
                    policy: CoreTimeoutPolicy::from(payload.policy),
                }
            }
            wit::RasRpcCommand::WriteStdout(text) => CoreRasRpcCommand::WriteStdout { text },
            wit::RasRpcCommand::CompleteTask => CoreRasRpcCommand::CompleteTask,
            wit::RasRpcCommand::GetDag => CoreRasRpcCommand::GetDag,
            wit::RasRpcCommand::AskHumanApproval(prompt) => {
                CoreRasRpcCommand::AskHumanApproval { prompt }
            }
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
            wit::RasRpcCommand::GetTools => CoreRasRpcCommand::GetTools,
            wit::RasRpcCommand::ExecuteTool(payload) => CoreRasRpcCommand::ExecuteTool {
                call_id: payload.call_id,
                name: payload.name,
                arguments: payload.arguments,
            },
            wit::RasRpcCommand::GenerateLlmStream(payload) => {
                CoreRasRpcCommand::GenerateLlmStream {
                    model: payload.model,
                    messages_json: payload.messages_json,
                    tools_json: payload.tools_json,
                }
            }
            wit::RasRpcCommand::CallExtension(payload) => CoreRasRpcCommand::CallExtension {
                extension_id: payload.extension_id,
                method: payload.method,
                arguments: payload.arguments,
            },
            wit::RasRpcCommand::LogTracedEvent(payload) => CoreRasRpcCommand::LogTracedEvent {
                trace_id: payload.trace_id,
                module: payload.module,
                message: payload.message,
            },
        }
    }
}

impl From<CoreRasRpcCommand> for wit::RasRpcCommand {
    fn from(cmd: CoreRasRpcCommand) -> Self {
        match cmd {
            CoreRasRpcCommand::FileRead { path } => {
                wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned())
            }
            CoreRasRpcCommand::FileWrite { path, data } => {
                wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                    path: path.to_string_lossy().into_owned(),
                    data,
                })
            }
            CoreRasRpcCommand::FileEditPatch { path, diff } => {
                wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                    path: path.to_string_lossy().into_owned(),
                    diff,
                })
            }
            CoreRasRpcCommand::SpawnBashProcess { command } => {
                wit::RasRpcCommand::SpawnBashProcess(command)
            }
            CoreRasRpcCommand::CreateNode {
                parent_id,
                node_type,
            } => wit::RasRpcCommand::CreateNode(wit::CreateNodePayload {
                parent_id,
                node_type,
            }),
            CoreRasRpcCommand::SetNodeText { node_id, text } => {
                wit::RasRpcCommand::SetNodeText(wit::SetNodeTextPayload { node_id, text })
            }
            CoreRasRpcCommand::MergeNodes {
                node_ids,
                summary_text,
            } => wit::RasRpcCommand::MergeNodes(wit::MergeNodesPayload {
                node_ids,
                summary_text,
            }),
            CoreRasRpcCommand::DeleteNode { node_id } => wit::RasRpcCommand::DeleteNode(node_id),
            CoreRasRpcCommand::TakeSnapshot {
                node_id,
                target_paths,
            } => wit::RasRpcCommand::TakeSnapshot(wit::TakeSnapshotPayload {
                node_id,
                target_paths: target_paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect(),
            }),
            CoreRasRpcCommand::CheckoutSnapshot { node_id } => {
                wit::RasRpcCommand::CheckoutSnapshot(node_id)
            }
            CoreRasRpcCommand::OpenHttpStream { url, headers, body } => {
                wit::RasRpcCommand::OpenHttpStream(wit::OpenHttpStreamPayload {
                    url,
                    headers: headers.into_iter().collect(),
                    body,
                })
            }
            CoreRasRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
                wit::RasRpcCommand::SetStreamTimeoutPolicy(wit::SetStreamTimeoutPolicyPayload {
                    target: wit::Target::from(target),
                    policy: wit::TimeoutPolicy::from(policy),
                })
            }
            CoreRasRpcCommand::WriteStdout { text } => wit::RasRpcCommand::WriteStdout(text),
            CoreRasRpcCommand::CompleteTask => wit::RasRpcCommand::CompleteTask,
            CoreRasRpcCommand::GetDag => wit::RasRpcCommand::GetDag,
            CoreRasRpcCommand::AskHumanApproval { prompt } => {
                wit::RasRpcCommand::AskHumanApproval(prompt)
            }
            CoreRasRpcCommand::ReportTokenUsage {
                prompt_tokens,
                completion_tokens,
            } => wit::RasRpcCommand::ReportTokenUsage(wit::ReportTokenUsagePayload {
                prompt_tokens,
                completion_tokens,
            }),
            CoreRasRpcCommand::SpawnMcpServer {
                name,
                command,
                args,
            } => wit::RasRpcCommand::SpawnMcpServer(wit::SpawnMcpServerPayload {
                name,
                command,
                args,
            }),
            CoreRasRpcCommand::SendMcpRequest { name, message } => {
                wit::RasRpcCommand::SendMcpRequest(wit::SendMcpRequestPayload { name, message })
            }
            CoreRasRpcCommand::GetRepoMap => wit::RasRpcCommand::GetRepoMap,
            CoreRasRpcCommand::GetTools => wit::RasRpcCommand::GetTools,
            CoreRasRpcCommand::ExecuteTool {
                call_id,
                name,
                arguments,
            } => wit::RasRpcCommand::ExecuteTool(wit::ExecuteToolPayload {
                call_id,
                name,
                arguments,
            }),
            CoreRasRpcCommand::OpenFile { path, writeable } => {
                if writeable {
                    wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                        path: path.to_string_lossy().into_owned(),
                        data: vec![],
                    })
                } else {
                    wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned())
                }
            }
            CoreRasRpcCommand::OpenProcess { command } => {
                wit::RasRpcCommand::SpawnBashProcess(command)
            }
            CoreRasRpcCommand::GenerateLlmStream {
                model,
                messages_json,
                tools_json,
            } => wit::RasRpcCommand::GenerateLlmStream(wit::GenerateLlmStreamPayload {
                model,
                messages_json,
                tools_json,
            }),
            CoreRasRpcCommand::CallExtension {
                extension_id,
                method,
                arguments,
            } => wit::RasRpcCommand::CallExtension(wit::CallExtensionPayload {
                extension_id,
                method,
                arguments,
            }),
            CoreRasRpcCommand::LogTracedEvent {
                trace_id,
                module,
                message,
            } => wit::RasRpcCommand::LogTracedEvent(wit::LogTracedEventPayload {
                trace_id,
                module,
                message,
            }),
        }
    }
}
