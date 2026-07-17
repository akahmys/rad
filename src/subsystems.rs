use crate::error::UnifiedError;
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub trait FsSubsystem: Send + Sync {
    /// Reads a file.
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails.
    fn file_read(&self, path: &Path) -> Result<Vec<u8>, UnifiedError>;

    /// Writes data to a file.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    fn file_write(&self, path: &Path, data: &[u8]) -> Result<(), UnifiedError>;

    /// Edits a file using a patch.
    ///
    /// # Errors
    ///
    /// Returns an error if patching fails.
    fn file_edit_patch(&self, path: &Path, diff: &str) -> Result<(), UnifiedError>;

    /// Takes a snapshot of specified target paths.
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot creation fails.
    fn take_snapshot(&self, node_id: &str, target_paths: &[PathBuf]) -> Result<(), UnifiedError>;

    /// Checks out a snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if checking out fails.
    fn checkout_snapshot(&self, node_id: &str) -> Result<(), UnifiedError>;

    /// Returns the workspace directory path.
    fn workspace_dir(&self) -> &Path;
}

pub trait ProcessSubsystem: Send + Sync {
    /// Spawns a bash process.
    ///
    /// # Errors
    ///
    /// Returns an error if spawning fails.
    fn spawn_bash_process(
        &self,
        command: &str,
        cwd: Option<&Path>,
        call_id: String,
        name: String,
        arguments: String,
    ) -> Result<crate::process::RunningProcess, UnifiedError>;
}

pub trait DagSubsystem: Send + Sync {
    /// Creates a new node in the DAG.
    ///
    /// # Errors
    ///
    /// Returns an error if creation fails.
    fn create_node(&self, parent_id: &str, node_type: &str) -> Result<String, UnifiedError>;

    /// Sets node text in the DAG.
    ///
    /// # Errors
    ///
    /// Returns an error if setting fails.
    fn set_node_text(&self, node_id: &str, text: &str) -> Result<(), UnifiedError>;

    /// Merges multiple nodes in the DAG.
    ///
    /// # Errors
    ///
    /// Returns an error if merge fails.
    fn merge_nodes(&self, node_ids: &[String], summary_text: &str) -> Result<String, UnifiedError>;

    /// Deletes a node in the DAG.
    ///
    /// # Errors
    ///
    /// Returns an error if deletion fails.
    fn delete_node(&self, node_id: &str) -> Result<(), UnifiedError>;

    /// Gets the current DAG representation.
    ///
    /// # Errors
    ///
    /// Returns an error if getting fails.
    fn get_dag(&self) -> Result<Value, UnifiedError>;
}

pub trait NetworkSubsystem: Send + Sync {
    /// Opens an HTTP stream.
    ///
    /// # Errors
    ///
    /// Returns an error if opening fails.
    fn open_http_stream(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: &str,
        event_tx: std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
        llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    ) -> Result<String, UnifiedError>;
}
