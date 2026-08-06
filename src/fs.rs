use std::fs;
use std::path::{Path, PathBuf};


#[cfg(test)]
mod tests;

use parking_lot::Mutex;

pub struct FsSandbox {
    workspace_dir: PathBuf,
    snapshot_dir: PathBuf,
    fs_read_allow: Mutex<Vec<String>>,
    fs_write_allow: Mutex<Vec<String>>,
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
            fs_read_allow: Mutex::new(fs_read_allow),
            fs_write_allow: Mutex::new(fs_write_allow),
        }
    }

    pub fn update_permissions(&self, read: Vec<String>, write: Vec<String>) {
        *self.fs_read_allow.lock() = read;
        *self.fs_write_allow.lock() = write;
    }

    #[must_use]
    pub fn workspace_dir(&self) -> &Path {
        &self.workspace_dir
    }

    fn clean_path(path: &Path) -> Result<PathBuf, String> {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if let Some(last) = components.last() {
                        match last {
                            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                                // Do not pop root or prefix
                            }
                            _ => {
                                components.pop();
                            }
                        }
                    }
                }
                std::path::Component::Normal(c) => {
                    components.push(std::path::Component::Normal(c));
                }
                std::path::Component::CurDir => {}
                std::path::Component::Prefix(p) => {
                    components.push(std::path::Component::Prefix(p));
                }
                std::path::Component::RootDir => {
                    components.clear();
                    components.push(std::path::Component::RootDir);
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

    /// Resolves, expands tildes, and canonicalizes raw input paths cleanly.
    pub fn resolve_target_path(&self, raw_path: &Path) -> Result<PathBuf, String> {
        let expanded = crate::config::expand_tilde(&raw_path.to_string_lossy());
        let absolute = if expanded.is_absolute() {
            expanded.to_path_buf()
        } else {
            let abs_workspace = if self.workspace_dir.is_absolute() {
                self.workspace_dir.clone()
            } else {
                std::env::current_dir()
                    .map_err(|e| format!("Failed to get current dir: {e}"))?
                    .join(&self.workspace_dir)
            };
            abs_workspace.join(&expanded)
        };

        if absolute.exists() {
            absolute
                .canonicalize()
                .map_err(|e| format!("Failed to canonicalize path: {e}"))
        } else {
            Self::clean_path(&absolute)
        }
    }

    fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, String> {
        self.resolve_target_path(path)
    }

    fn has_permission(&self, path: &Path, allowed_patterns: &[String]) -> Result<bool, String> {
        let canonical_path = self.resolve_target_path(path)?;
        for pattern in allowed_patterns {
            if pattern == "*" {
                return Ok(true);
            }
            let pattern_buf = PathBuf::from(pattern);
            let expanded_pattern = crate::config::expand_tilde(&pattern_buf.to_string_lossy());
            let absolute_pattern = if expanded_pattern.is_absolute() {
                expanded_pattern.to_path_buf()
            } else {
                self.workspace_dir.join(&expanded_pattern)
            };
            let canonical_pattern = Self::clean_path(&absolute_pattern)?;
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
    pub fn file_read(&self, path: &Path) -> Result<Vec<u8>, crate::error::UnifiedError> {
        let resolved = self
            .resolve_target_path(path)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        let read_allow = self.fs_read_allow.lock();
        let allowed = self
            .has_permission(&resolved, &read_allow)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        if !allowed {
            return Err(crate::error::UnifiedError::l2(
                format!("Read permission denied for path: {:?}", path),
                "FsPermission",
            ));
        }
        fs::read(&resolved)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to read file: {e}"), "Fs"))
    }

    /// Lists the entry names of a directory (non-recursive).
    ///
    /// # Errors
    ///
    /// Returns an error if the read permission check fails or the
    /// directory can't be read.
    pub fn dir_list(&self, path: &Path) -> Result<Vec<String>, crate::error::UnifiedError> {
        let resolved = self
            .resolve_target_path(path)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        let read_allow = self.fs_read_allow.lock();
        let allowed = self
            .has_permission(&resolved, &read_allow)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        if !allowed {
            return Err(crate::error::UnifiedError::l2(
                format!("Read permission denied for path: {:?}", path),
                "FsPermission",
            ));
        }
        let entries = fs::read_dir(&resolved).map_err(|e| {
            crate::error::UnifiedError::l1(format!("Failed to read directory: {e}"), "Fs")
        })?;
        Ok(entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect())
    }

    /// Writes data to a file on the filesystem.
    ///
    /// # Errors
    ///
    /// Returns an error if the write permission check fails or file write fails.
    pub fn file_write(&self, path: &Path, data: &[u8]) -> Result<(), crate::error::UnifiedError> {
        let resolved = self
            .resolve_target_path(path)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        let write_allow = self.fs_write_allow.lock();
        let allowed = self
            .has_permission(&resolved, &write_allow)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        if !allowed {
            return Err(crate::error::UnifiedError::l2(
                format!("Write permission denied for path: {:?}", path),
                "FsPermission",
            ));
        }
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::error::UnifiedError::l1(
                    format!("Failed to create parent directory: {e}"),
                    "Fs",
                )
            })?;
        }
        fs::write(&resolved, data)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Failed to write file: {e}"), "Fs"))
    }

    /// Patches a file on the filesystem using a unified diff.
    ///
    /// # Errors
    ///
    /// Returns an error if the write permission check fails, patch parsing fails, or patch application fails.
    pub fn file_edit_patch(
        &self,
        path: &Path,
        diff: &str,
    ) -> Result<(), crate::error::UnifiedError> {
        let resolved = self
            .resolve_target_path(path)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        let write_allow = self.fs_write_allow.lock();
        let allowed = self
            .has_permission(&resolved, &write_allow)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Fs"))?;
        if !allowed {
            return Err(crate::error::UnifiedError::l2(
                format!("Write permission denied for path: {:?}", path),
                "FsPermission",
            ));
        }
        let original_text = if resolved.exists() {
            fs::read_to_string(&resolved).map_err(|e| {
                crate::error::UnifiedError::l1(
                    format!("Failed to read file for patching: {e}"),
                    "Fs",
                )
            })?
        } else {
            String::new()
        };

        let diff_patch = diffy::Patch::from_str(diff).map_err(|e| {
            crate::error::UnifiedError::l2(format!("Failed to parse diff patch: {e}"), "FsPatch")
        })?;
        let modified = diffy::apply(&original_text, &diff_patch).map_err(|e| {
            crate::error::UnifiedError::l2(format!("Failed to apply patch: {e}"), "FsPatch")
        })?;

        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::error::UnifiedError::l1(
                    format!("Failed to create parent directory: {e}"),
                    "Fs",
                )
            })?;
        }
        fs::write(&resolved, modified).map_err(|e| {
            crate::error::UnifiedError::l1(format!("Failed to write patched file: {e}"), "Fs")
        })
    }
}

mod snapshot;

impl crate::subsystems::FsSubsystem for FsSandbox {
    fn file_read(&self, path: &Path) -> Result<Vec<u8>, crate::error::UnifiedError> {
        self.file_read(path)
    }

    fn dir_list(&self, path: &Path) -> Result<Vec<String>, crate::error::UnifiedError> {
        self.dir_list(path)
    }

    fn file_write(&self, path: &Path, data: &[u8]) -> Result<(), crate::error::UnifiedError> {
        self.file_write(path, data)
    }

    fn file_edit_patch(&self, path: &Path, diff: &str) -> Result<(), crate::error::UnifiedError> {
        self.file_edit_patch(path, diff)
    }

    fn take_snapshot(
        &self,
        node_id: &str,
        target_paths: &[PathBuf],
    ) -> Result<(), crate::error::UnifiedError> {
        self.take_snapshot(node_id, target_paths)
    }

    fn checkout_snapshot(&self, node_id: &str) -> Result<(), crate::error::UnifiedError> {
        self.checkout_snapshot(node_id)
    }

    fn workspace_dir(&self) -> &Path {
        self.workspace_dir()
    }
}
