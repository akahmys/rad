#![deny(clippy::pedantic)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
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
    running_task: Mutex<Option<std::thread::JoinHandle<Result<(), String>>>>,
    abort_flag: Arc<AtomicBool>,
}

impl Orchestrator {
    #[must_use]
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
            running_task: Mutex::new(None),
            abort_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Checks if a task is currently executing.
    pub fn is_running(&self) -> bool {
        let Ok(mut guard) = self.running_task.lock() else { return false; };
        if let Some(ref handle) = *guard {
            if handle.is_finished() {
                *guard = None;
                return false;
            }
            return true;
        }
        false
    }

    /// Spawns the autonomous execution loop in a background thread.
    ///
    /// # Errors
    ///
    /// Returns an error if Wasm runtime initialization or execution fails,
    /// or if a task is already running.
    pub fn run_task(self: &Arc<Self>, instruction: String) -> Result<(), String> {
        if self.is_running() {
            return Err("A task is already running. Use /rollback to stop it first.".to_string());
        }

        self.abort_flag.store(false, Ordering::SeqCst);
        let self_clone = self.clone();
        let handle = std::thread::spawn(move || {
            self_clone.run_task_internal(instruction)
        });

        if let Ok(mut guard) = self.running_task.lock() {
            *guard = Some(handle);
        }

        Ok(())
    }

    fn run_task_internal(&self, instruction: String) -> Result<(), String> {
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

        self.process_event_loop(&event_rx, wasm_runtime)?;

        Ok(())
    }

    fn get_or_init_runtime<'a>(
        &self,
        guard: &'a mut Option<WasmRuntime>,
        event_tx: Sender<RasCoreEvent>,
    ) -> Result<&'a mut Option<WasmRuntime>, String> {
        let ext_config = self.config.extensions.iter().find(|e| e.enabled);
        let Some(ext) = ext_config else {
            return Ok(guard);
        };

        let permissions = ext.permissions.clone().unwrap_or_default();
        let wasm_path = Path::new(&ext.source);
        if wasm_path.exists() {
            let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl { dag: self.dag.clone() });
            let network_subsystem = Arc::new(crate::http::HttpManager);
            let runtime = WasmRuntime::new(
                wasm_path,
                permissions,
                self.sandbox.clone() as Arc<dyn crate::subsystems::FsSubsystem>,
                self.process_manager.clone() as Arc<dyn crate::subsystems::ProcessSubsystem>,
                dag_subsystem,
                network_subsystem,
                self.active_processes.clone(),
                event_tx,
            )?;
            *guard = Some(runtime);
        }
        Ok(guard)
    }

    fn process_event_loop(
        &self,
        event_rx: &Receiver<RasCoreEvent>,
        wasm_runtime: &mut Option<WasmRuntime>,
    ) -> Result<(), String> {
        while let Ok(event) = event_rx.recv() {
            if self.abort_flag.load(Ordering::SeqCst) {
                break;
            }

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

    /// Rolls back the session state (DAG and filesystem sandbox) to the specified node.
    ///
    /// # Errors
    ///
    /// Returns an error if the node ID does not exist in the DAG or filesystem rollback fails.
    pub fn rollback(&self, node_id: &str) -> Result<(), String> {
        self.abort_flag.store(true, Ordering::SeqCst);

        if let Ok(mut wasm_guard) = self.wasm_runtime.lock() {
            *wasm_guard = None;
        }

        let Ok(mut guard) = self.running_task.lock() else {
            return Err("Failed to lock running_task".to_string());
        };
        if let Some(handle) = guard.take() {
            let _ = handle.join();
        }

        let mut dag_guard = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        if !dag_guard.nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found in DAG"));
        }

        self.sandbox.checkout_snapshot(node_id)?;
        dag_guard.current_node_id = Some(node_id.to_string());

        Ok(())
    }
}

#[cfg(test)]
mod tests;
