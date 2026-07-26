// RPC verification fan-out and the main Wasm event-dispatch loop, split out
// of `runner.rs` to stay under the 300-line file limit.
use super::Orchestrator;
use crate::ipc::{RasCoreEvent, route_event_to_terminal};
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;

impl Orchestrator {
    /// Verifies an RPC request across all active extensions EXCEPT the calling one.
    ///
    /// # Errors
    ///
    /// Returns error if any extension rejects the operation.
    pub fn verify_rpc_exclude(
        &self,
        exclude_name: &str,
        _request: &crate::ipc::RasRpcRequest,
        req_bytes: &[u8],
    ) -> Result<(), String> {
        crate::log_host!(
            "[HOST] verify_rpc_exclude started (exclude: {})",
            exclude_name
        );
        let runtimes = {
            let guard = self.wasm_runtime.lock();
            guard.clone()
        };

        for (name, runtime_arc) in runtimes {
            if name == exclude_name {
                crate::log_host!(
                    "[HOST] verify_rpc_exclude: skipping excluded extension '{}'",
                    name
                );
                continue;
            }
            crate::log_host!("[HOST] verify_rpc_exclude: try_locking '{}'", name);
            let Some(mut runtime) = runtime_arc.try_lock() else {
                crate::log_host!(
                    "[HOST] verify_rpc_exclude: skipping currently locked extension '{}' (already in call chain)",
                    name
                );
                continue;
            };
            crate::log_host!(
                "[HOST] verify_rpc_exclude: locked '{}', calling verify_rpc",
                name
            );
            let res = runtime.verify_rpc(req_bytes);
            crate::log_host!(
                "[HOST] verify_rpc_exclude: verify_rpc for '{}' returned: {:?}",
                name,
                res
            );
            if let Err(e) = res {
                return Err(format!("Operation rejected by extension '{name}': {e}"));
            }
        }
        crate::log_host!("[HOST] verify_rpc_exclude completed successfully");
        Ok(())
    }

    pub(crate) fn process_event_loop(
        &self,
        event_rx: &Receiver<RasCoreEvent>,
        wasm_runtimes: &HashMap<String, Arc<Mutex<WasmRuntime>>>,
    ) -> Result<(), String> {
        while let Ok(event) = event_rx.recv() {
            if self.abort_flag.load(Ordering::SeqCst) {
                break;
            }

            let _ = route_event_to_terminal(&event);

            if let RasCoreEvent::TaskCompleted = event {
                break;
            }

            for runtime_arc in wasm_runtimes.values() {
                let mut runtime = runtime_arc.lock();
                runtime.on_event(&event)?;
            }
        }
        Ok(())
    }
}
