use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::config::PermissionConfig;
use crate::ipc::RasCoreEvent;
use crate::process::RunningProcess;
use crate::subsystems::{DagSubsystem, FsSubsystem, NetworkSubsystem, ProcessSubsystem};

pub mod bindings;
pub mod imports;
pub mod permissions;
pub mod rpc;
pub mod rpc_dag;
pub mod rpc_fs;
pub mod rpc_meta;
pub mod rpc_network;
pub mod rpc_process;
pub mod rpc_terminal;

#[cfg(test)]
mod tests;

pub enum HostStream {
    File(std::fs::File),
    PipeReader(Mutex<std::sync::mpsc::Receiver<Vec<u8>>>),
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
    pub active_mcp_servers: Arc<Mutex<HashMap<String, crate::mcp::McpProcess>>>,
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

pub struct WasmRuntime {
    pub store: Store<WasmState>,
    pub extension: Option<bindings::RadExtension>,
    pub orchestrator: Option<bindings::rad_orchestrator::RadOrchestrator>,
    pub security_guard: Option<bindings::rad_security_guard::RadSecurityGuard>,
    pub tool_provider: Option<bindings::rad_tool_provider::RadToolProvider>,
    pub instance: wasmtime::component::Instance,
    pub role: String,
}

impl WasmRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        wasm_path: &Path,
        role: String,
        permissions: PermissionConfig,
        sandbox: Arc<dyn FsSubsystem>,
        process_manager: Arc<dyn ProcessSubsystem>,
        dag: Arc<dyn DagSubsystem>,
        network: Arc<dyn NetworkSubsystem>,
        active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
        orchestrator: Option<std::sync::Weak<crate::orchestrator::Orchestrator>>,
        hitl_enabled: bool,
    ) -> Result<Self, String> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_memory(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(|e| format!("Failed to create Engine: {e}"))?;
        let component = Component::from_file(&engine, wasm_path)
            .map_err(|e| format!("Failed to load Wasm component: {e}"))?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| format!("Linker error WASI: {e}"))?;

        match role.as_str() {
            "orchestrator" => {
                bindings::rad_orchestrator::RadOrchestrator::add_to_linker(
                    &mut linker,
                    |state: &mut WasmState| state,
                )
                .map_err(|e| format!("Linker error RadOrchestrator: {e}"))?;
            }
            "security" => {
                bindings::rad_security_guard::RadSecurityGuard::add_to_linker(
                    &mut linker,
                    |state: &mut WasmState| state,
                )
                .map_err(|e| format!("Linker error RadSecurityGuard: {e}"))?;
            }
            "tool-provider" => {
                bindings::rad_tool_provider::RadToolProvider::add_to_linker(
                    &mut linker,
                    |state: &mut WasmState| state,
                )
                .map_err(|e| format!("Linker error RadToolProvider: {e}"))?;
            }
            _ => {
                bindings::RadExtension::add_to_linker(&mut linker, |state: &mut WasmState| state)
                    .map_err(|e| format!("Linker error RadExtension: {e}"))?;
            }
        }

        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .inherit_env()
            .build();
        let resource_table = ResourceTable::new();

        let state = WasmState {
            name,
            sandbox,
            process_manager,
            dag,
            network,
            permissions,
            active_processes,
            active_mcp_servers: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            llm_timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Infinite)),
            orchestrator,
            hitl_enabled,
            wasi,
            resource_table,
        };

        let mut store = Store::new(&engine, state);
        let instance = linker
            .instantiate(&mut store, &component)
            .map_err(|e| format!("Failed to instantiate component: {e}"))?;

        let mut extension = None;
        let mut orchestrator = None;
        let mut security_guard = None;
        let mut tool_provider = None;

        match role.as_str() {
            "orchestrator" => {
                orchestrator = Some(
                    bindings::rad_orchestrator::RadOrchestrator::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create orchestrator bindings: {e}"))?,
                );
            }
            "security" => {
                security_guard = Some(
                    bindings::rad_security_guard::RadSecurityGuard::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create security bindings: {e}"))?,
                );
            }
            "tool-provider" => {
                tool_provider = Some(
                    bindings::rad_tool_provider::RadToolProvider::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create tool-provider bindings: {e}"))?,
                );
            }
            _ => {
                extension = Some(
                    bindings::RadExtension::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create legacy bindings: {e}"))?,
                );
            }
        }

        Ok(Self {
            store,
            extension,
            orchestrator,
            security_guard,
            tool_provider,
            instance,
            role,
        })
    }

    pub fn on_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        let ext_name = self.store.data().name.clone();
        let wit_event = bindings::wit::RasCoreEvent::from(event.clone());

        if self.role == "orchestrator" {
            if let Some(ref orch) = self.orchestrator {
                orch.call_on_event(&mut self.store, &wit_event)
                    .map_err(|e| format_wasm_error(&ext_name, "on_event", &e))?
                    .map_err(|e| format!("Extension internal error: {e}"))
            } else {
                Err("Orchestrator bindings missing".to_string())
            }
        } else if self.role == "security" || self.role == "tool-provider" {
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
        let bindings_cmd = bindings::wit::RasRpcCommand::from(request.command);

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
        } else if self.role == "orchestrator" || self.role == "tool-provider" {
            // Orchestrator and tool-provider are auto-approved by host unless targeted by a security guard
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

    pub fn execute_tool(&mut self, name: &str, arguments: &str) -> Result<String, String> {
        let ext_name = self.store.data().name.clone();
        if let Some(ref provider) = self.tool_provider {
            let res = provider
                .call_execute_tool(&mut self.store, name, arguments)
                .map_err(|e| format_wasm_error(&ext_name, "execute_tool", &e))??;
            Ok(res.rep().to_string())
        } else {
            Err("Tool provider bindings missing".to_string())
        }
    }
}

fn format_wasm_error(ext_name: &str, action: &str, err: &wasmtime::Error) -> String {
    let err_str = err.to_string();
    eprintln!(
        "[WASM Runtime Error] Extension '{ext_name}' failed during {action}. Details: {err_str}"
    );
    format!("Extension '{ext_name}' failed during {action}: {err_str}")
}
