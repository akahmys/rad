//! `modules/dag` through a real kernel, driven the way AWU 986 will drive it.
//!
//! The graph's behaviour is covered by `modules/dag/src/graph/tests.rs` — the
//! host's own `src/dag/tests.rs`, moved across unchanged — and persistence by
//! `store/tests.rs`. What only exists here is the dispatch surface: the method
//! names, the payload shapes, and that a graph built through them comes back
//! in the shape every existing reader already parses.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn wasm(name: &str) -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/{name}.wasm"));
        if p.exists() {
            return p;
        }
    }
    panic!("{name}.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2")
}

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "dag",
        &wasm("dag_module"),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .unwrap_or_else(|e| panic!("dag should load: {e}"));
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("dag".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

fn call(k: &Arc<KernelShared>, method: &str, payload: serde_json::Value) -> serde_json::Value {
    let reply = k
        .call("test", "dag", method, &payload.to_string())
        .unwrap_or_else(|e| panic!("{method} failed: {e}"));
    serde_json::from_str(&reply).unwrap_or_else(|e| panic!("{method} returned non-JSON: {e}"))
}

fn open(k: &Arc<KernelShared>, ws: &tempfile::TempDir, session: &str) {
    call(
        k,
        "dag.open",
        serde_json::json!({
            "workspace": ws.path().to_string_lossy(),
            "session_id": session
        }),
    );
}

#[test]
fn a_conversation_can_be_built_and_read_back_through_dispatch() {
    let ws = tempfile::tempdir().unwrap();
    let k = kernel();
    open(&k, &ws, "s");

    let root = call(
        &k,
        "dag.create_node",
        serde_json::json!({ "node_type": "user" }),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    call(
        &k,
        "dag.set_node_text",
        serde_json::json!({ "node_id": root, "text": "hello" }),
    );
    let child = call(
        &k,
        "dag.create_node",
        serde_json::json!({ "parent_id": root, "node_type": "assistant" }),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();

    let dag = call(&k, "dag.get", serde_json::json!({}));
    assert_eq!(dag["nodes"][&root]["text"], "hello");
    assert_eq!(dag["nodes"][&child]["parent_ids"][0], root.as_str());
    assert_eq!(dag["current_node_id"], child.as_str());
}

/// `dag.get` has to return exactly what `GetDag` and `kernel.dag` return, since
/// `agent-loop`'s walk already parses that shape and AWU 986 swaps the producer
/// underneath it without touching the consumer.
#[test]
fn dag_get_returns_the_shape_existing_readers_parse() {
    let ws = tempfile::tempdir().unwrap();
    let k = kernel();
    open(&k, &ws, "s");
    call(
        &k,
        "dag.create_node",
        serde_json::json!({ "node_type": "user" }),
    );

    let from_module = call(&k, "dag.get", serde_json::json!({}));
    // Round-tripped through the host's own type: if a field were missing or
    // renamed this is where it fails, rather than in a consumer much later.
    let parsed: rad::dag::Dag = serde_json::from_value(from_module.clone())
        .unwrap_or_else(|e| panic!("the host cannot read the module's graph: {e}\n{from_module}"));
    assert_eq!(parsed.nodes.len(), 1);
    assert_eq!(parsed.next_node_index, 1);
}

/// Errors cross dispatch as errors rather than as an empty success — a caller
/// that took "no such node" for "done" would corrupt the conversation quietly.
#[test]
fn an_operation_on_a_missing_node_comes_back_as_an_error() {
    let ws = tempfile::tempdir().unwrap();
    let k = kernel();
    open(&k, &ws, "s");

    let err = k
        .call(
            "test",
            "dag",
            "dag.set_node_text",
            &serde_json::json!({ "node_id": "node_404", "text": "x" }).to_string(),
        )
        .expect_err("setting text on a missing node must fail");
    assert!(err.contains("node_404"), "{err}");
}

/// The module persists on every mutation, so a second kernel — a restart, or
/// §3.6.6's reload after a trap — attaching to the same session sees the same
/// conversation. This is the property stage 9's decision (A) rests on.
#[test]
fn a_second_kernel_attaching_to_the_session_sees_the_same_conversation() {
    let ws = tempfile::tempdir().unwrap();

    let first = kernel();
    open(&first, &ws, "shared");
    let id = call(
        &first,
        "dag.create_node",
        serde_json::json!({ "node_type": "user" }),
    )["id"]
        .as_str()
        .unwrap()
        .to_string();
    call(
        &first,
        "dag.set_node_text",
        serde_json::json!({ "node_id": id, "text": "survives a reload" }),
    );

    // A wholly separate instance, with its own store and its own memory.
    let second = kernel();
    open(&second, &ws, "shared");
    let dag = call(&second, "dag.get", serde_json::json!({}));
    assert_eq!(
        dag["nodes"][&id]["text"], "survives a reload",
        "the conversation did not survive a fresh instance: {dag}"
    );
}

#[test]
fn the_module_declares_every_method_the_host_will_need() {
    let k = kernel();
    for method in [
        "dag.open",
        "dag.get",
        "dag.create_node",
        "dag.set_node_text",
        "dag.merge_nodes",
        "dag.delete_node",
    ] {
        assert_eq!(
            k.provider_of(method).as_deref(),
            Some("dag"),
            "nothing provides '{method}'"
        );
    }
}
