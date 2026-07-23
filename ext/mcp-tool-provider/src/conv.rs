use crate::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;

// Converters
impl From<wit::Target> for rad_models::Target {
    fn from(t: wit::Target) -> Self {
        match t {
            wit::Target::Llm => rad_models::Target::Llm,
            wit::Target::Process(p) => rad_models::Target::Process(p.to_string()),
        }
    }
}

impl From<wit::TimeoutPolicy> for rad_models::TimeoutPolicy {
    fn from(tp: wit::TimeoutPolicy) -> Self {
        match tp {
            wit::TimeoutPolicy::Dynamic(p) => rad_models::TimeoutPolicy::Dynamic {
                heartbeat_timeout_ms: p.heartbeat_timeout_ms,
                max_silent_wait_ms: p.max_silent_wait_ms,
            },
            wit::TimeoutPolicy::Infinite => rad_models::TimeoutPolicy::Infinite,
        }
    }
}

impl From<CoreRpcCommand> for wit::RasRpcCommand {
    fn from(cmd: CoreRpcCommand) -> Self {
        match cmd {
            CoreRpcCommand::FileRead { path } => {
                wit::RasRpcCommand::FileRead(path.to_string_lossy().to_string())
            }
            CoreRpcCommand::FileWrite { path, data } => {
                wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                    path: path.to_string_lossy().to_string(),
                    data,
                })
            }
            CoreRpcCommand::FileEditPatch { path, diff } => {
                wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                    path: path.to_string_lossy().to_string(),
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
                    .map(|p| p.to_string_lossy().to_string())
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
                    target: match target {
                        rad_models::Target::Llm => wit::Target::Llm,
                        rad_models::Target::Process(p) => {
                            wit::Target::Process(p.parse().unwrap_or(0))
                        }
                    },
                    policy: match policy {
                        rad_models::TimeoutPolicy::Dynamic {
                            heartbeat_timeout_ms,
                            max_silent_wait_ms,
                        } => wit::TimeoutPolicy::Dynamic(wit::DynamicPolicy {
                            heartbeat_timeout_ms,
                            max_silent_wait_ms,
                        }),
                        rad_models::TimeoutPolicy::Infinite => wit::TimeoutPolicy::Infinite,
                    },
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
            CoreRpcCommand::LogTracedEvent { .. } => {
                panic!("LogTracedEvent serialization arm")
            }
        }
    }
}
