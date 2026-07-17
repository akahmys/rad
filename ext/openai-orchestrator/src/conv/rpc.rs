use crate::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;

impl From<wit::RasRpcCommand> for CoreRpcCommand {
    fn from(cmd: wit::RasRpcCommand) -> Self {
        match cmd {
            wit::RasRpcCommand::FileRead(path) => CoreRpcCommand::FileRead {
                path: std::path::PathBuf::from(path),
            },
            wit::RasRpcCommand::FileWrite(payload) => CoreRpcCommand::FileWrite {
                path: std::path::PathBuf::from(payload.path),
                data: payload.data,
            },
            wit::RasRpcCommand::FileEditPatch(payload) => CoreRpcCommand::FileEditPatch {
                path: std::path::PathBuf::from(payload.path),
                diff: payload.diff,
            },
            wit::RasRpcCommand::SpawnBashProcess(cmd_str) => {
                CoreRpcCommand::SpawnBashProcess { command: cmd_str }
            }
            wit::RasRpcCommand::CreateNode(payload) => CoreRpcCommand::CreateNode {
                parent_id: payload.parent_id,
                node_type: payload.node_type,
            },
            wit::RasRpcCommand::SetNodeText(payload) => CoreRpcCommand::SetNodeText {
                node_id: payload.node_id,
                text: payload.text,
            },
            wit::RasRpcCommand::MergeNodes(payload) => CoreRpcCommand::MergeNodes {
                node_ids: payload.node_ids,
                summary_text: payload.summary_text,
            },
            wit::RasRpcCommand::DeleteNode(node_id) => CoreRpcCommand::DeleteNode { node_id },
            wit::RasRpcCommand::TakeSnapshot(payload) => CoreRpcCommand::TakeSnapshot {
                node_id: payload.node_id,
                target_paths: payload
                    .target_paths
                    .into_iter()
                    .map(std::path::PathBuf::from)
                    .collect(),
            },
            wit::RasRpcCommand::CheckoutSnapshot(node_id) => {
                CoreRpcCommand::CheckoutSnapshot { node_id }
            }
            wit::RasRpcCommand::OpenHttpStream(payload) => CoreRpcCommand::OpenHttpStream {
                url: payload.url,
                headers: payload.headers.into_iter().collect(),
                body: payload.body,
            },
            wit::RasRpcCommand::SetStreamTimeoutPolicy(payload) => {
                CoreRpcCommand::SetStreamTimeoutPolicy {
                    target: rad_models::Target::from(payload.target),
                    policy: rad_models::TimeoutPolicy::from(payload.policy),
                }
            }
            wit::RasRpcCommand::WriteStdout(text) => CoreRpcCommand::WriteStdout { text },
            wit::RasRpcCommand::CompleteTask => CoreRpcCommand::CompleteTask,
            wit::RasRpcCommand::GetDag => CoreRpcCommand::GetDag,
            wit::RasRpcCommand::AskHumanApproval(prompt) => {
                CoreRpcCommand::AskHumanApproval { prompt }
            }
            wit::RasRpcCommand::ReportTokenUsage(payload) => CoreRpcCommand::ReportTokenUsage {
                prompt_tokens: payload.prompt_tokens,
                completion_tokens: payload.completion_tokens,
            },
            wit::RasRpcCommand::SpawnMcpServer(payload) => CoreRpcCommand::SpawnMcpServer {
                name: payload.name,
                command: payload.command,
                args: payload.args,
            },
            wit::RasRpcCommand::SendMcpRequest(payload) => CoreRpcCommand::SendMcpRequest {
                name: payload.name,
                message: payload.message,
            },
            wit::RasRpcCommand::GetRepoMap => CoreRpcCommand::GetRepoMap,
            wit::RasRpcCommand::GetTools => CoreRpcCommand::GetTools,
            wit::RasRpcCommand::ExecuteTool(payload) => CoreRpcCommand::ExecuteTool {
                call_id: payload.call_id,
                name: payload.name,
                arguments: payload.arguments,
            },
            wit::RasRpcCommand::GenerateLlmStream(payload) => CoreRpcCommand::GenerateLlmStream {
                model: payload.model,
                messages_json: payload.messages_json,
                tools_json: payload.tools_json,
            },
            wit::RasRpcCommand::CallExtension(payload) => CoreRpcCommand::CallExtension {
                extension_id: payload.extension_id,
                method: payload.method,
                arguments: payload.arguments,
            },
        }
    }
}

impl From<CoreRpcCommand> for wit::RasRpcCommand {
    fn from(cmd: CoreRpcCommand) -> Self {
        match cmd {
            CoreRpcCommand::FileRead { path } => {
                wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned())
            }
            CoreRpcCommand::FileWrite { path, data } => {
                wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                    path: path.to_string_lossy().into_owned(),
                    data,
                })
            }
            CoreRpcCommand::FileEditPatch { path, diff } => {
                wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                    path: path.to_string_lossy().into_owned(),
                    diff,
                })
            }
            CoreRpcCommand::SpawnBashProcess { command } => {
                wit::RasRpcCommand::SpawnBashProcess(command)
            }
            CoreRpcCommand::CreateNode {
                parent_id,
                node_type,
            } => wit::RasRpcCommand::CreateNode(wit::CreateNodePayload {
                parent_id,
                node_type,
            }),
            CoreRpcCommand::SetNodeText { node_id, text } => {
                wit::RasRpcCommand::SetNodeText(wit::SetNodeTextPayload { node_id, text })
            }
            CoreRpcCommand::MergeNodes {
                node_ids,
                summary_text,
            } => wit::RasRpcCommand::MergeNodes(wit::MergeNodesPayload {
                node_ids,
                summary_text,
            }),
            CoreRpcCommand::DeleteNode { node_id } => wit::RasRpcCommand::DeleteNode(node_id),
            CoreRpcCommand::TakeSnapshot {
                node_id,
                target_paths,
            } => wit::RasRpcCommand::TakeSnapshot(wit::TakeSnapshotPayload {
                node_id,
                target_paths: target_paths
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect(),
            }),
            CoreRpcCommand::CheckoutSnapshot { node_id } => {
                wit::RasRpcCommand::CheckoutSnapshot(node_id)
            }
            CoreRpcCommand::OpenHttpStream { url, headers, body } => {
                wit::RasRpcCommand::OpenHttpStream(wit::OpenHttpStreamPayload {
                    url,
                    headers: headers.into_iter().collect(),
                    body,
                })
            }
            CoreRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
                wit::RasRpcCommand::SetStreamTimeoutPolicy(wit::SetStreamTimeoutPolicyPayload {
                    target: wit::Target::from(target),
                    policy: wit::TimeoutPolicy::from(policy),
                })
            }
            CoreRpcCommand::WriteStdout { text } => wit::RasRpcCommand::WriteStdout(text),
            CoreRpcCommand::CompleteTask => wit::RasRpcCommand::CompleteTask,
            CoreRpcCommand::GetDag => wit::RasRpcCommand::GetDag,
            CoreRpcCommand::AskHumanApproval { prompt } => {
                wit::RasRpcCommand::AskHumanApproval(prompt)
            }
            CoreRpcCommand::ReportTokenUsage {
                prompt_tokens,
                completion_tokens,
            } => wit::RasRpcCommand::ReportTokenUsage(wit::ReportTokenUsagePayload {
                prompt_tokens,
                completion_tokens,
            }),
            CoreRpcCommand::SpawnMcpServer {
                name,
                command,
                args,
            } => wit::RasRpcCommand::SpawnMcpServer(wit::SpawnMcpServerPayload {
                name,
                command,
                args,
            }),
            CoreRpcCommand::SendMcpRequest { name, message } => {
                wit::RasRpcCommand::SendMcpRequest(wit::SendMcpRequestPayload { name, message })
            }
            CoreRpcCommand::GetRepoMap => wit::RasRpcCommand::GetRepoMap,
            CoreRpcCommand::GetTools => wit::RasRpcCommand::GetTools,
            CoreRpcCommand::ExecuteTool {
                call_id,
                name,
                arguments,
            } => wit::RasRpcCommand::ExecuteTool(wit::ExecuteToolPayload {
                call_id,
                name,
                arguments,
            }),
            CoreRpcCommand::OpenFile { .. } | CoreRpcCommand::OpenProcess { .. } => {
                panic!("OpenFile and OpenProcess are now directly imported capabilities")
            }
            CoreRpcCommand::GenerateLlmStream {
                model,
                messages_json,
                tools_json,
            } => wit::RasRpcCommand::GenerateLlmStream(wit::GenerateLlmStreamPayload {
                model,
                messages_json,
                tools_json,
            }),
            CoreRpcCommand::CallExtension {
                extension_id,
                method,
                arguments,
            } => wit::RasRpcCommand::CallExtension(wit::CallExtensionPayload {
                extension_id,
                method,
                arguments,
            }),
        }
    }
}
