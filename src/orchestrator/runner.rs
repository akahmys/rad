use super::Orchestrator;
use crate::git;
use crate::ipc::{RasCoreEvent, route_event_to_terminal};
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, Sender, channel};

impl Orchestrator {
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
            if let Err(e) = self_clone.run_task_internal(&instruction) {
                println!("\x1b[1;31mOrchestrator task failed: {e}\x1b[0m");
                Err(e)
            } else {
                Ok(())
            }
        });

        *self.running_task.lock() = Some(handle);

        Ok(())
    }

    fn run_task_internal(self: &Arc<Self>, instruction: &str) -> Result<(), String> {
        let config = self.config.lock().clone();
        let workspace_path = Path::new(&config.core.workspace);
        let session_id = self.session_id.lock().clone();

        // Apply active LLM profile environment settings
        if let Some(ref active_name) = config.llm.active
            && let Some(profile) = config.llm.endpoints.get(active_name)
        {
            unsafe {
                std::env::set_var("OPENAI_BASE_URL", &profile.base_url);
                if let Some(key) = profile.resolved_api_key() {
                    std::env::set_var("OPENAI_API_KEY", key);
                }
                if let Some(ref model) = profile.model {
                    std::env::set_var("OPENAI_MODEL", model);
                }
            }
        }

        // 1. Git Autopilot Setup
        let (has_git, initial_sha) =
            crate::orchestrator::autopilot::setup_git_autopilot(workspace_path, &session_id);

        let mut attempts = 0;
        let max_attempts = 2;

        while attempts < max_attempts {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err("Task aborted by user".to_string());
            }
            let (event_tx, event_rx) = channel::<RasCoreEvent>();

            let wasm_runtimes = self.get_or_init_runtimes(&event_tx)?;
            for runtime_arc in wasm_runtimes.values() {
                let mut runtime = runtime_arc.lock();
                runtime.set_event_tx(event_tx.clone());
            }

            let mut success = true;

            if attempts > 0 {
                let active_calls = {
                    let active_procs = self.active_processes.lock();
                    active_procs
                        .values()
                        .map(|proc| rad_models::PendingToolCallInfo {
                            id: proc.call_id.clone(),
                            name: proc.name.clone(),
                            arguments: proc.arguments.clone(),
                            pgid: Some(proc.pgid().as_raw().to_string()),
                        })
                        .collect::<Vec<_>>()
                };

                let rehydrate_event = RasCoreEvent::Rehydrate { active_calls };
                for (name, runtime_arc) in &wasm_runtimes {
                    let mut runtime = runtime_arc.lock();
                    if let Err(e) = runtime.on_event(&rehydrate_event) {
                        eprintln!("Failed to rehydrate runtime {name}: {e}");
                        success = false;
                        break;
                    }
                }
                if !success {
                    self.clear_runtimes()?;
                    attempts += 1;
                    continue;
                }
            }

            let init_event = RasCoreEvent::HumanInputReceived {
                text: instruction.to_string(),
            };
            if wasm_runtimes.is_empty() {
                let _ = event_tx.send(init_event);
            } else {
                for (name, runtime_arc) in &wasm_runtimes {
                    let mut runtime = runtime_arc.lock();
                    if let Err(e) = runtime.on_event(&init_event) {
                        println!("Wasm execution error on {name}: {e}. Recovering...");
                        success = false;
                        break;
                    }
                }
                if !success {
                    self.clear_runtimes()?;
                    attempts += 1;
                    continue;
                }
            }

            match self.process_event_loop(&event_rx, &wasm_runtimes) {
                Ok(()) => {
                    // Check verification command
                    if let Some(ref verify_cmd) = config.core.verification_command {
                        println!("Running autopilot verification: {verify_cmd}");
                        if crate::orchestrator::autopilot::run_verification_cmd(
                            workspace_path,
                            verify_cmd,
                        ) {
                            println!("Verification PASSED.");
                            if has_git {
                                let _ =
                                    git::create_checkpoint(workspace_path, "verification_passed");
                            }
                        } else {
                            if let Some(ref sha) = initial_sha {
                                println!(
                                    "Verification FAILED. Rolling back codebase to stable SHA: {sha}"
                                );
                                let _ = git::rollback_to_checkpoint(workspace_path, sha);
                            }
                            return Err(
                                "Autopilot verification command failed. Codebase rolled back."
                                    .to_string(),
                            );
                        }
                    }
                    break;
                }
                Err(e) => {
                    if self.abort_flag.load(Ordering::SeqCst) {
                        return Err("Task aborted by user".to_string());
                    }
                    println!("Wasm runtime crashed: {e}. Recovering...");
                    self.clear_runtimes()?;
                    attempts += 1;
                }
            }
            drop(event_tx);
        }

        if attempts >= max_attempts {
            return Err("Wasm execution failed after maximum recovery attempts".to_string());
        }

        Ok(())
    }

    fn get_or_init_runtimes(
        self: &Arc<Self>,
        event_tx: &Sender<RasCoreEvent>,
    ) -> Result<HashMap<String, Arc<Mutex<WasmRuntime>>>, String> {
        let mut guard = self.wasm_runtime.lock();
        let config_guard = self.config.lock();
        for ext in &config_guard.extensions {
            if !ext.enabled {
                continue;
            }
            if guard.contains_key(&ext.name) {
                continue;
            }
            let permissions = ext.permissions.clone().unwrap_or_default();
            let wasm_path = Path::new(&ext.source);
            if wasm_path.exists() {
                let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl {
                    dag: self.dag.clone(),
                });
                let network_subsystem = Arc::new(crate::http::HttpManager);
                let mut runtime = WasmRuntime::new(
                    ext.name.clone(),
                    wasm_path,
                    ext.role.clone(),
                    permissions,
                    self.sandbox.clone() as Arc<dyn crate::subsystems::FsSubsystem>,
                    self.process_manager.clone() as Arc<dyn crate::subsystems::ProcessSubsystem>,
                    dag_subsystem,
                    network_subsystem,
                    self.active_processes.clone(),
                    event_tx.clone(),
                    Some(Arc::downgrade(self)),
                    config_guard.core.hitl_enabled,
                )?;

                if runtime.tool_provider.is_some() {
                    match runtime.get_tools() {
                        Ok(json_str) => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str)
                                && let Some(arr) = val.as_array()
                            {
                                println!(
                                    "\x1b[32mVerified {} tools from extension '{}'\x1b[0m",
                                    arr.len(),
                                    ext.name
                                );
                            }
                        }
                        Err(e) => {
                            println!("\x1b[31mTool provider '{}' error: {e}\x1b[0m", ext.name);
                        }
                    }
                }

                guard.insert(ext.name.clone(), Arc::new(Mutex::new(runtime)));
            }
        }
        Ok(guard.clone())
    }

    fn clear_runtimes(&self) -> Result<(), String> {
        let mut guard = self.wasm_runtime.lock();
        guard.clear();
        Ok(())
    }

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
            crate::log_host!("[HOST] verify_rpc_exclude: trying lock on '{}'", name);
            let Some(mut runtime) = runtime_arc.try_lock() else {
                crate::log_host!(
                    "[HOST] verify_rpc_exclude: failed to lock '{}', skipping",
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

    fn process_event_loop(
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
