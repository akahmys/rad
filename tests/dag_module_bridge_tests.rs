//! The host driving `modules/dag` (AWU 986).
//!
//! `dag_module_tests.rs` drives the module directly. This is about the seam:
//! with a module loaded it owns the graph, and the `Arc` the host still holds —
//! read by the auto-save in `src/main.rs`, by `command/tree.rs` and by
//! `command/compact.rs`, none of which know a module exists — becomes a cache
//! refreshed from the module's replies.
//!
//! **The failure this exists to catch is divergence**: two copies of the
//! conversation that stop agreeing, which no single-sided test can see.
use parking_lot::Mutex;
use rad::config::{Config, CoreConfig, ModuleConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use rad::subsystems::DagSubsystem;
use std::sync::Arc;

fn config_for(dir: &std::path::Path, with_module: bool) -> Config {
    let workspace = dir.join("workspace");
    let snapshots = dir.join("snapshots");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&snapshots).unwrap();
    Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: dir.join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        modules: if with_module {
            vec![ModuleConfig {
                name: "dag".to_string(),
                source: "target/wasm32-wasip2/debug/dag_module.wasm".to_string(),
                enabled: true,
                config: serde_json::Value::Null,
            }]
        } else {
            vec![]
        },
        ..Default::default()
    }
}

fn orchestrator(dir: &std::path::Path, with_module: bool) -> Arc<Orchestrator> {
    Arc::new(Orchestrator::new(
        config_for(dir, with_module),
        "bridge".to_string(),
        Arc::new(Mutex::new(Dag::new())),
        None,
    ))
}

/// The same subsystem the extension host builds, wired to this orchestrator.
fn subsystem(orch: &Arc<Orchestrator>) -> rad::dag::DagSubsystemImpl {
    rad::dag::DagSubsystemImpl {
        dag: orch.dag.clone(),
        kernel: orch.kernel.lock().clone(),
    }
}

#[test]
fn a_mutation_through_the_host_reaches_the_module() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);

    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "written through the host").unwrap();

    // Asked of the module, not of the host's copy.
    let from_module = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("test", "dag", "dag.get", "{}"))
        .expect("the kernel is loaded")
        .expect("dag.get must answer");
    let from_module: serde_json::Value = serde_json::from_str(&from_module).unwrap();
    assert_eq!(
        from_module["nodes"][&id]["text"],
        "written through the host"
    );
}

/// The readers that still hold the `Arc` do not know a module exists, so the
/// copy has to be right whenever they look. This is the one that fails if the
/// refresh after a mutation is ever dropped.
#[test]
fn the_hosts_copy_still_matches_after_a_mutation() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);

    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "both sides").unwrap();
    let child = dag.create_node(&id, "assistant").unwrap();

    let host = orch.dag.lock();
    assert_eq!(
        host.nodes.get(&id).map(|n| n.text.as_str()),
        Some("both sides"),
        "the host's copy went stale: {:?}",
        host.nodes
    );
    assert_eq!(host.current_node_id.as_deref(), Some(child.as_str()));
    assert_eq!(host.next_node_index, 2);
}

/// With no module the subsystem behaves exactly as it always did. Without this,
/// the tests above would be equally consistent with the fallback being broken —
/// and the fallback is what every host without a `dag` module runs.
#[test]
fn without_the_module_the_host_copy_is_still_authoritative() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), false);
    let dag = subsystem(&orch);

    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "local only").unwrap();

    assert_eq!(
        orch.dag.lock().nodes.get(&id).map(|n| n.text.as_str()),
        Some("local only")
    );
}

/// `get_dag` is what the extension's `GetDag` returns and what
/// `agent-loop` reads through `kernel.dag`. Both must see the module's graph,
/// not a copy that happens to agree.
#[test]
fn both_read_paths_see_the_modules_graph() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);
    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "one source").unwrap();

    // The RPC path.
    let via_rpc = dag.get_dag().unwrap();
    assert_eq!(via_rpc["nodes"][&id]["text"], "one source");

    // The kernel path, which `agent-loop` uses.
    let via_kernel = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("agent-loop", "kernel", "kernel.dag", "{}"))
        .expect("the kernel is loaded")
        .expect("kernel.dag must answer");
    let via_kernel: serde_json::Value = serde_json::from_str(&via_kernel).unwrap();
    assert_eq!(via_kernel["nodes"][&id]["text"], "one source");
}

/// The module persists on every mutation, so what the host wrote through it is
/// on disk in the file `src/session.rs` reads — the property that lets the
/// host's own auto-save keep working unchanged through the migration.
#[test]
fn what_the_host_wrote_is_on_disk_where_session_rs_looks_for_it() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);
    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "durable").unwrap();

    let workspace = dir.path().join("workspace");
    let loaded = rad::session::load_session(&workspace.to_string_lossy(), "bridge")
        .expect("the module should have written a session file the host can read");
    assert_eq!(
        loaded.nodes.get(&id).map(|n| n.text.as_str()),
        Some("durable")
    );
}

/// Rolling back moves the conversation pointer. With the module owning the
/// graph, moving it in the host's cache alone leaves the module still pointing
/// at the old tip — and the next `create_node` parents off *that*, so the
/// rollback is silently undone one turn later.
#[test]
fn a_rollback_moves_the_pointer_in_the_module_too() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);

    let first = dag.create_node("", "user").unwrap();
    dag.set_node_text(&first, "first").unwrap();
    std::fs::create_dir_all(dir.path().join("snapshots").join(&first)).unwrap();
    let second = dag.create_node(&first, "assistant").unwrap();
    dag.set_node_text(&second, "second").unwrap();

    orch.rollback(&first).expect("rollback should succeed");

    let from_module = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("test", "dag", "dag.get", "{}"))
        .expect("the kernel is loaded")
        .expect("dag.get must answer");
    let from_module: serde_json::Value = serde_json::from_str(&from_module).unwrap();
    assert_eq!(
        from_module["current_node_id"],
        first.as_str(),
        "the module still points at the old tip, so the next turn will undo the rollback"
    );
}

/// Starting a new session clears the conversation. Clearing only the host's
/// cache leaves the module holding the old one, which the next mutation copies
/// straight back over the cache — and the module also keeps writing to the
/// session file it was opened with.
#[test]
fn resetting_the_session_clears_the_module_too() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);

    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "from the old session").unwrap();

    orch.reset_session().expect("reset should succeed");

    let from_module = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("test", "dag", "dag.get", "{}"))
        .expect("the kernel is loaded")
        .expect("dag.get must answer");
    let from_module: serde_json::Value = serde_json::from_str(&from_module).unwrap();
    assert_eq!(
        from_module["nodes"].as_object().map(serde_json::Map::len),
        Some(0),
        "the module kept the old conversation: {from_module}"
    );
}

/// Which *file* the module writes to after a reset. Clearing its memory is only
/// half of it: still pointed at the old session, the next node would overwrite
/// the conversation that was just archived with an empty one.
#[test]
fn after_a_reset_the_module_writes_to_the_new_session_not_the_old_one() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);
    let workspace = dir.path().join("workspace");
    let workspace = workspace.to_string_lossy().to_string();

    let id = dag.create_node("", "user").unwrap();
    dag.set_node_text(&id, "archived").unwrap();

    let new_id = orch.reset_session().expect("reset should succeed");
    let fresh = dag.create_node("", "user").unwrap();
    dag.set_node_text(&fresh, "new session").unwrap();

    // The session that ended still holds what it held.
    let old = rad::session::load_session(&workspace, "bridge")
        .expect("the archived session should still be readable");
    assert_eq!(
        old.nodes.get(&id).map(|n| n.text.as_str()),
        Some("archived"),
        "the module overwrote the archived session"
    );

    // And the new one holds the new conversation.
    let current = rad::session::load_session(&workspace, &new_id)
        .expect("the new session should have been written");
    assert_eq!(
        current.nodes.get(&fresh).map(|n| n.text.as_str()),
        Some("new session")
    );
}

/// `/compact` merges nodes. Merging in the host's copy alone would be undone by
/// the next refresh — the same shape as AWU 988's rollback, and it had no test
/// until the readers were routed through one path.
#[test]
fn compaction_merges_in_the_module() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let dag = subsystem(&orch);

    let a = dag.create_node("", "user").unwrap();
    dag.set_node_text(&a, "one").unwrap();
    let b = dag.create_node(&a, "assistant").unwrap();
    dag.set_node_text(&b, "two").unwrap();

    // Through the kernel, which is what the orchestrator's own helper does —
    // reached this way rather than by widening that method's visibility to suit
    // a test.
    let merged = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| {
            k.call(
                "host",
                "dag",
                "dag.merge_nodes",
                &serde_json::json!({ "node_ids": [&a, &b], "summary_text": "[Compacted] both" })
                    .to_string(),
            )
        })
        .expect("a module is loaded")
        .expect("merge should succeed");
    let merged: serde_json::Value = serde_json::from_str(&merged).unwrap();
    let merged_id = merged["id"].as_str().expect("an id").to_string();

    // Read back the way `/tree` and `/status` do.
    let after = orch.conversation();
    assert_eq!(
        after.nodes.len(),
        1,
        "both nodes should have merged: {after:?}"
    );
    assert_eq!(
        after.nodes.get(&merged_id).map(|n| n.text.as_str()),
        Some("[Compacted] both")
    );
}

/// `conversation()` is the one read path, and it must show the module's graph
/// rather than the host's cache. Asserted against a graph the cache has never
/// seen: written straight to the module, with no host mutation to refresh it.
#[test]
fn conversation_reads_the_module_not_the_cache() {
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);

    let id = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| {
            k.call(
                "host",
                "dag",
                "dag.create_node",
                &serde_json::json!({ "node_type": "user" }).to_string(),
            )
        })
        .expect("a module is loaded")
        .expect("create should succeed");
    let id: serde_json::Value = serde_json::from_str(&id).unwrap();
    let id = id["id"].as_str().unwrap().to_string();

    assert!(
        orch.dag.lock().nodes.is_empty(),
        "the cache should not have been refreshed by a direct module call"
    );
    assert!(
        orch.conversation().nodes.contains_key(&id),
        "conversation() returned the stale cache instead of the module's graph"
    );
}
