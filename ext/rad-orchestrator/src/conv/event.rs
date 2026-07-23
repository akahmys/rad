use crate::radcomp::extension::types as wit;
use rad_models::RasCoreEvent as CoreCoreEvent;

impl From<wit::RasCoreEvent> for CoreCoreEvent {
    fn from(event: wit::RasCoreEvent) -> Self {
        match event {
            wit::RasCoreEvent::HttpChunkReceived(chunk) => {
                CoreCoreEvent::HttpChunkReceived { chunk }
            }
            wit::RasCoreEvent::HttpErrorReceived(message) => {
                CoreCoreEvent::HttpErrorReceived { message }
            }
            wit::RasCoreEvent::ToolCallRequested(payload) => {
                let args_val =
                    serde_json::from_str(&payload.args).unwrap_or(serde_json::Value::Null);
                CoreCoreEvent::ToolCallRequested {
                    call_id: payload.call_id,
                    name: payload.name,
                    args: args_val,
                }
            }
            wit::RasCoreEvent::ProcessSpawned(payload) => CoreCoreEvent::ProcessSpawned {
                pgid: payload.pgid.to_string(),
                pid: payload.pid,
            },
            wit::RasCoreEvent::ProcessStdout(payload) => CoreCoreEvent::ProcessStdout {
                pgid: payload.pgid.to_string(),
                data: payload.data,
            },
            wit::RasCoreEvent::ProcessStderr(payload) => CoreCoreEvent::ProcessStderr {
                pgid: payload.pgid.to_string(),
                data: payload.data,
            },
            wit::RasCoreEvent::ProcessExited(payload) => CoreCoreEvent::ProcessExited {
                pgid: payload.pgid.to_string(),
                exit_code: payload.exit_code,
            },
            wit::RasCoreEvent::FileChanged(payload) => CoreCoreEvent::FileChanged {
                path: std::path::PathBuf::from(payload.path),
                change_type: payload.change_type,
            },
            wit::RasCoreEvent::StreamTimeout(payload) => CoreCoreEvent::StreamTimeout {
                target: payload.target,
                duration_ms: payload.duration_ms,
            },
            wit::RasCoreEvent::HumanInputReceived(text) => {
                CoreCoreEvent::HumanInputReceived { text }
            }
            wit::RasCoreEvent::TaskCompleted => CoreCoreEvent::TaskCompleted,
            wit::RasCoreEvent::Rehydrate(active_calls) => CoreCoreEvent::Rehydrate {
                active_calls: active_calls
                    .into_iter()
                    .map(rad_models::PendingToolCallInfo::from)
                    .collect(),
            },
            wit::RasCoreEvent::McpResponse(payload) => CoreCoreEvent::McpResponse {
                call_id: payload.call_id,
                name: payload.name,
                message: payload.message,
            },
            wit::RasCoreEvent::LlmConnectorEvent(event) => {
                CoreCoreEvent::LlmConnectorEvent { event }
            }
        }
    }
}
