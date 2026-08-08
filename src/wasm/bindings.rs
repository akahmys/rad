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

pub mod rad_kernel {
    // The migration's new surface, live alongside the existing one. A fourth,
    // unrelated package: adding it cannot change the type of anything the six
    // current extensions import, so none of them break (ARCHITECTURE-NEXT.md
    // §2, §9.1). Nothing implements this world yet — registering the bindings
    // first is what proves that claim on the real build rather than a probe.
    wasmtime::component::bindgen!({
        path: "wit/kernel/kernel.wit",
        world: "module",
        additional_derives: [serde::Serialize, serde::Deserialize],
        // The host types behind the resources. Without these, bindgen generates
        // empty placeholder types and every implementation has to invent a
        // mapping of its own; naming them here makes the resource table hold
        // the real thing.
        with: {
            "rad:kernel/types/process": crate::kernel::proc::KernelProcess,
            "rad:kernel/types/byte-stream": crate::kernel::stream::KernelStream,
        },
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
