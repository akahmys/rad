use std::fs;
use std::path::Path;
use crate::dag::Dag;

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
    let json = serde_json::to_string_pretty(dag)
        .map_err(|e| format!("Failed to serialize DAG: {e}"))?;
    fs::write(&session_file, json)
        .map_err(|e| format!("Failed to write session file: {e}"))?;
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
        return Err(format!("Session file '{}' not found", session_file.display()));
    }
    let json = fs::read_to_string(&session_file)
        .map_err(|e| format!("Failed to read session file: {e}"))?;
    let dag = serde_json::from_str(&json)
        .map_err(|e| format!("Failed to deserialize DAG: {e}"))?;
    Ok(dag)
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
}
