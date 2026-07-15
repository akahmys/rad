use crate::wasm::bindings::wit;
use rad_models::RasCoreEvent as CoreRasCoreEvent;

impl From<CoreRasCoreEvent> for wit::RasCoreEvent {
    fn from(event: CoreRasCoreEvent) -> Self {
        match event {
            CoreRasCoreEvent::HttpChunkReceived { chunk } => {
                wit::RasCoreEvent::HttpChunkReceived(chunk)
            }
            CoreRasCoreEvent::HttpErrorReceived { message } => {
                wit::RasCoreEvent::HttpErrorReceived(message)
            }
            CoreRasCoreEvent::ToolCallRequested {
                call_id,
                name,
                args,
            } => wit::RasCoreEvent::ToolCallRequested(wit::ToolCallRequest {
                call_id,
                name,
                args: args.to_string(),
            }),
            CoreRasCoreEvent::ProcessSpawned { pgid, pid } => {
                wit::RasCoreEvent::ProcessSpawned(wit::ProcessSpawnInfo {
                    pgid: pgid.parse().unwrap_or(0),
                    pid,
                })
            }
            CoreRasCoreEvent::ProcessStdout { pgid, data } => {
                wit::RasCoreEvent::ProcessStdout(wit::ProcessOutput {
                    pgid: pgid.parse().unwrap_or(0),
                    data,
                })
            }
            CoreRasCoreEvent::ProcessStderr { pgid, data } => {
                wit::RasCoreEvent::ProcessStderr(wit::ProcessOutput {
                    pgid: pgid.parse().unwrap_or(0),
                    data,
                })
            }
            CoreRasCoreEvent::ProcessExited { pgid, exit_code } => {
                wit::RasCoreEvent::ProcessExited(wit::ProcessExitInfo {
                    pgid: pgid.parse().unwrap_or(0),
                    exit_code,
                })
            }
            CoreRasCoreEvent::FileChanged { path, change_type } => {
                wit::RasCoreEvent::FileChanged(wit::FileChangeInfo {
                    path: path.to_string_lossy().into_owned(),
                    change_type,
                })
            }
            CoreRasCoreEvent::StreamTimeout {
                target,
                duration_ms,
            } => wit::RasCoreEvent::StreamTimeout(wit::StreamTimeoutInfo {
                target,
                duration_ms,
            }),
            CoreRasCoreEvent::HumanInputReceived { text } => {
                wit::RasCoreEvent::HumanInputReceived(text)
            }
            CoreRasCoreEvent::TaskCompleted => wit::RasCoreEvent::TaskCompleted,
            CoreRasCoreEvent::Rehydrate { active_calls } => wit::RasCoreEvent::Rehydrate(
                active_calls
                    .into_iter()
                    .map(wit::PendingToolCallInfo::from)
                    .collect(),
            ),
            CoreRasCoreEvent::McpResponse {
                call_id,
                name,
                message,
            } => wit::RasCoreEvent::McpResponse(wit::McpResponsePayload {
                call_id,
                name,
                message,
            }),
        }
    }
}
