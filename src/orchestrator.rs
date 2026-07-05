use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::path::Path;
use crate::config::Config;
use crate::fs::FsSandbox;
use crate::process::{ProcessManager, RunningProcess};
use crate::dag::Dag;
use crate::wasm::WasmRuntime;
use crate::ipc::{RasCoreEvent, route_event_to_terminal};

pub struct Orchestrator {
    config: Config,
    sandbox: Arc<FsSandbox>,
    process_manager: Arc<ProcessManager>,
    pub dag: Arc<Mutex<Dag>>,
    active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
    pub session_id: String,
    wasm_runtime: Mutex<Option<WasmRuntime>>,
}

impl Orchestrator {
    pub fn new(config: Config, session_id: String, dag: Arc<Mutex<Dag>>) -> Self {
        let sandbox = Arc::new(FsSandbox::new(
            config.core.workspace.clone().into(),
            config.core.snapshot.clone().into(),
            config.extensions.iter().flat_map(|e| e.permissions.as_ref().map(|p| p.fs_read_allow.clone()).unwrap_or_default()).collect(),
            config.extensions.iter().flat_map(|e| e.permissions.as_ref().map(|p| p.fs_write_allow.clone()).unwrap_or_default()).collect(),
        ));
        let process_manager = Arc::new(ProcessManager::new());
        let active_processes = Arc::new(Mutex::new(HashMap::new()));

        Self {
            config,
            sandbox,
            process_manager,
            dag,
            active_processes,
            session_id,
            wasm_runtime: Mutex::new(None),
        }
    }

    /// Spawns the autonomous execution loop in the same process.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasm runtime initialization or execution fails.
    pub fn run_task(&self, instruction: String) -> Result<(), String> {
        let (event_tx, event_rx) = channel::<RasCoreEvent>();
        
        let mut wasm_guard = self.wasm_runtime.lock().map_err(|e| format!("Wasm lock error: {e}"))?;
        let wasm_runtime = self.get_or_init_runtime(&mut wasm_guard, event_tx.clone())?;

        let init_event = RasCoreEvent::HumanInputReceived { text: instruction };
        if let Some(ref mut runtime) = *wasm_runtime {
            runtime.on_event(&init_event)?;
        } else {
            let _ = event_tx.send(init_event);
        }

        drop(event_tx);

        self.process_event_loop(event_rx, wasm_runtime)?;

        Ok(())
    }

    fn get_or_init_runtime<'a>(
        &self,
        guard: &'a mut Option<WasmRuntime>,
        event_tx: Sender<RasCoreEvent>,
    ) -> Result<&'a mut Option<WasmRuntime>, String> {
        if let Some(ref mut runtime) = *guard {
            runtime.set_event_tx(event_tx);
            return Ok(guard);
        }

        let ext_config = self.config.extensions.iter().find(|e| e.enabled);
        let Some(ext) = ext_config else {
            return Ok(guard);
        };

        let permissions = ext.permissions.clone().unwrap_or_default();
        let wasm_path = Path::new(&ext.source);
        if wasm_path.exists() {
            let runtime = WasmRuntime::new(
                wasm_path,
                permissions,
                self.sandbox.clone(),
                self.process_manager.clone(),
                self.dag.clone(),
                self.active_processes.clone(),
                event_tx,
            )?;
            *guard = Some(runtime);
        }
        Ok(guard)
    }

    fn process_event_loop(
        &self,
        event_rx: Receiver<RasCoreEvent>,
        wasm_runtime: &mut Option<WasmRuntime>,
    ) -> Result<(), String> {
        while let Ok(event) = event_rx.recv() {
            // Route events to terminal in real-time
            let _ = route_event_to_terminal(&event);

            match event {
                RasCoreEvent::HttpChunkReceived { chunk } => {
                    if let Some(runtime) = wasm_runtime {
                        runtime.on_event(&RasCoreEvent::HttpChunkReceived { chunk })?;
                    }
                }
                RasCoreEvent::ProcessExited { pgid, exit_code } => {
                    if let Some(runtime) = wasm_runtime {
                        runtime.on_event(&RasCoreEvent::ProcessExited { pgid, exit_code })?;
                    }
                }
                RasCoreEvent::FileChanged { path, change_type } => {
                    if let Some(runtime) = wasm_runtime {
                        runtime.on_event(&RasCoreEvent::FileChanged { path, change_type })?;
                    }
                }
                RasCoreEvent::StreamTimeout { target, duration_ms } => {
                    if let Some(runtime) = wasm_runtime {
                        runtime.on_event(&RasCoreEvent::StreamTimeout { target, duration_ms })?;
                    }
                }
                RasCoreEvent::TaskCompleted => {
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_creation() {
        let config = Config::default();
        let dag = Arc::new(Mutex::new(Dag::new()));
        let orch = Orchestrator::new(config, "test_session".to_string(), dag);
        assert_eq!(orch.session_id, "test_session");
    }
}
