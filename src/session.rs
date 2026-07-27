use crate::dag::Dag;
use std::fs;
use std::path::Path;

/// Saves the session DAG state to the .rad/sessions/<session_id>.json file.
///
/// # Errors
///
/// Returns an error if directory creation or file writing fails.
pub fn save_session(workspace: &str, session_id: &str, dag: &Dag) -> Result<(), String> {
    let sessions_dir = Path::new(workspace).join(".rad").join("sessions");
    if !sessions_dir.exists() {
        fs::create_dir_all(&sessions_dir)
            .map_err(|e| format!("Failed to create sessions directory: {e}"))?;
    }
    let session_file = sessions_dir.join(format!("{session_id}.json"));
    let json =
        serde_json::to_string_pretty(dag).map_err(|e| format!("Failed to serialize DAG: {e}"))?;
    fs::write(&session_file, json).map_err(|e| format!("Failed to write session file: {e}"))?;
    Ok(())
}

/// Loads the session DAG state from the .rad/sessions/<session_id>.json file.
///
/// # Errors
///
/// Returns an error if the session file does not exist, reading, or deserialization fails.
pub fn load_session(workspace: &str, session_id: &str) -> Result<Dag, String> {
    let session_file = Path::new(workspace)
        .join(".rad")
        .join("sessions")
        .join(format!("{session_id}.json"));
    if !session_file.exists() {
        return Err(format!(
            "Session file '{}' not found",
            session_file.display()
        ));
    }
    let json = fs::read_to_string(&session_file)
        .map_err(|e| format!("Failed to read session file: {e}"))?;
    let dag = serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize DAG: {e}"))?;
    Ok(dag)
}

/// Deletes session files beyond the `keep` most-recently-modified ones, so
/// `.rad/sessions/` doesn't accumulate unbounded (Phase 50-1 — 150+ files
/// were observed in this repo's own `.rad/` during investigation; nothing
/// else ever deletes session files). `keep_id`'s file is never deleted
/// regardless of its age or position, since it's the session about to be
/// (or already) resumed and may be written to again before this process
/// exits. Best-effort: I/O errors (missing directory, permission issues)
/// are silently ignored rather than surfaced, since pruning is a hygiene
/// nicety, not something a session start should fail over.
pub fn prune_sessions(workspace: &str, keep: usize, keep_id: &str) {
    let sessions_dir = Path::new(workspace).join(".rad").join("sessions");
    let Ok(entries) = fs::read_dir(&sessions_dir) else {
        return;
    };

    let mut candidates: Vec<(std::path::PathBuf, std::time::SystemTime)> = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .filter(|entry| entry.path().file_stem().and_then(|s| s.to_str()) != Some(keep_id))
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            Some((entry.path(), modified))
        })
        .collect();

    if candidates.len() <= keep {
        return;
    }

    candidates.sort_by_key(|(_, modified)| std::cmp::Reverse(*modified));
    for (path, _) in candidates.into_iter().skip(keep) {
        let _ = fs::remove_file(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_session() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().to_str().unwrap();
        let session_id = "test_session_id";

        let mut dag = Dag::new();
        let node_id = dag.create_node("", "root").unwrap();
        dag.set_node_text(&node_id, "hello world").unwrap();

        // Save
        save_session(workspace, session_id, &dag).unwrap();

        // Load
        let loaded = load_session(workspace, session_id).unwrap();
        assert_eq!(loaded.current_node_id, Some(node_id.clone()));
        assert_eq!(loaded.nodes.get(&node_id).unwrap().text, "hello world");
    }

    fn touch_session(workspace: &str, id: &str) {
        save_session(workspace, id, &Dag::new()).unwrap();
    }

    #[test]
    fn test_prune_sessions_keeps_only_the_newest_n() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().to_str().unwrap();

        for i in 0..5 {
            touch_session(workspace, &format!("s{i}"));
            // Ensure distinct mtimes so ordering is deterministic across
            // filesystems with coarse mtime resolution.
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        prune_sessions(workspace, 2, "keep-me-not-present");

        let sessions_dir = Path::new(workspace).join(".rad").join("sessions");
        let mut remaining: Vec<_> = fs::read_dir(&sessions_dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        remaining.sort();
        assert_eq!(remaining, vec!["s3.json".to_string(), "s4.json".to_string()]);
    }

    #[test]
    fn test_prune_sessions_never_deletes_the_current_session() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().to_str().unwrap();

        touch_session(workspace, "old");
        std::thread::sleep(std::time::Duration::from_millis(10));
        touch_session(workspace, "current");

        // "current" is the oldest by mtime here (created second, but treat
        // it as the resumed session): even with keep=0 it must survive.
        prune_sessions(workspace, 0, "current");

        assert!(load_session(workspace, "current").is_ok());
    }

    #[test]
    fn test_prune_sessions_is_a_noop_under_the_limit() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().to_str().unwrap();
        touch_session(workspace, "only");

        prune_sessions(workspace, 50, "unrelated");

        assert!(load_session(workspace, "only").is_ok());
    }

    #[test]
    fn test_prune_sessions_missing_directory_is_a_noop() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().to_str().unwrap();
        // No sessions directory exists at all — must not panic.
        prune_sessions(workspace, 1, "whatever");
    }
}
