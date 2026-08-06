// Path resolution and permission matching for `FsSandbox`, split out of
// `fs.rs` to stay under the 300-line file limit. Kept together because every
// function here answers the same question — "which absolute path does this
// input really mean, and is it allowed?" — before any I/O happens.
use std::path::{Path, PathBuf};

impl super::FsSandbox {
    /// Normalizes a path lexically (resolving `..`/`.` without touching the
    /// filesystem), then canonicalizes the longest existing prefix so the
    /// result is comparable against permission patterns even when the target
    /// itself does not exist yet.
    pub(super) fn clean_path(path: &Path) -> Result<PathBuf, String> {
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
    ///
    /// # Errors
    ///
    /// Returns an error if the working directory cannot be read or the path
    /// cannot be canonicalized.
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

    pub(super) fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, String> {
        self.resolve_target_path(path)
    }

    /// Reports whether `path` falls under any of `allowed_patterns`. A literal
    /// `*` allows everything; every other pattern is resolved the same way an
    /// input path is, so the comparison is between two canonical forms.
    pub(super) fn has_permission(
        &self,
        path: &Path,
        allowed_patterns: &[String],
    ) -> Result<bool, String> {
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
}
