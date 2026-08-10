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
