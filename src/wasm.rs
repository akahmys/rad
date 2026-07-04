use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::{Caller, Engine, Instance, Linker, Module, Store};

use crate::config::PermissionConfig;
use crate::dag::Dag;
use crate::fs::FsSandbox;
use crate::ipc::{RasCoreEvent, RasRpcRequest, RasRpcResponse};
use crate::process::{ProcessManager, RunningProcess};

pub mod permissions;
pub mod rpc;

#[cfg(test)]
mod tests;

pub struct WasmState {
    pub sandbox: Arc<FsSandbox>,
    pub process_manager: Arc<ProcessManager>,
    pub dag: Arc<Mutex<Dag>>,
    pub permissions: PermissionConfig,
    pub active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
    pub event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
    pub llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
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
    pub fn new(
        wasm_path: &Path,
        permissions: PermissionConfig,
        sandbox: Arc<FsSandbox>,
        process_manager: Arc<ProcessManager>,
        dag: Arc<Mutex<Dag>>,
        active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
    ) -> Result<Self, String> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_memory(true);
        let engine = Engine::new(&config).map_err(|e| format!("Failed to create Engine: {e}"))?;
        let module = Module::from_file(&engine, wasm_path)
            .map_err(|e| format!("Failed to load Wasm module from file: {e}"))?;

        Self::new_with_module(&module, permissions, sandbox, process_manager, dag, active_processes, event_tx)
    }

    pub fn new_with_module(
        module: &Module,
        permissions: PermissionConfig,
        sandbox: Arc<FsSandbox>,
        process_manager: Arc<ProcessManager>,
        dag: Arc<Mutex<Dag>>,
        active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
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
            sandbox,
            process_manager,
            dag,
            permissions,
            active_processes,
            event_tx,
            llm_timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Infinite)),
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
}

fn handle_host_rpc(caller: &mut Caller<'_, WasmState>, req_ptr: i32, req_len: i32) -> u64 {
    let Some(memory) = caller.get_export("memory").and_then(wasmtime::Extern::into_memory) else { return 0 };

    let Ok(req_ptr_usize) = usize::try_from(req_ptr) else { return 0 };
    let Ok(req_len_usize) = usize::try_from(req_len) else { return 0 };

    let mut buf = vec![0; req_len_usize];
    if memory.read(&*caller, req_ptr_usize, &mut buf).is_err() {
        return 0;
    }

    let Ok(raw_request) = String::from_utf8(buf) else { return 0 };

    let Ok(request) = serde_json::from_str::<RasRpcRequest>(&raw_request) else {
        let resp = RasRpcResponse {
            id: None,
            result: Err("JSON Parse error in Host".to_string()),
        };
        return write_response_to_guest(caller, &resp);
    };

    let state = caller.data();
    if let Err(err_msg) = permissions::check_permissions(&request.command, &state.permissions) {
        let resp = RasRpcResponse {
            id: request.id,
            result: Err(err_msg),
        };
        return write_response_to_guest(caller, &resp);
    }

    let result = rpc::execute_rpc_command(
        &request.command,
        &state.sandbox,
        &state.process_manager,
        &state.dag,
        &state.active_processes,
        &state.event_tx,
        &state.llm_timeout_policy,
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
