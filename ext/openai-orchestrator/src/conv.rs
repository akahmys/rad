use crate::radcomp::extension::types as wit;

impl From<wit::Target> for rad_models::Target {
    fn from(t: wit::Target) -> Self {
        match t {
            wit::Target::Llm => rad_models::Target::Llm,
            wit::Target::Process(p) => rad_models::Target::Process(p.to_string()),
        }
    }
}

impl From<rad_models::Target> for wit::Target {
    fn from(t: rad_models::Target) -> Self {
        match t {
            rad_models::Target::Llm => wit::Target::Llm,
            rad_models::Target::Process(p) => wit::Target::Process(p.parse().unwrap_or(0)),
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

impl From<rad_models::TimeoutPolicy> for wit::TimeoutPolicy {
    fn from(tp: rad_models::TimeoutPolicy) -> Self {
        match tp {
            rad_models::TimeoutPolicy::Dynamic {
                heartbeat_timeout_ms,
                max_silent_wait_ms,
            } => wit::TimeoutPolicy::Dynamic(wit::DynamicPolicy {
                heartbeat_timeout_ms,
                max_silent_wait_ms,
            }),
            rad_models::TimeoutPolicy::Infinite => wit::TimeoutPolicy::Infinite,
        }
    }
}

impl From<wit::PendingToolCallInfo> for rad_models::PendingToolCallInfo {
    fn from(info: wit::PendingToolCallInfo) -> Self {
        rad_models::PendingToolCallInfo {
            id: info.id,
            name: info.name,
            arguments: info.arguments,
            pgid: info.pgid.map(|n| n.to_string()),
        }
    }
}

impl From<rad_models::PendingToolCallInfo> for wit::PendingToolCallInfo {
    fn from(info: rad_models::PendingToolCallInfo) -> Self {
        wit::PendingToolCallInfo {
            id: info.id,
            name: info.name,
            arguments: info.arguments,
            pgid: info.pgid.and_then(|s| s.parse().ok()),
        }
    }
}

pub mod event;
pub mod rpc;
