use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

#[test]
fn test_async_task_cancellation_on_rollback() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec![],
        }),
        network: Some(rad::config::NetworkConfig {
            allow_network: true,
            allow_domains: vec!["127.0.0.1".to_string()],
        }),
        ..Default::default()
    };

    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: temp_dir.path().join("logs").to_string_lossy().to_string(),
        hitl_enabled: false,
        verification_command: None,
    };
    let wasm_path = "target/wasm32-wasip2/debug/rad_orchestrator.wasm";
    let ext_config = rad::config::ExtensionConfig {
        name: "rad-orchestrator".to_string(),
        enabled: true,
        role: "orchestrator".to_string(),
        source: wasm_path.to_string(),
        permissions: Some(perms),
        config: HashMap::new(),
    };
    config.extensions = vec![ext_config];

    let dag = Arc::new(Mutex::new(Dag::new()));
    let initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();

        // Setup a dummy snapshot directory for the node to satisfy rollback
        let snapshot_dir = snapshots.join(&n0);
        fs::create_dir_all(snapshot_dir).unwrap();

        n0
    };

    let orchestrator = Arc::new(Orchestrator::new(
        config,
        "test_session".to_string(),
        dag.clone(),
        None,
    ));

    // Trigger run_task (it will run asynchronously in background thread)
    let _ = orchestrator.run_task("hello".to_string());

    // Trigger rollback to cancel and abort the background thread
    let rollback_res = orchestrator.rollback(&initial_node);
    assert!(rollback_res.is_ok(), "Rollback should succeed");

    // Ensure task is no longer running after rollback completes
    assert!(
        !orchestrator.is_running(),
        "Task should be stopped and joined after rollback"
    );
}
