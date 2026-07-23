#![deny(clippy::pedantic)]
#![allow(
    unsafe_op_in_unsafe_fn,
    clippy::same_length_and_capacity,
    clippy::collapsible_match
)]

wit_bindgen::generate!({
    path: "../../wit/rad.wit",
    world: "rad-security-guard",
});

use self::radcomp::extension::types as wit;
use rad_models::RasRpcCommand as CoreRpcCommand;

struct SecurityGuardImpl;

impl Guest for SecurityGuardImpl {
    fn verify_rpc(command: wit::RasRpcCommand) -> bool {
        let rpc_cmd = CoreRpcCommand::from(command);
        match rpc_cmd {
            CoreRpcCommand::FileWrite { path, .. } => {
                if path.to_string_lossy().contains("blocked.txt") {
                    return false;
                }
            }
            CoreRpcCommand::SpawnBashProcess { command } => {
                if command.contains("blocked_command") || command.contains("blocked.txt") {
                    return false;
                }
            }
            CoreRpcCommand::ExecuteTool { arguments, .. } => {
                if arguments.contains("blocked.txt") || arguments.contains("blocked_command") {
                    return false;
                }
            }
            _ => {}
        }
        true
    }
}

export!(SecurityGuardImpl);

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
            wit::RasRpcCommand::LogTracedEvent(payload) => CoreRpcCommand::LogTracedEvent {
                trace_id: payload.trace_id,
                module: payload.module,
                message: payload.message,
            },
        }
    }
}
