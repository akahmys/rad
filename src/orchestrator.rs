// test edit]

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
}

impl Orchestrator {
    #[must_use]
    pub fn new(
        config: Config,
        session_id: String,
        dag: Arc<Mutex<Dag>>,
        config_path: Option<String>,
    ) -> Self {
        let sandbox = Arc::new(FsSandbox::new(
            config.core.workspace.clone().into(),
            config.core.snapshot.clone().into(),
            config
                .extensions
                .iter()
                .flat_map(|e| {
                    e.permissions
                        .as_ref()
                        .map(|p| p.fs_read_allow.clone())
                        .unwrap_or_default()
                })
                .collect(),
            config
                .extensions
                .iter()
                .flat_map(|e| {
                    e.permissions
                        .as_ref()
                        .map(|p| p.fs_write_allow.clone())
                        .unwrap_or_default()
                })
                .collect(),
        ));
        let process_manager = Arc::new(ProcessManager::new());
        let active_processes = Arc::new(Mutex::new(HashMap::new()));

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
        }
    }

    /// Resets the current session by saving it and creating a new empty session ID.
    ///
    /// # Errors
    ///
    /// Returns error if saving session fails.
    pub fn reset_session(&self) -> Result<String, String> {
        let old_id = self.session_id.lock().clone();
        let config_guard = self.config.lock();

        // 1. Save the current DAG
        {
            let dag_guard = self.dag.lock();
            crate::session::save_session(&config_guard.core.workspace, &old_id, &dag_guard)?;
        }

        // 2. Generate a new session ID
        let new_id = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string();

        // 3. Update session_id and DAG
        {
            let mut session_guard = self.session_id.lock();
            *session_guard = new_id.clone();
        }

        {
            let mut dag_guard = self.dag.lock();
            *dag_guard = crate::dag::Dag::new();
        }

        // 4. Save the new empty DAG
        {
            let dag_guard = self.dag.lock();
            crate::session::save_session(&config_guard.core.workspace, &new_id, &dag_guard)?;
        }

        // 5. Reset Wasm runtime state
        self.wasm_runtime.lock().clear();

        // 6. Reset token usage
        *self.token_usage.lock() = TokenUsage::default();

        Ok(new_id)
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
        self.sandbox.update_permissions(
            new_cfg
                .extensions
                .iter()
                .flat_map(|e| {
                    e.permissions
                        .as_ref()
                        .map(|p| p.fs_read_allow.clone())
                        .unwrap_or_default()
                })
                .collect(),
            new_cfg
                .extensions
                .iter()
                .flat_map(|e| {
                    e.permissions
                        .as_ref()
                        .map(|p| p.fs_write_allow.clone())
                        .unwrap_or_default()
                })
                .collect(),
        );

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

        {
            let mut guard = self.running_task.lock();
            if let Some(handle) = guard.take() {
                let _ = handle.join();
            }
        }

        let mut dag_guard = self.dag.lock();
        if !dag_guard.nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found in DAG"));
        }

        self.sandbox.checkout_snapshot(node_id)?;
        dag_guard.current_node_id = Some(node_id.to_string());

        Ok(())
    }

    /// Aborts the currently running task.
    pub fn abort(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
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
