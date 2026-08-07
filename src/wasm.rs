use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use wasmtime::Store;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiView};

use crate::config::PermissionConfig;
use crate::ipc::RasCoreEvent;
use crate::process::RunningProcess;
use crate::subsystems::{DagSubsystem, FsSubsystem, NetworkSubsystem, ProcessSubsystem};

pub mod bindings;
pub mod bindings_event;
pub mod imports;
mod imports_delegate;
mod imports_http;
mod imports_process;
mod imports_resources;
mod imports_resources_exec;
mod imports_resources_file;
mod imports_rpc;
mod imports_tool;
pub mod loader;
pub mod permissions;
pub mod rpc;
pub mod rpc_dag;
pub mod rpc_fs;
pub mod rpc_meta;
mod rpc_meta_llm_connector;
mod rpc_meta_llm_fallback;
pub mod rpc_network;
pub mod rpc_process;
pub mod rpc_terminal;

#[cfg(test)]
mod tests;

pub enum HostStream {
    File(std::fs::File),
    PipeReader(Mutex<std::sync::mpsc::Receiver<Vec<u8>>>),
    /// Like `PipeReader`, but the producer can report a mid-stream failure
    /// (connect/read timeout, HTTP error) as an `Err` instead of it being
    /// silently swallowed as an empty/unparseable chunk. Used by
    /// `open_http_stream` for the `llm-connector` extension.
    PipeReaderFallible(Mutex<std::sync::mpsc::Receiver<Result<Vec<u8>, String>>>),
    PipeWriter(Mutex<Box<dyn std::io::Write + Send>>),
    Closed,
}

pub struct HostFile {
    pub path: std::path::PathBuf,
    pub file: std::fs::File,
}

pub struct HostExecution {
    pub running: Mutex<crate::process::RunningProcess>,
    pub stdout: Mutex<Option<std::sync::mpsc::Receiver<Vec<u8>>>>,
    pub stderr: Mutex<Option<std::sync::mpsc::Receiver<Vec<u8>>>>,
    pub stdin: Mutex<Option<Box<dyn std::io::Write + Send>>>,
}

pub struct WasmState {
    pub name: String,
    pub sandbox: Arc<dyn FsSubsystem>,
    pub process_manager: Arc<dyn ProcessSubsystem>,
    pub dag: Arc<dyn DagSubsystem>,
    pub network: Arc<dyn NetworkSubsystem>,
    pub permissions: PermissionConfig,
    pub active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
    pub event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
    pub llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    pub orchestrator: Option<std::sync::Weak<crate::orchestrator::Orchestrator>>,
    pub hitl_enabled: bool,
    pub wasi: WasiCtx,
    pub resource_table: ResourceTable,
}

impl WasiView for WasmState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasmState {
    pub fn is_aborted(&self) -> bool {
        if let Some(orch) = self
            .orchestrator
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
        {
            return orch.is_aborted();
        }
        false
    }
}

pub struct WasmRuntime {
    pub store: Store<WasmState>,
    pub extension: Option<bindings::RadExtension>,
    pub orchestrator: Option<bindings::rad_orchestrator::RadOrchestrator>,
    pub security_guard: Option<bindings::rad_security_guard::RadSecurityGuard>,
    pub tool_provider: Option<bindings::rad_tool_provider::RadToolProvider>,
    pub llm_connector: Option<bindings::rad_llm_connector::LlmConnector>,
    pub instance: wasmtime::component::Instance,
    pub role: String,
}

impl WasmRuntime {
    pub fn on_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        if self.store.data().is_aborted() {
            return Err("Task aborted by user".to_string());
        }
        let ext_name = self.store.data().name.clone();
        crate::log_host!(
            "[HOST] Dispatching event to Wasm '{}': {:?}",
            ext_name,
            event
        );
        let wit_event = bindings::wit::RasCoreEvent::from(event.clone());

        if self.role == "orchestrator" {
            if let Some(ref orch) = self.orchestrator {
                let res = orch
                    .call_on_event(&mut self.store, &wit_event)
                    .map_err(|e| format_wasm_error(&ext_name, "on_event", &e))?;
                crate::log_host!("[HOST] Wasm '{}' on_event returned: {:?}", ext_name, res);
                res.map_err(|e| format!("Extension internal error: {e}"))
            } else {
                Err("Orchestrator bindings missing".to_string())
            }
        } else if self.role == "security"
            || self.role == "tool-provider"
            || self.role == "llm-connector"
            || self.role == "context-tools"
            || self.role == "web-access"
        {
            Ok(())
        } else {
            if let Some(ref ext) = self.extension {
                ext.call_on_event(&mut self.store, &wit_event)
                    .map_err(|e| format_wasm_error(&ext_name, "on_event", &e))?
                    .map_err(|e| format!("Extension internal error: {e}"))
            } else {
                Err("Legacy extension bindings missing".to_string())
            }
        }
    }

    pub fn set_event_tx(&mut self, event_tx: std::sync::mpsc::Sender<RasCoreEvent>) {
        let state = self.store.data_mut();
        state.event_tx = event_tx;
    }

    pub fn verify_rpc(&mut self, req_bytes: &[u8]) -> Result<(), String> {
        let request: crate::ipc::RasRpcRequest = serde_json::from_slice(req_bytes)
            .map_err(|e| format!("Failed to parse request bytes: {e}"))?;
        let bindings_cmd = bindings::wit::RasRpcCommand::from(request.command.clone());
        crate::log_host!(
            "[HOST] verify_rpc for extension '{}': CoreCommand = {:?}, bindings::wit = {:?}",
            self.store.data().name,
            request.command,
            bindings_cmd
        );

        let ext_name = self.store.data().name.clone();

        if self.role == "security" {
            if let Some(ref guard) = self.security_guard {
                let approved = guard
                    .call_verify_rpc(&mut self.store, &bindings_cmd)
                    .map_err(|e| format_wasm_error(&ext_name, "verify_rpc", &e))?;
                if !approved {
                    return Err("Operation rejected by security extension".to_string());
                }
            } else {
                return Err("Security guard bindings missing".to_string());
            }
        } else if self.role == "orchestrator"
            || self.role == "tool-provider"
            || self.role == "llm-connector"
            || self.role == "context-tools"
            || self.role == "web-access"
        {
            // Orchestrator, tool-provider, and llm-connector are auto-approved by host unless targeted by a security guard
        } else {
            if let Some(ref ext) = self.extension {
                let approved = ext
                    .call_verify_rpc(&mut self.store, &bindings_cmd)
                    .map_err(|e| format_wasm_error(&ext_name, "verify_rpc", &e))?;
                if !approved {
                    return Err("Operation rejected by security extension".to_string());
                }
            } else {
                return Err("Legacy extension bindings missing".to_string());
            }
        }

        Ok(())
    }

    pub fn get_tools(&mut self) -> Result<String, String> {
        let ext_name = self.store.data().name.clone();
        if let Some(ref provider) = self.tool_provider {
            provider
                .call_get_tools(&mut self.store)
                .map_err(|e| format_wasm_error(&ext_name, "get_tools", &e))?
        } else {
            Err("Tool provider bindings missing".to_string())
        }
    }

    pub fn call_extension_method(
        &mut self,
        method: &str,
        arguments: &str,
    ) -> Result<String, String> {
        // Nothing implements this any more. `context-tools` was the only role
        // that ever did, and it is a kernel module now (AWU 957) — the host
        // routes `CallExtension` to modules first and only reaches here if no
        // module provides the method.
        //
        // Kept as an explicit error rather than deleted along with the caller:
        // it is the seam every remaining extension will pass through as it
        // moves across, and an error naming the method is what makes a missing
        // module obvious instead of silent.
        let _ = arguments;
        Err(format!(
            "no module provides '{}.{method}', and no extension implements it",
            self.role
        ))
    }
}

fn format_wasm_error(ext_name: &str, action: &str, err: &wasmtime::Error) -> String {
    let err_str = err.to_string();
    println!(
        "[WASM Runtime Error] Extension '{ext_name}' failed during {action}. Details: {err_str}"
    );
    format!("Extension '{ext_name}' failed during {action}: {err_str}")
}
