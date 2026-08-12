#![deny(clippy::pedantic)]

use crate::config::Config;
use crate::dag::Dag;
use crate::fs::FsSandbox;
use crate::process::{ProcessManager, RunningProcess};
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Default, Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

pub struct Orchestrator {
    pub(crate) config: Mutex<Config>,
    config_path: Option<String>,
    sandbox: Arc<FsSandbox>,
    process_manager: Arc<ProcessManager>,
    pub dag: Arc<Mutex<Dag>>,
    active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
    pub session_id: Mutex<String>,
    pub(crate) wasm_runtime: Mutex<HashMap<String, Arc<Mutex<WasmRuntime>>>>,
    running_task: Mutex<Option<std::thread::JoinHandle<Result<(), String>>>>,
    abort_flag: Arc<AtomicBool>,
    pub token_usage: Arc<Mutex<TokenUsage>>,
    /// The kernel, once booted. `None` until `main` brings it up, and while no
    /// modules are configured. Held here because the RPC handlers reach the
    /// world through `RpcContext.orchestrator`, and during the migration a
    /// `CallExtension` has to be answerable by either surface.
    pub kernel: Mutex<Option<Arc<crate::kernel::KernelShared>>>,
}

/// A fresh session id: seconds since the epoch, which is also what orders
/// sessions on disk.
fn new_session_id() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
        .to_string()
}

/// The union of every extension's filesystem allow-lists, read and write.
///
/// Shared by construction and by `reload`, which have to agree: a sandbox
/// rebuilt from a different rule than it was created with would widen or narrow
/// access on a config reload for no reason anyone could see.
fn fs_allow_lists(extensions: &[crate::config::ExtensionConfig]) -> (Vec<String>, Vec<String>) {
    let mut read = Vec::new();
    let mut write = Vec::new();
    for permissions in extensions.iter().filter_map(|e| e.permissions.as_ref()) {
        read.extend(permissions.fs_read_allow.iter().cloned());
        write.extend(permissions.fs_write_allow.iter().cloned());
    }
    (read, write)
}

/// Brings the kernel up and reports what loaded.
///
/// Called during construction rather than from `main`, because `modules` is
/// part of the config and an Orchestrator built from a config that declares
/// them but silently has none is a trap — every integration test hit it, and
/// each one had to remember to boot the kernel by hand. With nothing configured
/// this loads nothing and costs nothing.
fn boot_kernel(config: &Config) -> Arc<crate::kernel::KernelShared> {
    let (kernel, loaded) = crate::kernel::boot(config);
    if !loaded.is_empty() {
        crate::log_host!(
            "[kernel] loaded {} module(s): {}",
            loaded.len(),
            loaded.join(", ")
        );
    }
    kernel
}

impl Orchestrator {
    #[must_use]
    pub fn new(
        config: Config,
        session_id: String,
        dag: Arc<Mutex<Dag>>,
        config_path: Option<String>,
    ) -> Self {
        let (fs_read_allow, fs_write_allow) = fs_allow_lists(&config.extensions);
        let sandbox = Arc::new(FsSandbox::new(
            config.core.workspace.clone().into(),
            config.core.snapshot.clone().into(),
            fs_read_allow,
            fs_write_allow,
        ));
        let process_manager = Arc::new(ProcessManager::new());
        let active_processes = Arc::new(Mutex::new(HashMap::new()));
        let kernel = boot_kernel(&config);
        // After boot, because `boot` builds the kernel before any module can
        // ask. Handing it the same `Arc` the orchestrator holds rather than a
        // copy: `kernel.dag` must answer with the live conversation, not a
        // snapshot taken at startup.
        *kernel.dag.lock() = Some(Arc::clone(&dag));
        // The module owns the graph when one is loaded, so it has to be told
        // which session it is holding. Same file the host reads, so both agree
        // at boot and diverge only if one of them stops going through the
        // other — which is what `dag_module_bridge_tests` watches for.
        // The terminal is a process-wide singleton with no context to thread a
        // handle through, so it is given the kernel here rather than at each call.
        crate::terminal::get_terminal().attach_kernel(Arc::clone(&kernel));
        if kernel.provider_of("dag.open").is_some() {
            // No workspace: the kernel preopens it as the module's `.`, so a
            // host-absolute path would name a different directory inside the
            // guest's view.
            let payload = serde_json::json!({ "session_id": session_id }).to_string();
            if let Err(e) = kernel.call("host", "dag", "dag.open", &payload) {
                eprintln!("\x1b[33mWarning: dag module could not open the session: {e}\x1b[0m");
            }
        }
        // After boot, because `boot` builds the kernel before any module can
        // ask. Handing it the same `Arc` the orchestrator holds rather than a
        // copy: `kernel.dag` must answer with the live conversation, not a
        // snapshot taken at startup.

        Self {
            config: Mutex::new(config),
            config_path,
            sandbox,
            process_manager,
            dag,
            active_processes,
            session_id: Mutex::new(session_id),
            wasm_runtime: Mutex::new(HashMap::new()),
            running_task: Mutex::new(None),
            abort_flag: Arc::new(AtomicBool::new(false)),
            token_usage: Arc::new(Mutex::new(TokenUsage::default())),
            kernel: Mutex::new(Some(kernel)),
        }
    }

    /// The conversation, from whoever owns it.
    ///
    /// The single read path. Callers used to lock `self.dag` directly, which
    /// was correct only while the host owned the graph; with a `dag` module
    /// loaded that field is a cache, and stage 9 is removing it. Going through
    /// here means the last step — deleting the field — touches this function
    /// and not every reader.
    ///
    /// # Panics
    ///
    /// Never: a module that answers with something unreadable falls back to the
    /// host's copy rather than failing a `/tree` or a `/compact`.
    #[must_use]
    pub fn conversation(&self) -> crate::dag::Dag {
        if let Some(kernel) = self.kernel.lock().clone()
            && kernel.provider_of("dag.get").is_some()
            && let Ok(reply) = kernel.call("host", "dag", "dag.get", "{}")
            && let Ok(dag) = serde_json::from_str(&reply)
        {
            return dag;
        }
        self.dag.lock().clone()
    }

    /// Runs one operation on the `dag` module, if one is loaded.
    ///
    /// A no-op without a module, which is the fallback every rad that does not
    /// configure one runs. Failures are logged rather than propagated: these
    /// calls sit alongside a host-side change that has already happened, and
    /// failing the caller would leave the two further apart, not closer.
    fn on_dag_module(&self, method: &str, payload: &serde_json::Value) {
        if let Some(Err(e)) = self.try_dag_module(method, payload) {
            crate::log_host!("[HOST] {method} failed: {e}");
        }
    }

    /// The same call, with the outcome handed back.
    ///
    /// `None` still means "no module"; the two exist separately because most
    /// callers sit beside a host-side change that has already happened and have
    /// nothing useful to do with a failure, while `/compact` is reporting to a
    /// person and does.
    pub(crate) fn try_dag_module(
        &self,
        method: &str,
        payload: &serde_json::Value,
    ) -> Option<Result<String, String>> {
        let kernel = self.kernel.lock().clone()?;
        kernel.provider_of(method)?;
        Some(kernel.call("host", "dag", method, &payload.to_string()))
    }

    /// Resets the current session by saving it and creating a new empty session ID.
    ///
    /// # Errors
    ///
    /// Returns error if saving session fails.
    pub fn reset_session(&self) -> Result<String, String> {
        let old_id = self.session_id.lock().clone();
        let config_guard = self.config.lock();
        let workspace = &config_guard.core.workspace;

        // Save the session that is ending before anything is cleared.
        crate::session::save_session(workspace, &old_id, &self.dag.lock())?;

        let new_id = new_session_id();
        self.session_id.lock().clone_from(&new_id);
        *self.dag.lock() = crate::dag::Dag::new();
        // The module owns the graph when one is loaded, so clearing the host's
        // copy alone leaves it holding the old conversation — which the next
        // mutation copies straight back over the cache. Opening the new session
        // does both halves: the module loads an empty graph *and* starts
        // writing to the new file. A separate "clear" step is worse than
        // useless here — it saves through the still-open handle, overwriting
        // the session that was just archived, which
        // `after_a_reset_the_module_writes_to_the_new_session_not_the_old_one`
        // caught.
        self.on_dag_module(
            "dag.open",
            &serde_json::json!({ "session_id": new_id.clone() }),
        );

        // And save the empty one, so the new id exists on disk immediately.
        crate::session::save_session(workspace, &new_id, &self.dag.lock())?;

        self.discard_session_state();
        Ok(new_id)
    }

    /// Drops everything scoped to the session that just ended. Runtimes are
    /// rebuilt on the next task; processes and token counts belong to the old
    /// conversation and would otherwise be attributed to the new one.
    fn discard_session_state(&self) {
        self.wasm_runtime.lock().clear();
        self.active_processes.lock().clear();
        *self.token_usage.lock() = TokenUsage::default();
    }

    /// Dynamically reloads configuration from `config_path`.
    ///
    /// # Errors
    ///
    /// Returns error if reloading fails.
    pub fn reload(&self) -> Result<(), String> {
        let new_cfg = crate::config::load_config(self.config_path.as_deref())
            .map_err(|e| format!("Failed to load configuration: {e}"))?;

        // 1. Overwrite config
        {
            let mut config_guard = self.config.lock();
            *config_guard = new_cfg.clone();
        }

        // 2. Update sandbox file system permissions
        let (fs_read_allow, fs_write_allow) = fs_allow_lists(&new_cfg.extensions);
        self.sandbox
            .update_permissions(fs_read_allow, fs_write_allow);

        // 3. Reset Wasm runtime state so it gets re-initialized with new configs on next run
        self.wasm_runtime.lock().clear();

        Ok(())
    }

    /// Checks if a task is currently executing.
    pub fn is_running(&self) -> bool {
        let mut guard = self.running_task.lock();
        if let Some(ref handle) = *guard {
            if handle.is_finished() {
                *guard = None;
                return false;
            }
            return true;
        }
        false
    }

    /// Rolls back the session state (DAG and filesystem sandbox) to the specified node.
    ///
    /// # Errors
    ///
    /// Returns an error if the node ID does not exist in the DAG or filesystem rollback fails.
    pub fn rollback(&self, node_id: &str) -> Result<(), String> {
        self.abort_flag.store(true, Ordering::SeqCst);

        {
            let mut wasm_guard = self.wasm_runtime.lock();
            wasm_guard.clear();
        }

        self.active_processes.lock().clear();

        {
            let mut guard = self.running_task.lock();
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }

        if !self.conversation().nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found in DAG"));
        }

        self.sandbox.checkout_snapshot(node_id)?;
        self.dag.lock().current_node_id = Some(node_id.to_string());
        // Same reason as `reset_session`: the module still points at the old
        // tip otherwise, and the next turn parents off it and undoes the
        // rollback a turn later, where it looks like nothing to do with this.
        self.on_dag_module(
            "dag.set_current",
            &serde_json::json!({ "node_id": node_id }),
        );

        Ok(())
    }

    /// Checks if the orchestrator task has been aborted.
    pub fn is_aborted(&self) -> bool {
        self.abort_flag.load(Ordering::SeqCst)
    }

    /// Aborts the currently running task.
    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
        {
            let mut procs = self.active_processes.lock();
            for proc in procs.values_mut() {
                proc.kill_group();
            }
            procs.clear();
        }
        let mut guard = self.running_task.lock();
        if let Some(handle) = guard.take() {
            let _ = handle.join();
        }
    }
}

pub mod autopilot;
pub mod runner;

#[cfg(test)]
mod tests;
