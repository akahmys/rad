use crate::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;

fn convert_fs_wit_to_core(cmd: wit::RasRpcCommand) -> Option<CoreRpcCommand> {
    match cmd {
        wit::RasRpcCommand::FileRead(path) => Some(CoreRpcCommand::FileRead {
            path: std::path::PathBuf::from(path),
        }),
        wit::RasRpcCommand::FileWrite(payload) => Some(CoreRpcCommand::FileWrite {
            path: std::path::PathBuf::from(payload.path),
            data: payload.data,
        }),
        wit::RasRpcCommand::FileEditPatch(payload) => Some(CoreRpcCommand::FileEditPatch {
            path: std::path::PathBuf::from(payload.path),
            diff: payload.diff,
        }),
        _ => None,
    }
}

fn convert_dag_wit_to_core(cmd: wit::RasRpcCommand) -> Option<CoreRpcCommand> {
    match cmd {
        wit::RasRpcCommand::CreateNode(payload) => Some(CoreRpcCommand::CreateNode {
            parent_id: payload.parent_id,
            node_type: payload.node_type,
        }),
        wit::RasRpcCommand::SetNodeText(payload) => Some(CoreRpcCommand::SetNodeText {
            node_id: payload.node_id,
            text: payload.text,
        }),
        wit::RasRpcCommand::MergeNodes(payload) => Some(CoreRpcCommand::MergeNodes {
            node_ids: payload.node_ids,
            summary_text: payload.summary_text,
        }),
        wit::RasRpcCommand::DeleteNode(node_id) => Some(CoreRpcCommand::DeleteNode { node_id }),
        wit::RasRpcCommand::TakeSnapshot(payload) => Some(CoreRpcCommand::TakeSnapshot {
            node_id: payload.node_id,
            target_paths: payload
                .target_paths
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect(),
        }),
        wit::RasRpcCommand::CheckoutSnapshot(node_id) => {
            Some(CoreRpcCommand::CheckoutSnapshot { node_id })
        }
        wit::RasRpcCommand::GetDag => Some(CoreRpcCommand::GetDag),
        _ => None,
    }
}

fn convert_mcp_wit_to_core(cmd: wit::RasRpcCommand) -> Option<CoreRpcCommand> {
    match cmd {
        wit::RasRpcCommand::SpawnMcpServer(payload) => Some(CoreRpcCommand::SpawnMcpServer {
            name: payload.name,
            command: payload.command,
            args: payload.args,
        }),
        wit::RasRpcCommand::SendMcpRequest(payload) => Some(CoreRpcCommand::SendMcpRequest {
            name: payload.name,
            message: payload.message,
        }),
        wit::RasRpcCommand::GetTools => Some(CoreRpcCommand::GetTools),
        wit::RasRpcCommand::ExecuteTool(payload) => Some(CoreRpcCommand::ExecuteTool {
            call_id: payload.call_id,
            name: payload.name,
            arguments: payload.arguments,
        }),
        _ => None,
    }
}

impl From<wit::RasRpcCommand> for CoreRpcCommand {
    fn from(cmd: wit::RasRpcCommand) -> Self {
        if let Some(c) = convert_fs_wit_to_core(cmd.clone()) {
            return c;
        }
        if let Some(c) = convert_dag_wit_to_core(cmd.clone()) {
            return c;
        }
        if let Some(c) = convert_mcp_wit_to_core(cmd.clone()) {
            return c;
        }
        match cmd {
            wit::RasRpcCommand::SpawnBashProcess(cmd_str) => {
                CoreRpcCommand::SpawnBashProcess { command: cmd_str }
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
            wit::RasRpcCommand::AskHumanApproval(prompt) => {
                CoreRpcCommand::AskHumanApproval { prompt }
            }
            wit::RasRpcCommand::ReportTokenUsage(payload) => CoreRpcCommand::ReportTokenUsage {
                prompt_tokens: payload.prompt_tokens,
                completion_tokens: payload.completion_tokens,
            },
            wit::RasRpcCommand::GetRepoMap => CoreRpcCommand::GetRepoMap,
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
            wit::RasRpcCommand::LogTracedEvent(payload) => CoreRpcCommand::LogTracedEvent {
                trace_id: payload.trace_id,
                module: payload.module,
                message: payload.message,
            },
            _ => unreachable!(),
        }
    }
}

fn convert_fs_core_to_wit(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
    match cmd {
        CoreRpcCommand::FileRead { path } => {
            Some(wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned()))
        }
        CoreRpcCommand::FileWrite { path, data } => {
            Some(wit::RasRpcCommand::FileWrite(wit::FileWritePayload {
                path: path.to_string_lossy().into_owned(),
                data,
            }))
        }
        CoreRpcCommand::FileEditPatch { path, diff } => {
            Some(wit::RasRpcCommand::FileEditPatch(wit::FilePatchPayload {
                path: path.to_string_lossy().into_owned(),
                diff,
            }))
        }
        _ => None,
    }
}

fn convert_dag_core_to_wit(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
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
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        })),
        CoreRpcCommand::CheckoutSnapshot { node_id } => {
            Some(wit::RasRpcCommand::CheckoutSnapshot(node_id))
        }
        CoreRpcCommand::GetDag => Some(wit::RasRpcCommand::GetDag),
        _ => None,
    }
}

fn convert_mcp_core_to_wit(cmd: CoreRpcCommand) -> Option<wit::RasRpcCommand> {
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
        if let Some(w) = convert_fs_core_to_wit(cmd.clone()) {
            return w;
        }
        if let Some(w) = convert_dag_core_to_wit(cmd.clone()) {
            return w;
        }
        if let Some(w) = convert_mcp_core_to_wit(cmd.clone()) {
            return w;
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
                    target: wit::Target::from(target),
                    policy: wit::TimeoutPolicy::from(policy),
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
            CoreRpcCommand::LogTracedEvent {
                trace_id,
                module,
                message,
            } => wit::RasRpcCommand::LogTracedEvent(wit::LogTracedEventPayload {
                trace_id,
                module,
                message,
            }),
            _ => unreachable!(),
        }
    }
}
