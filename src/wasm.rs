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
mod imports_resources;
mod imports_rpc;
pub mod loader;
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
    pub llm_connector: Option<bindings::rad_llm_connector::LlmConnector>,
    pub context_tools: Option<bindings::rad_context_tools::ContextToolsExtension>,
    pub web_access: Option<bindings::rad_web_access::WebAccessExtension>,
    pub instance: wasmtime::component::Instance,
    pub role: String,
}

impl WasmRuntime {
    pub fn on_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
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

    pub fn call_extension_method(
        &mut self,
        method: &str,
        arguments: &str,
    ) -> Result<String, String> {
        let ext_name = self.store.data().name.clone();
        match self.role.as_str() {
            "context-tools" => {
                if let Some(ref ct) = self.context_tools {
                    match method {
                        "optimize" => {
                            use crate::wasm::bindings::rad_context_tools::exports::radcomp::context_tools::context_tools::OptimizationRequest;
                            let req: OptimizationRequest = serde_json::from_str(arguments)
                                .map_err(|e| format!("Failed to parse OptimizationRequest: {e}"))?;
                            let resp = ct
                                .radcomp_context_tools_context_tools()
                                .call_optimize(&mut self.store, &req)
                                .map_err(|e| format_wasm_error(&ext_name, "optimize", &e))??;
                            serde_json::to_string(&resp)
                                .map_err(|e| format!("Serialization error: {e}"))
                        }
                        "get-repo-map" => {
                            let resp = ct
                                .radcomp_context_tools_context_tools()
                                .call_get_repo_map(&mut self.store)
                                .map_err(|e| format_wasm_error(&ext_name, "get_repo_map", &e))??;
                            Ok(resp)
                        }
                        other => Err(format!("Unknown context-tools method: {other}")),
                    }
                } else {
                    Err("context-tools bindings missing".to_string())
                }
            }
            "web-access" => {
                if let Some(ref wa) = self.web_access {
                    match method {
                        "search" => {
                            let resp = wa
                                .radcomp_web_access_web_access()
                                .call_search(&mut self.store, arguments)
                                .map_err(|e| format_wasm_error(&ext_name, "search", &e))??;
                            Ok(resp)
                        }
                        "fetch" => {
                            let resp = wa
                                .radcomp_web_access_web_access()
                                .call_fetch(&mut self.store, arguments)
                                .map_err(|e| format_wasm_error(&ext_name, "fetch", &e))??;
                            Ok(resp)
                        }
                        other => Err(format!("Unknown web-access method: {other}")),
                    }
                } else {
                    Err("web-access bindings missing".to_string())
                }
            }
            other => Err(format!(
                "call_extension_method not supported for role: {other}"
            )),
        }
    }
}

fn format_wasm_error(ext_name: &str, action: &str, err: &wasmtime::Error) -> String {
    let err_str = err.to_string();
    println!(
        "[WASM Runtime Error] Extension '{ext_name}' failed during {action}. Details: {err_str}"
    );
    format!("Extension '{ext_name}' failed during {action}: {err_str}")
}
