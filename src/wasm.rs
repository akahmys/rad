use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store, AsContextMut};

use crate::config::PermissionConfig;
use crate::ipc::{RasCoreEvent, RasRpcRequest, RasRpcResponse};
use crate::process::RunningProcess;

use crate::subsystems::{FsSubsystem, ProcessSubsystem, DagSubsystem, NetworkSubsystem};

pub mod permissions;
pub mod rpc;

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
    pub event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
    pub llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    pub orchestrator: Option<std::sync::Weak<crate::orchestrator::Orchestrator>>,
}

pub struct WasmRuntime {
    pub store: Store<WasmState>,
    pub instance: Instance,
}

impl WasmRuntime {
    /// Creates a new `WasmRuntime` by loading the module from the given path,
    /// setting up host imports, and instantiating the module.
    ///
    /// # Errors
    ///
    /// Returns an error if engine creation, compilation, linking, or instantiation fails.
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
    ) -> Result<Self, String> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_memory(true);
        let engine = Engine::new(&config).map_err(|e| format!("Failed to create Engine: {e}"))?;
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| format!("Failed to load Wasm module from file: {e}"))?;

        Self::new_with_module(name, &module, permissions, sandbox, process_manager, dag, network, active_processes, event_tx, orchestrator)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_module(
        name: String,
        module: &Module,
        permissions: PermissionConfig,
        sandbox: Arc<dyn FsSubsystem>,
        process_manager: Arc<dyn ProcessSubsystem>,
        dag: Arc<dyn DagSubsystem>,
        network: Arc<dyn NetworkSubsystem>,
        active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
        orchestrator: Option<std::sync::Weak<crate::orchestrator::Orchestrator>>,
    ) -> Result<Self, String> {
        let mut linker = Linker::new(module.engine());

        linker.func_wrap(
            "env",
            "rad_host_rpc",
            |mut caller: Caller<'_, WasmState>, req_ptr: i32, req_len: i32| -> u64 {
                handle_host_rpc(&mut caller, req_ptr, req_len)
            },
        ).map_err(|e| format!("Linker error: {e}"))?;

        let state = WasmState {
            name,
            sandbox,
            process_manager,
            dag,
            network,
            permissions,
            active_processes,
            event_tx,
            llm_timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Infinite)),
            orchestrator,
        };

        let mut store = Store::new(module.engine(), state);
        let instance = linker
            .instantiate(&mut store, module)
            .map_err(|e| format!("Failed to instantiate module: {e}"))?;

        Ok(Self {
            store,
            instance,
        })
    }

    /// Dispatches an event to the Wasm extension.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization, memory allocation, or function execution fails.
    pub fn on_event(&mut self, event: &RasCoreEvent) -> Result<(), String> {
        let event_bytes = serde_json::to_vec(event).map_err(|e| format!("Event serialization error: {e}"))?;
        let len = i32::try_from(event_bytes.len()).map_err(|e| format!("Invalid length: {e}"))?;

        let alloc_fn = self.instance
            .get_typed_func::<i32, i32>(&mut self.store, "alloc")
            .map_err(|e| format!("Failed to get 'alloc' export: {e}"))?;

        let ptr = alloc_fn
            .call(&mut self.store, len)
            .map_err(|e| format!("Failed to call 'alloc': {e}"))?;

        let memory = self.instance
            .get_export(&mut self.store, "memory")
            .and_then(wasmtime::Extern::into_memory)
            .ok_or_else(|| "Failed to get export memory".to_string())?;

        let Ok(ptr_usize) = usize::try_from(ptr) else {
            return Err("Invalid event pointer".to_string());
        };

        memory
            .write(&mut self.store, ptr_usize, &event_bytes)
            .map_err(|e| format!("Failed to write event to memory: {e}"))?;

        let on_event_fn = self.instance
            .get_typed_func::<(i32, i32), u64>(&mut self.store, "rad_on_event")
            .map_err(|e| format!("Failed to get 'rad_on_event' export: {e}"))?;

        let ret = on_event_fn
            .call(&mut self.store, (ptr, len))
            .map_err(|e| format!("Failed to call 'rad_on_event': {e}"))?;

        let dealloc_fn = self.instance
            .get_typed_func::<(i32, i32), ()>(&mut self.store, "dealloc")
            .map_err(|e| format!("Failed to get 'dealloc' export: {e}"))?;

        dealloc_fn
            .call(&mut self.store, (ptr, len))
            .map_err(|e| format!("Failed to call 'dealloc' for event: {e}"))?;

        if ret != 0 {
            let err_ptr = (ret >> 32) as u32;
            let err_len = (ret & 0xFFFF_FFFF) as u32;
            if err_ptr != 0 && err_len > 0 {
                let err_ptr_usize = err_ptr as usize;
                let err_len_usize = err_len as usize;
                let mut err_buf = vec![0; err_len_usize];
                if memory.read(&self.store, err_ptr_usize, &mut err_buf).is_ok() {
                    let err_msg = String::from_utf8_lossy(&err_buf).into_owned();
                    return Err(format!("Extension error: {err_msg}"));
                }
            }
            return Err("Extension execution failed".to_string());
        }

        Ok(())
    }

    pub fn set_event_tx(&mut self, event_tx: std::sync::mpsc::Sender<RasCoreEvent>) {
        let state = self.store.data_mut();
        state.event_tx = event_tx;
    }

    /// Invokes the security verification hook (`rad_verify_rpc`) on the extension.
    ///
    /// # Errors
    ///
    /// Returns error if verify hook rejects the operation.
    pub fn verify_rpc(&mut self, req_bytes: &[u8]) -> Result<(), String> {
        let verify_fn = self.instance
            .get_typed_func::<(i32, i32), u32>(&mut self.store, "rad_verify_rpc");
        
        let Ok(f) = verify_fn else {
            return Ok(());
        };

        let len = i32::try_from(req_bytes.len()).map_err(|e| format!("Invalid length: {e}"))?;
        let alloc_fn = self.instance
            .get_typed_func::<i32, i32>(&mut self.store, "alloc")
            .map_err(|e| format!("Failed to get 'alloc' export: {e}"))?;
        
        let ptr = alloc_fn
            .call(&mut self.store, len)
            .map_err(|e| format!("Failed to call 'alloc': {e}"))?;

        let memory = self.instance
            .get_export(&mut self.store, "memory")
            .and_then(wasmtime::Extern::into_memory)
            .ok_or_else(|| "Failed to get export memory".to_string())?;

        let Ok(ptr_usize) = usize::try_from(ptr) else {
            return Err("Invalid pointer".to_string());
        };

        memory
            .write(&mut self.store, ptr_usize, req_bytes)
            .map_err(|e| format!("Failed to write request to memory: {e}"))?;

        let ret = f.call(&mut self.store, (ptr, len))
            .map_err(|e| format!("Failed to call 'rad_verify_rpc': {e}"))?;

        let dealloc_fn = self.instance
            .get_typed_func::<(i32, i32), ()>(&mut self.store, "dealloc")
            .map_err(|e| format!("Failed to get 'dealloc' export: {e}"))?;

        dealloc_fn
            .call(&mut self.store, (ptr, len))
            .map_err(|e| format!("Failed to call 'dealloc' for verify request: {e}"))?;

        if ret != 1 {
            return Err("Operation rejected by security extension".to_string());
        }

        Ok(())
    }
}

fn handle_host_rpc(caller: &mut Caller<'_, WasmState>, req_ptr: i32, req_len: i32) -> u64 {
    let Some(memory) = caller.get_export("memory").and_then(wasmtime::Extern::into_memory) else { return 0 };

    let Ok(req_ptr_usize) = usize::try_from(req_ptr) else { return 0 };
    let Ok(req_len_usize) = usize::try_from(req_len) else { return 0 };

    let mut buf = vec![0; req_len_usize];
    if memory.read(&*caller, req_ptr_usize, &mut buf).is_err() {
        return 0;
    }

    let Ok(raw_request) = std::str::from_utf8(&buf) else { return 0 };

    let Ok(request) = serde_json::from_str::<RasRpcRequest>(raw_request) else {
        let resp = RasRpcResponse {
            id: None,
            result: Err("JSON Parse error in Host".to_string()),
        };
        return write_response_to_guest(caller, &resp);
    };

    {
        let state = caller.data();
        if let Err(err_msg) = permissions::check_permissions(&request.command, &state.permissions) {
            let resp = RasRpcResponse {
                id: request.id,
                result: Err(err_msg),
            };
            return write_response_to_guest(caller, &resp);
        }
    }

    // 1. First, call the calling instance's own verification hook (safe, no deadlock)
    let my_verify_result = {
        let verify_fn = caller.get_export("rad_verify_rpc")
            .and_then(|e| e.into_func())
            .and_then(|f| f.typed::<(i32, i32), u32>(&*caller).ok());

        if let Some(f) = verify_fn {
            match f.call(caller.as_context_mut(), (req_ptr, req_len)) {
                Ok(1) => Ok(()),
                _ => Err("Operation rejected by security extension".to_string()),
            }
        } else {
            Ok(())
        }
    };

    if let Err(err_msg) = my_verify_result {
        let resp = RasRpcResponse {
            id: request.id,
            result: Err(err_msg),
        };
        return write_response_to_guest(caller, &resp);
    }

    // 2. Next, verify against all OTHER active extensions through Orchestrator
    let other_verify_result = {
        let state = caller.data();
        let my_name = state.name.clone();
        if let Some(ref weak_orch) = state.orchestrator {
            weak_orch.upgrade().map(|orch| orch.verify_rpc_exclude(&my_name, &request, &buf))
        } else {
            None
        }
    };

    if let Some(Err(err_msg)) = other_verify_result {
        let resp = RasRpcResponse {
            id: request.id,
            result: Err(err_msg),
        };
        return write_response_to_guest(caller, &resp);
    }

    let state = caller.data();
    let orchestrator = state.orchestrator.as_ref().and_then(|w| w.upgrade());
    let result = rpc::execute_rpc_command(
        &request.command,
        &*state.sandbox,
        &*state.process_manager,
        &*state.dag,
        &*state.network,
        &state.active_processes,
        &state.event_tx,
        &state.llm_timeout_policy,
        orchestrator.as_ref(),
        request.id.clone().unwrap_or_else(|| "unknown".to_string()),
    );

    let resp = RasRpcResponse {
        id: request.id,
        result,
    };

    write_response_to_guest(caller, &resp)
}

fn write_response_to_guest(caller: &mut Caller<'_, WasmState>, resp: &RasRpcResponse) -> u64 {
    let Ok(resp_bytes) = serde_json::to_vec(resp) else { return 0 };
    let Ok(resp_len) = i32::try_from(resp_bytes.len()) else { return 0 };

    let Some(alloc_export) = caller.get_export("alloc") else { return 0 };
    let Some(alloc_fn) = alloc_export.into_func() else { return 0 };
    let Ok(typed_alloc) = alloc_fn.typed::<i32, i32>(&*caller) else { return 0 };
    
    let Ok(resp_ptr) = typed_alloc.call(&mut *caller, resp_len) else { return 0 };

    let Some(memory) = caller.get_export("memory").and_then(wasmtime::Extern::into_memory) else { return 0 };

    let Ok(resp_ptr_usize) = usize::try_from(resp_ptr) else { return 0 };

    if memory.write(&mut *caller, resp_ptr_usize, &resp_bytes).is_err() {
        return 0;
    }

    let Ok(resp_ptr_u64) = u64::try_from(resp_ptr) else { return 0 };
    let Ok(resp_len_u64) = u64::try_from(resp_len) else { return 0 };

    (resp_ptr_u64 << 32) | resp_len_u64
}
