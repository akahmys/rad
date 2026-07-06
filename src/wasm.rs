use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::{Engine, Store};
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

use crate::config::PermissionConfig;
use crate::ipc::RasCoreEvent;
use crate::process::RunningProcess;
use crate::subsystems::{FsSubsystem, ProcessSubsystem, DagSubsystem, NetworkSubsystem};

pub mod permissions;
pub mod rpc;
pub mod bindings;

#[cfg(test)]
mod tests;

pub struct WasmState {
    pub name: String,
    pub sandbox: Arc<dyn FsSubsystem>,
    pub process_manager: Arc<dyn ProcessSubsystem>,
    pub dag: Arc<dyn DagSubsystem>,
    pub network: Arc<dyn NetworkSubsystem>,
    pub permissions: PermissionConfig,
    pub active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
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

impl bindings::radcomp::extension::types::Host for WasmState {}

impl bindings::RadExtensionImports for WasmState {
    fn host_rpc(
        &mut self,
        command: bindings::radcomp::extension::types::RasRpcCommand,
    ) -> Result<String, String> {
        let rpc_cmd = rad_models::RasRpcCommand::from(command);
        
        permissions::check_permissions(&rpc_cmd, &self.permissions)?;
 
        let orchestrator = self.orchestrator.as_ref().and_then(|w| w.upgrade());
        if let Some(ref orch) = orchestrator {
            let req = crate::ipc::RasRpcRequest {
                id: Some("wasm_call".to_string()),
                command: rpc_cmd.clone(),
            };
            if let Ok(buf) = serde_json::to_vec(&req) {
                orch.verify_rpc_exclude(&self.name, &req, &buf)?;
            }
        }

        let result = rpc::execute_rpc_command(
            &rpc_cmd,
            &*self.sandbox,
            &*self.process_manager,
            &*self.dag,
            &*self.network,
            &self.active_processes,
            &self.active_mcp_servers,
            &self.event_tx,
            &self.llm_timeout_policy,
            orchestrator.as_ref(),
            "wasm_call".to_string(),
            self.hitl_enabled,
        );

        match result {
            Ok(val) => Ok(val.to_string()),
            Err(e) => Err(e),
        }
    }
}

pub struct WasmRuntime {
    pub store: Store<WasmState>,
    pub extension: bindings::RadExtension,
    pub instance: wasmtime::component::Instance,
}

impl WasmRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        wasm_path: &Path,
        permissions: PermissionConfig,
        sandbox: Arc<dyn FsSubsystem>,
        process_manager: Arc<dyn ProcessSubsystem>,
        dag: Arc<dyn DagSubsystem>,
        network: Arc<dyn NetworkSubsystem>,
        active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
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
        wasmtime_wasi::add_to_linker_sync(&mut linker).map_err(|e| format!("Linker error WASI: {e}"))?;
        bindings::RadExtension::add_to_linker(&mut linker, |state: &mut WasmState| state)
            .map_err(|e| format!("Linker error RadExtension: {e}"))?;

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
        let instance = linker.instantiate(&mut store, &component)
            .map_err(|e| format!("Failed to instantiate component: {e}"))?;
        let extension = bindings::RadExtension::new(&mut store, &instance)
            .map_err(|e| format!("Failed to create bindings: {e}"))?;

        Ok(Self {
            store,
            extension,
            instance,
        })
    }

    pub fn on_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        let wit_event = bindings::radcomp::extension::types::RasCoreEvent::from(event.clone());
        self.extension
            .call_on_event(&mut self.store, &wit_event)
            .map_err(|e| format!("Extension event call failed: {e}"))?
            .map_err(|e| format!("Extension internal error: {e}"))
    }

    pub fn set_event_tx(&mut self, event_tx: std::sync::mpsc::Sender<RasCoreEvent>) {
        let state = self.store.data_mut();
        state.event_tx = event_tx;
    }

    pub fn verify_rpc(&mut self, req_bytes: &[u8]) -> Result<(), String> {
        let request: crate::ipc::RasRpcRequest = serde_json::from_slice(req_bytes)
            .map_err(|e| format!("Failed to parse request bytes: {e}"))?;
        let bindings_cmd = bindings::radcomp::extension::types::RasRpcCommand::from(request.command);
        
        let approved = self.extension
            .call_verify_rpc(&mut self.store, &bindings_cmd)
            .map_err(|e| format!("Failed to call 'verify_rpc': {e}"))?;

        if !approved {
            return Err("Operation rejected by security extension".to_string());
        }

        Ok(())
    }
}
