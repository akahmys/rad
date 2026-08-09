// The main Wasm event-dispatch loop, split out of `runner.rs` to stay under
// the 300-line file limit.
//
// It used to share this file with `verify_rpc_exclude`, the RPC verification
// fan-out that asked every extension but the caller to approve each request.
// That went in AWU 973 with `ext/security-guard`: policy is a module now, and
// `modules/mcp` asks it directly (ARCHITECTURE-NEXT.md §3.4.3).
use super::Orchestrator;
use crate::ipc::{RasCoreEvent, route_event_to_terminal};
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;

impl Orchestrator {
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
