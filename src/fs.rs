use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub struct FsSandbox {
    workspace_dir: PathBuf,
    snapshot_dir: PathBuf,
    fs_read_allow: Vec<String>,
    fs_write_allow: Vec<String>,
}

impl FsSandbox {
    #[must_use]
    pub fn new(
        workspace_dir: PathBuf,
        snapshot_dir: PathBuf,
        fs_read_allow: Vec<String>,
        fs_write_allow: Vec<String>,
    ) -> Self {
        Self {
            workspace_dir,
            snapshot_dir,
            fs_read_allow,
            fs_write_allow,
        }
    }

    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, String> {
        let absolute_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let abs_workspace = if self.workspace_dir.is_absolute() {
                self.workspace_dir.clone()
            } else {
                std::env::current_dir()
                    .map_err(|e| format!("Failed to get current dir: {e}"))?
                    .join(&self.workspace_dir)
            };
            abs_workspace.join(path)
        };

        if absolute_path.exists() {
            absolute_path
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize path: {e}"))
        } else {
            Self::clean_path(&absolute_path)
        }
    }

    fn clean_path(path: &Path) -> Result<PathBuf, String> {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::Normal(c) => {
                    components.push(c);
                }
                std::path::Component::CurDir => {}
                std::path::Component::Prefix(p) => {
                    components.push(p.as_os_str());
                }
                std::path::Component::RootDir => {
                    components.clear();
                    components.push(std::ffi::OsStr::new("/"));
                }
            }
        }
        let cleaned = components.iter().collect::<PathBuf>();
        let mut current = cleaned.as_path();
        while !current.exists() {
            if let Some(parent) = current.parent() {
                current = parent;
            } else {
                break;
            }
        }
        if current.exists() {
            let canonical_parent = current
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize parent: {e}"))?;
            if let Ok(relative) = cleaned.strip_prefix(current) {
                return Ok(canonical_parent.join(relative));
            }
        }
        Ok(cleaned)
    }

    fn has_permission(&self, path: &Path, allowed_patterns: &[String]) -> Result<bool, String> {
        let canonical_path = self.canonicalize_path(path)?;
        for pattern in allowed_patterns {
            if pattern == "*" {
                return Ok(true);
            }
            let pattern_buf = PathBuf::from(pattern);
            let absolute_pattern = if pattern_buf.is_absolute() {
                pattern_buf
            } else {
                self.workspace_dir.join(&pattern_buf)
            };
            let canonical_pattern = if absolute_pattern.exists() {
                absolute_pattern
                    .canonicalize()
                    .map_err(|e| format!("Failed to canonicalize pattern path: {e}"))?
            } else {
                Self::clean_path(&absolute_pattern)?
            };
            if canonical_path.starts_with(&canonical_pattern) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Reads a file from the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if the read permission check fails or file read fails.
    pub fn file_read(&self, path: &Path) -> Result<Vec<u8>, String> {
        let allowed = self.has_permission(path, &self.fs_read_allow)?;
        if !allowed {
            return Err("Read permission denied".to_string());
        }
        let canonical_path = self.canonicalize_path(path)?;
        fs::read(&canonical_path).map_err(|e| format!("Failed to read file: {e}"))
    }

    /// Writes data to a file on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if the write permission check fails or file write fails.
    pub fn file_write(&self, path: &Path, data: &[u8]) -> Result<(), String> {
        let allowed = self.has_permission(path, &self.fs_write_allow)?;
        if !allowed {
            return Err("Write permission denied".to_string());
        }
        let canonical_path = self.canonicalize_path(path)?;
        if let Some(parent) = canonical_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {e}"))?;
        }
        fs::write(&canonical_path, data).map_err(|e| format!("Failed to write file: {e}"))
    }

    /// Patches a file on the filesystem using a unified diff.
    ///
    /// # Errors
    ///
    /// Returns an error if the write permission check fails, patch parsing fails, or patch application fails.
    pub fn file_edit_patch(&self, path: &Path, diff: &str) -> Result<(), String> {
        let allowed = self.has_permission(path, &self.fs_write_allow)?;
        if !allowed {
            return Err("Write permission denied".to_string());
        }
        let canonical_path = self.canonicalize_path(path)?;

        let original = if canonical_path.exists() {
            fs::read_to_string(&canonical_path)
                .map_err(|e| format!("Failed to read file for patching: {e}"))?
        } else {
            String::new()
        };

        let diff_patch = diffy::Patch::from_str(diff)
            .map_err(|e| format!("Failed to parse diff patch: {e}"))?;
        let modified = diffy::apply(&original, &diff_patch)
            .map_err(|e| format!("Failed to apply patch: {e}"))?;

        if let Some(parent) = canonical_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {e}"))?;
        }
        fs::write(&canonical_path, modified).map_err(|e| format!("Failed to write patched file: {e}"))
    }

    /// Takes a snapshot of the specified target paths.
    ///
    /// # Errors
    ///
    /// Returns an error if snapshot directory creation fails or files cannot be copied.
    pub fn take_snapshot(&self, node_id: &str, target_paths: &[PathBuf]) -> Result<(), String> {
        let snapshot_node_dir = self.snapshot_dir.join(node_id);
        if snapshot_node_dir.exists() {
            fs::remove_dir_all(&snapshot_node_dir)
                .map_err(|e| format!("Failed to clean existing snapshot dir: {e}"))?;
        }
        fs::create_dir_all(&snapshot_node_dir)
            .map_err(|e| format!("Failed to create snapshot node dir: {e}"))?;

        for target in target_paths {
            let canonical_target = self.canonicalize_path(target)?;
            let canonical_workspace = self.workspace_dir
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize workspace dir: {e}"))?;
            let relative_target = canonical_target
                .strip_prefix(&canonical_workspace)
                .map_err(|_| "Target path is outside the workspace".to_string())?;
            let dest_path = snapshot_node_dir.join(relative_target);

            if canonical_target.is_dir() {
                Self::copy_dir_all(&canonical_target, &dest_path)?;
            } else if canonical_target.is_file() {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create snapshot parent dir: {e}"))?;
                }
                fs::copy(&canonical_target, &dest_path)
                    .map_err(|e| format!("Failed to copy file to snapshot: {e}"))?;
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
                fs::copy(entry.path(), &dest_path).map_err(|e| format!("Failed to copy file: {e}"))?;
            }
        }
        Ok(())
    }

    /// Restores the snapshot associated with the node ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the snapshot directory does not exist or restore fails.
    pub fn checkout_snapshot(&self, node_id: &str) -> Result<(), String> {
        let snapshot_node_dir = self.snapshot_dir.join(node_id);
        if !snapshot_node_dir.exists() {
            return Err(format!("Snapshot for node {node_id} does not exist"));
        }
        Self::restore_dir_all(&snapshot_node_dir, &snapshot_node_dir, &self.workspace_dir)?;
        Ok(())
    }

    fn restore_dir_all(
        base_snapshot_dir: &Path,
        current_dir: &Path,
        workspace_dir: &Path,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current_dir).map_err(|e| format!("Failed to read snapshot dir: {e}"))? {
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
                fs::copy(&path, &dest_path).map_err(|e| format!("Failed to restore file to workspace: {e}"))?;
            }
        }
        Ok(())
    }
}
