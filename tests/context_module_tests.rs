//! End-to-end coverage for context compaction through the *kernel* boundary,
//! replacing `tests/context_tools_tests.rs` when the extension was deleted.
//!
//! The original existed because the guest-side unit tests never crossed a real
//! component boundary, and a serialization mismatch there once made `optimize`
//! silently window nothing — every field deserialized as its default and the
//! summary still looked plausible. The same class of failure is possible over
//! dispatch, so the coverage moves rather than disappears.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn kernel_with_context() -> Arc<KernelShared> {
    // Panics rather than skipping: this suite exists to catch a serialization
    // mismatch that already shipped once, and a vacuous pass would hide it as
    // effectively as the bug did.
    let path = ["debug", "release"]
        .iter()
        .map(|p| PathBuf::from(format!("target/wasm32-wasip2/{p}/context_module.wasm")))
        .find(|p| p.exists())
        .expect("context_module.wasm not built for wasm32-wasip2");
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "context-tools",
        &path,
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .expect("context module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("context-tools".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

fn request(count: usize, max_history: Option<u32>) -> String {
    let messages: Vec<serde_json::Value> = (0..count)
        .map(|i| {
            serde_json::json!({
                "node_id": i.to_string(),
                "role": if i % 2 == 0 { "user" } else { "assistant" },
                "content": format!("message {i}")
            })
        })
        .collect();
    serde_json::json!({ "messages": messages, "max_history": max_history }).to_string()
}

#[test]
fn optimize_actually_windows_across_the_dispatch_boundary() {
    // The regression the original test was written for: a field that fails to
    // deserialize defaults to `None`, windowing is skipped, and the response
    // still looks reasonable. Asserting the count *changed* is what catches it.
    let k = kernel_with_context();
    let reply = k
        .call(
            "test",
            "context-tools",
            "context-tools.optimize",
            &request(20, Some(5)),
        )
        .expect("optimize should succeed");
    let v: serde_json::Value = serde_json::from_str(&reply).unwrap();
    let kept = v["optimized_messages"].as_array().unwrap().len();
    assert!(kept < 20, "max_history was ignored: kept {kept} of 20");
    assert!(kept > 0, "windowed everything away");
}

#[test]
fn without_a_budget_nothing_is_dropped() {
    let k = kernel_with_context();
    let reply = k
        .call(
            "test",
            "context-tools",
            "context-tools.optimize",
            &request(20, None),
        )
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(v["optimized_messages"].as_array().unwrap().len(), 20);
}

#[test]
fn an_empty_request_is_answered_not_rejected() {
    let k = kernel_with_context();
    let reply = k
        .call(
            "test",
            "context-tools",
            "context-tools.optimize",
            &request(0, Some(5)),
        )
        .unwrap();
    let v: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(v["summary"], "Empty request.");
}

#[test]
fn a_zero_budget_is_reported_across_the_boundary() {
    // The addition made in AWU 956: an error must survive dispatch as an error,
    // not arrive as an empty history that looks like successful compaction.
    let k = kernel_with_context();
    let err = k
        .call(
            "test",
            "context-tools",
            "context-tools.optimize",
            &request(5, Some(0)),
        )
        .unwrap_err();
    assert!(err.contains("greater than zero"), "{err}");
}
