use super::*;
use crate::config::{Config, CoreConfig, ExtensionConfig};
use crate::dag::Dag;
use parking_lot::Mutex;

fn minimal_orchestrator() -> Arc<Orchestrator> {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("workspace");
    let snapshots = temp.path().join("snapshots");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&snapshots).unwrap();

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    let dag = Arc::new(Mutex::new(Dag::new()));
    let orch = Arc::new(Orchestrator::new(
        config,
        "test_session".to_string(),
        dag,
        None,
    ));
    // Keep the tempdir alive for the orchestrator's lifetime by leaking it
    // — these are short-lived unit tests, not a resource-sensitive loop.
    std::mem::forget(temp);
    orch
}

fn add_message(orch: &Arc<Orchestrator>, parent: &str, role: &str, text: &str) -> String {
    let mut dag = orch.dag.lock();
    let id = dag.create_node(parent, role).unwrap();
    dag.set_node_text(&id, text).unwrap();
    id
}

#[test]
fn test_run_compact_reports_nothing_when_too_few_messages() {
    let orch = minimal_orchestrator();
    add_message(&orch, "", "user", "hello");
    let msg = run_compact(&orch);
    assert!(msg.contains("Nothing to compact yet"), "{msg}");
}

#[test]
fn test_run_compact_reports_missing_context_tools() {
    let orch = minimal_orchestrator();
    let n0 = add_message(&orch, "", "user", "goal");
    add_message(&orch, &n0, "assistant", "reply");
    let msg = run_compact(&orch);
    assert!(msg.contains("No extension registered for role"), "{msg}");
}

#[test]
fn test_run_compact_merges_dropped_range_via_real_context_tools() {
    let wasm_path = std::path::Path::new("target/wasm32-wasip2/debug/context_tools.wasm");
    if !wasm_path.exists() {
        return;
    }

    let orch = minimal_orchestrator();
    {
        let mut cfg = orch.config.lock();
        cfg.extensions.push(ExtensionConfig {
            name: "context-tools".to_string(),
            source: wasm_path.to_string_lossy().to_string(),
            enabled: true,
            role: "context-tools".to_string(),
            permissions: None,
            config: std::collections::HashMap::new(),
        });
    }

    let mut ids = Vec::new();
    let goal_id = add_message(&orch, "", "user", "goal message");
    ids.push(goal_id.clone());
    let mut parent = goal_id;
    // Past DEFAULT_MAX_HISTORY (30) so count-based windowing actually
    // drops a range of nodes for /compact to persist.
    for i in 0..40 {
        let role = if i % 2 == 0 { "assistant" } else { "user" };
        let id = add_message(&orch, &parent, role, "turn content");
        ids.push(id.clone());
        parent = id;
    }

    let before_count = orch.dag.lock().nodes.len();
    assert_eq!(before_count, ids.len());

    let msg = run_compact(&orch);
    assert!(msg.contains("Compacted"), "unexpected message: {msg}");

    let after_count = orch.dag.lock().nodes.len();
    assert!(
        after_count < before_count,
        "compaction should have reduced node count: {after_count} vs {before_count}"
    );

    // The goal must survive untouched.
    assert!(orch.dag.lock().nodes.contains_key(&ids[0]));
}
