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
        path: "wit/connector/llm-connector.wit",
        world: "llm-connector",
        additional_derives: [serde::Serialize, serde::Deserialize],
        with: {
            "radcomp:connector/types/stream-handle": crate::wasm::HostStream,
        }
    });
}

pub mod rad_context_tools {
    // `path` is the directory, not a single file: context-tools.wit shares
    // `package radcomp:extension` with rad.wit (see that file's docs), so
    // resolving it needs both files. `llm-connector.wit` lives in its own
    // `wit/connector/` subdirectory specifically so it doesn't collide with
    // this directory scan (it's a different, unrelated package).
    wasmtime::component::bindgen!({
        path: "wit",
        world: "context-tools-extension",
        additional_derives: [serde::Serialize, serde::Deserialize],
        with: {
            "radcomp:extension/types": crate::wasm::bindings::rad_extension::radcomp::extension::types,
        }
    });
}

pub use rad_extension::RadExtension;
pub use rad_extension::RadExtensionImports;
pub use rad_extension::radcomp::extension::types as wit;

// RasRpcCommand <-> WIT conversions live in bindings/rpc_command.rs to keep
// this file under the 300-line limit.
mod rpc_command;

// Target/TimeoutPolicy WIT conversions are generated in bindings/rpc_command.rs
// via the shared `rad_models::impl_rpc_target_*`/`impl_rpc_timeout_policy_*`
// macros (see that file and `models/src/rpc_conversion.rs`).
use rad_models::PendingToolCallInfo as CorePendingToolCallInfo;

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
