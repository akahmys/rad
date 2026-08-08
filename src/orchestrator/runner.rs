use super::Orchestrator;
use crate::git;
use crate::ipc::RasCoreEvent;
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Sender, channel};

// Runtime lifecycle (get_or_init_runtimes/clear_runtimes) and the RPC
// verification / event-dispatch loop live in sibling files to keep this one
// under the 300-line limit.
mod events;
mod runtimes;

/// How many times a crashed run is rebuilt from scratch before giving up.
const MAX_ATTEMPTS: u32 = 2;

type Runtimes = HashMap<String, Arc<Mutex<WasmRuntime>>>;

/// Git-autopilot state for one task, resolved once before the retry loop.
///
/// Gathered into a struct because the retry loop is per-attempt while all of
/// this is per-task: passing the four separately would invite an attempt to
/// re-resolve them and silently checkpoint against a different baseline.
struct Autopilot {
    workspace: PathBuf,
    has_git: bool,
    initial_sha: Option<String>,
    verification_command: Option<String>,
}

impl Autopilot {
    fn new(config: &crate::config::Config, session_id: &str) -> Self {
        let workspace = PathBuf::from(&config.core.workspace);
        let (has_git, initial_sha) =
            crate::orchestrator::autopilot::setup_git_autopilot(&workspace, session_id);
        Self {
            workspace,
            has_git,
            initial_sha,
            verification_command: config.core.verification_command.clone(),
        }
    }

    /// Runs the configured verification command, checkpointing on success and
    /// rolling the codebase back on failure. A task with no command configured
    /// verifies trivially.
    fn verify(&self) -> Result<(), String> {
        let Some(ref verify_cmd) = self.verification_command else {
            return Ok(());
        };
        println!("Running autopilot verification: {verify_cmd}");

        if crate::orchestrator::autopilot::run_verification_cmd(&self.workspace, verify_cmd) {
            println!("Verification PASSED.");
            if self.has_git {
                let _ = git::create_checkpoint(&self.workspace, "verification_passed");
            }
            return Ok(());
        }

        if let Some(ref sha) = self.initial_sha {
            println!("Verification FAILED. Rolling back codebase to stable SHA: {sha}");
            let _ = git::rollback_to_checkpoint(&self.workspace, sha);
        }
        Err("Autopilot verification command failed. Codebase rolled back.".to_string())
    }
}

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

    /// Drives the task, rebuilding the runtimes and retrying if one crashes.
    fn run_task_internal(self: &Arc<Self>, instruction: &str) -> Result<(), String> {
        crate::log_host!("[DEBUG] Starting run_task_internal: instruction = '{instruction}'");
        let config = self.config.lock().clone();
        let session_id = self.session_id.lock().clone();
        let autopilot = Autopilot::new(&config, &session_id);

        for attempt in 0..MAX_ATTEMPTS {
            if self.abort_flag.load(Ordering::SeqCst) {
                return Err("Task aborted by user".to_string());
            }
            if self
                .run_attempt(instruction, attempt, &autopilot)?
                .is_break()
            {
                return Ok(());
            }
        }

        Err("Wasm execution failed after maximum recovery attempts".to_string())
    }

    /// One pass: build the runtimes, hand them the instruction, run the event
    /// loop. `Break` means the task is finished; `Continue` means something
    /// crashed recoverably and the runtimes have already been cleared.
    fn run_attempt(
        self: &Arc<Self>,
        instruction: &str,
        attempt: u32,
        autopilot: &Autopilot,
    ) -> Result<ControlFlow<()>, String> {
        let (event_tx, event_rx) = channel::<RasCoreEvent>();
        let wasm_runtimes = self.runtimes_wired_to(&event_tx)?;

        // Only a retry has state to restore; the first attempt starts clean.
        if attempt > 0 && !self.rehydrate(&wasm_runtimes) {
            self.clear_runtimes();
            return Ok(ControlFlow::Continue(()));
        }

        if !Self::dispatch_instruction(instruction, &wasm_runtimes, &event_tx) {
            self.clear_runtimes();
            return Ok(ControlFlow::Continue(()));
        }

        crate::log_host!("[DEBUG] Entering process_event_loop...");
        match self.process_event_loop(&event_rx, &wasm_runtimes) {
            Ok(()) => {
                autopilot.verify()?;
                Ok(ControlFlow::Break(()))
            }
            Err(e) => {
                // An abort is the user's decision, not a crash to recover from.
                if self.abort_flag.load(Ordering::SeqCst) {
                    return Err("Task aborted by user".to_string());
                }
                println!("Wasm runtime crashed: {e}. Recovering...");
                self.clear_runtimes();
                Ok(ControlFlow::Continue(()))
            }
        }
    }

    /// The live runtimes, each pointed at this attempt's event channel.
    ///
    /// Every attempt opens a fresh channel, so the wiring has to be redone even
    /// when the runtimes themselves were reused — a runtime still holding the
    /// previous attempt's sender would emit into a channel nobody reads.
    fn runtimes_wired_to(
        self: &Arc<Self>,
        event_tx: &Sender<RasCoreEvent>,
    ) -> Result<Runtimes, String> {
        crate::log_host!("[DEBUG] Initializing WASM runtimes...");
        let wasm_runtimes = self.get_or_init_runtimes(event_tx)?;
        crate::log_host!("[DEBUG] Initialized {} WASM runtimes.", wasm_runtimes.len());
        for runtime_arc in wasm_runtimes.values() {
            runtime_arc.lock().set_event_tx(event_tx.clone());
        }
        Ok(wasm_runtimes)
    }

    /// Tells every runtime what was in flight when the previous attempt died.
    /// `false` means one of them could not take it, so the set is unusable.
    fn rehydrate(&self, wasm_runtimes: &Runtimes) -> bool {
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
        for (name, runtime_arc) in wasm_runtimes {
            if let Err(e) = runtime_arc.lock().on_event(&rehydrate_event) {
                eprintln!("Failed to rehydrate runtime {name}: {e}");
                return false;
            }
        }
        true
    }

    /// Hands the instruction to the orchestrator runtime. `false` means it
    /// failed and the set has to be rebuilt.
    ///
    /// With no runtimes at all the event goes straight onto the channel: the
    /// event loop is what actually serves a task, and it runs either way.
    fn dispatch_instruction(
        instruction: &str,
        wasm_runtimes: &Runtimes,
        event_tx: &Sender<RasCoreEvent>,
    ) -> bool {
        let init_event = RasCoreEvent::HumanInputReceived {
            text: instruction.to_string(),
        };

        if wasm_runtimes.is_empty() {
            crate::log_host!("[DEBUG] No WASM runtimes found, sending init_event to event_tx");
            let _ = event_tx.send(init_event);
            return true;
        }

        crate::log_host!(
            "[DEBUG] Dispatching HumanInputReceived to {} runtimes...",
            wasm_runtimes.len()
        );
        for (name, runtime_arc) in wasm_runtimes {
            if runtime_arc.lock().role != "orchestrator" {
                continue;
            }
            crate::log_host!("[DEBUG] Calling on_event on runtime '{name}'...");
            if let Err(e) = runtime_arc.lock().on_event(&init_event) {
                println!("Wasm execution error on {name}: {e}. Recovering...");
                return false;
            }
            crate::log_host!("[DEBUG] on_event on runtime '{name}' returned OK.");
        }
        true
    }
}
