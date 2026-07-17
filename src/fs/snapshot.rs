use std::fs;
use std::path::{Path, PathBuf};

impl super::FsSandbox {
    /// Takes a snapshot of the specified target paths.
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot directory creation fails or files cannot be copied.
    pub fn take_snapshot(&self, node_id: &str, target_paths: &[PathBuf]) -> Result<(), crate::error::UnifiedError> {
        let snapshot_node_dir = self.snapshot_dir.join(node_id);
        if snapshot_node_dir.exists() {
            fs::remove_dir_all(&snapshot_node_dir)
                .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to clean existing snapshot dir: {e}"), "FsSnapshot"))?;
        }
        fs::create_dir_all(&snapshot_node_dir)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to create snapshot node dir: {e}"), "FsSnapshot"))?;

        for target in target_paths {
            let canonical_target = self.canonicalize_path(target)
                .map_err(|e| crate::error::UnifiedError::l1(e, "FsSnapshot"))?;
            let canonical_workspace = self
                .workspace_dir
                .canonicalize()
                .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to canonicalize workspace dir: {e}"), "FsSnapshot"))?;
            let relative_target = canonical_target
                .strip_prefix(&canonical_workspace)
                .map_err(|_| crate::error::UnifiedError::l1("Target path is outside the workspace", "FsSnapshot"))?;
            let dest_path = snapshot_node_dir.join(relative_target);

            if canonical_target.is_dir() {
                Self::copy_dir_all(&canonical_target, &dest_path)
                    .map_err(|e| crate::error::UnifiedError::l1(e, "FsSnapshot"))?;
            } else if canonical_target.is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to create snapshot parent dir: {e}"), "FsSnapshot"))?;
                }
                fs::copy(&canonical_target, &dest_path)
                    .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to copy file to snapshot: {e}"), "FsSnapshot"))?;
            }
        }
        Ok(())
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
        fs::create_dir_all(dst).map_err(|e| format!("Failed to create destination dir: {e}"))?;
        for entry in fs::read_dir(src).map_err(|e| format!("Failed to read source dir: {e}"))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to get file type: {e}"))?;
            let dest_path = dst.join(entry.file_name());
            if file_type.is_dir() {
                Self::copy_dir_all(&entry.path(), &dest_path)?;
            } else {
                fs::copy(entry.path(), &dest_path)
                    .map_err(|e| format!("Failed to copy file: {e}"))?;
            }
        }
        Ok(())
    }

    /// Restores the snapshot associated with the node ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot directory does not exist or restore fails.
    pub fn checkout_snapshot(&self, node_id: &str) -> Result<(), crate::error::UnifiedError> {
        let snapshot_node_dir = self.snapshot_dir.join(node_id);
        if !snapshot_node_dir.exists() {
            return Err(crate::error::UnifiedError::l1(format!("Snapshot for node {node_id} does not exist"), "FsSnapshot"));
        }
        Self::restore_dir_all(&snapshot_node_dir, &snapshot_node_dir, &self.workspace_dir)
            .map_err(|e| crate::error::UnifiedError::l1(e, "FsSnapshot"))?;
        Ok(())
    }

    fn restore_dir_all(
        base_snapshot_dir: &Path,
        current_dir: &Path,
        workspace_dir: &Path,
    ) -> Result<(), String> {
        for entry in
            fs::read_dir(current_dir).map_err(|e| format!("Failed to read snapshot dir: {e}"))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
            let path = entry.path();
            let relative = path
                .strip_prefix(base_snapshot_dir)
                .map_err(|e| format!("Failed to calculate relative path: {e}"))?;
            let dest_path = workspace_dir.join(relative);

            if path.is_dir() {
                fs::create_dir_all(&dest_path)
                    .map_err(|e| format!("Failed to create directory in workspace: {e}"))?;
                Self::restore_dir_all(base_snapshot_dir, &path, workspace_dir)?;
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {e}"))?;
                }
                fs::copy(&path, &dest_path)
                    .map_err(|e| format!("Failed to restore file to workspace: {e}"))?;
            }
        }
        Ok(())
    }
}
