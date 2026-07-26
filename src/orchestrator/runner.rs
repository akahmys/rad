use super::Orchestrator;
use crate::git;
use crate::ipc::RasCoreEvent;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;

// Runtime lifecycle (get_or_init_runtimes/clear_runtimes) and the RPC
// verification / event-dispatch loop live in sibling files to keep this one
// under the 300-line limit.
mod events;
mod runtimes;

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
        crate::log_host!("[DEBUG] Starting run_task_internal: instruction = '{instruction}'");
        let config = self.config.lock().clone();
        let workspace_path = Path::new(&config.core.workspace);
        let session_id = self.session_id.lock().clone();

        // Apply active LLM profile environment settings
        if let Some(ref active_name) = config.llm.active
            && let Some(profile) = config.llm.endpoints.get(active_name)
        {
            unsafe {
                std::env::set_var("OPENAI_BASE_URL", &profile.base_url);
                std::env::set_var("LLM_BASE_URL", &profile.base_url);
                std::env::set_var("RAD_BASE_URL", &profile.base_url);
                if let Some(key) = profile.resolved_api_key() {
                    std::env::set_var("OPENAI_API_KEY", &key);
                    std::env::set_var("LLM_API_KEY", &key);
                }
                if let Some(ref model) = profile.model {
                    std::env::set_var("OPENAI_MODEL", model);
                    std::env::set_var("LLM_MODEL", model);
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

            crate::log_host!("[DEBUG] Initializing WASM runtimes...");
            let wasm_runtimes = self.get_or_init_runtimes(&event_tx)?;
            crate::log_host!("[DEBUG] Initialized {} WASM runtimes.", wasm_runtimes.len());
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
                crate::log_host!("[DEBUG] No WASM runtimes found, sending init_event to event_tx");
                let _ = event_tx.send(init_event);
            } else {
                crate::log_host!("[DEBUG] Dispatching HumanInputReceived to {} runtimes...", wasm_runtimes.len());
                for (name, runtime_arc) in &wasm_runtimes {
                    let is_orchestrator = {
                        let runtime = runtime_arc.lock();
                        runtime.role == "orchestrator"
                    };
                    if !is_orchestrator {
                        continue;
                    }
                    crate::log_host!("[DEBUG] Calling on_event on runtime '{name}'...");
                    let mut runtime = runtime_arc.lock();
                    if let Err(e) = runtime.on_event(&init_event) {
                        println!("Wasm execution error on {name}: {e}. Recovering...");
                        success = false;
                        break;
                    }
                    crate::log_host!("[DEBUG] on_event on runtime '{name}' returned OK.");
                }
                if !success {
                    self.clear_runtimes()?;
                    attempts += 1;
                    continue;
                }
            }

            crate::log_host!("[DEBUG] Entering process_event_loop...");

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
}
