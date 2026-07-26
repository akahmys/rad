pub mod rad_extension {
    wasmtime::component::bindgen!({
        path: "wit/rad.wit",
        world: "rad-extension",
        with: {
            "radcomp:extension/types/stream-handle": crate::wasm::HostStream,
            "radcomp:extension/types/file-handle": crate::wasm::HostFile,
            "radcomp:extension/types/execution-handle": crate::wasm::HostExecution,
        }
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

pub mod rad_llm_connector {
    wasmtime::component::bindgen!({
        path: "wit/llm-connector.wit",
        world: "llm-connector",
        additional_derives: [serde::Serialize, serde::Deserialize],
        with: {
            "radcomp:connector/types/stream-handle": crate::wasm::HostStream,
        }
    });
}

pub mod rad_context_tools {
    wasmtime::component::bindgen!({
        path: "wit/context-tools.wit",
        world: "context-tools-extension",
        additional_derives: [serde::Serialize, serde::Deserialize],
    });
}


pub use rad_extension::RadExtension;
pub use rad_extension::RadExtensionImports;
pub use rad_extension::radcomp::extension::types as wit;

// RasRpcCommand <-> WIT conversions live in bindings/rpc_command.rs to keep
// this file under the 300-line limit.
mod rpc_command;

use rad_models::{
    PendingToolCallInfo as CorePendingToolCallInfo, Target as CoreTarget,
    TimeoutPolicy as CoreTimeoutPolicy,
};

impl From<wit::Target> for CoreTarget {
    fn from(t: wit::Target) -> Self {
        match t {
            wit::Target::Llm => CoreTarget::Llm,
            wit::Target::Process(p) => CoreTarget::Process(p.to_string()),
        }
    }
}

impl From<CoreTarget> for wit::Target {
    fn from(t: CoreTarget) -> Self {
        match t {
            CoreTarget::Llm => wit::Target::Llm,
            CoreTarget::Process(p) => wit::Target::Process(p.parse().unwrap_or(0)),
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
            CoreTimeoutPolicy::Dynamic {
                heartbeat_timeout_ms,
                max_silent_wait_ms,
            } => wit::TimeoutPolicy::Dynamic(wit::DynamicPolicy {
                heartbeat_timeout_ms,
                max_silent_wait_ms,
            }),
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
            pgid: info.pgid.map(|p| p.to_string()),
        }
    }
}

impl From<CorePendingToolCallInfo> for wit::PendingToolCallInfo {
    fn from(info: CorePendingToolCallInfo) -> Self {
        wit::PendingToolCallInfo {
            id: info.id,
            name: info.name,
            arguments: info.arguments,
            pgid: info.pgid.map(|p| p.parse().unwrap_or(0)),
        }
    }
}

