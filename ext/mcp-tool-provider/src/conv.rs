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

fn convert_fs_cmd(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
    match cmd {
        CoreRpcCommand::FileRead { path } => {
            Some(wit::RasRpcCommand::FileRead(path.to_string_lossy().to_string()))
        }
        CoreRpcCommand::FileWrite { path, data } => {
            Some(wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                path: path.to_string_lossy().to_string(),
                data,
            }))
        }
        CoreRpcCommand::FileEditPatch { path, diff } => {
            Some(wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                path: path.to_string_lossy().to_string(),
                diff,
            }))
        }
        _ => None,
    }
}

fn convert_dag_cmd(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
    match cmd {
        CoreRpcCommand::CreateNode {
            parent_id,
            node_type,
        } => Some(wit::RasRpcCommand::CreateNode(wit::CreateNodePayload {
            parent_id,
            node_type,
        })),
        CoreRpcCommand::SetNodeText { node_id, text } => {
            Some(wit::RasRpcCommand::SetNodeText(wit::SetNodeTextPayload { node_id, text }))
        }
        CoreRpcCommand::MergeNodes {
            node_ids,
            summary_text,
        } => Some(wit::RasRpcCommand::MergeNodes(wit::MergeNodesPayload {
            node_ids,
            summary_text,
        })),
        CoreRpcCommand::DeleteNode { node_id } => Some(wit::RasRpcCommand::DeleteNode(node_id)),
        CoreRpcCommand::TakeSnapshot {
            node_id,
            target_paths,
        } => Some(wit::RasRpcCommand::TakeSnapshot(wit::TakeSnapshotPayload {
            node_id,
            target_paths: target_paths
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
        })),
        CoreRpcCommand::CheckoutSnapshot { node_id } => {
            Some(wit::RasRpcCommand::CheckoutSnapshot(node_id))
        }
        CoreRpcCommand::GetDag => Some(wit::RasRpcCommand::GetDag),
        _ => None,
    }
}

fn convert_mcp_cmd(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
    match cmd {
        CoreRpcCommand::SpawnMcpServer {
            name,
            command,
            args,
        } => Some(wit::RasRpcCommand::SpawnMcpServer(wit::SpawnMcpServerPayload {
            name,
            command,
            args,
        })),
        CoreRpcCommand::SendMcpRequest { name, message } => {
            Some(wit::RasRpcCommand::SendMcpRequest(wit::SendMcpRequestPayload { name, message }))
        }
        CoreRpcCommand::GetTools => Some(wit::RasRpcCommand::GetTools),
        CoreRpcCommand::ExecuteTool {
            call_id,
            name,
            arguments,
        } => Some(wit::RasRpcCommand::ExecuteTool(wit::ExecuteToolPayload {
            call_id,
            name,
            arguments,
        })),
        _ => None,
    }
}

impl From<CoreRpcCommand> for wit::RasRpcCommand {
    fn from(cmd: CoreRpcCommand) -> Self {
        if let Some(res) = convert_fs_cmd(cmd.clone()) {
            return res;
        }
        if let Some(res) = convert_dag_cmd(cmd.clone()) {
            return res;
        }
        if let Some(res) = convert_mcp_cmd(cmd.clone()) {
            return res;
        }
        match cmd {
            CoreRpcCommand::SpawnBashProcess { command } => {
                wit::RasRpcCommand::SpawnBashProcess(command)
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
            CoreRpcCommand::GetRepoMap => wit::RasRpcCommand::GetRepoMap,
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
            _ => unreachable!(),
        }
    }
}
