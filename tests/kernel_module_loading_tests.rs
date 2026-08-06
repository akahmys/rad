//! Loads the real `modules/echo` component through the kernel path.
//!
//! The registry's conflict logic is unit-tested in `src/kernel/tests.rs`; this
//! covers the part that can only be shown against an actual wasm component —
//! that a module instantiates, that `manifest()` is readable before the module
//! has been granted anything, and that `handle()` round-trips.
use rad::kernel::{KernelShared, ModuleRuntime, Registry};
use std::path::PathBuf;
use std::sync::Arc;

/// A kernel that outlives the module under test. These tests never dispatch,
/// but handing over an already-dead `Weak` would mean a later test that does
/// dispatch fails for a reason that has nothing to do with what it is checking.
fn live_kernel() -> Arc<KernelShared> {
    KernelShared::new()
}

fn echo_wasm() -> Option<PathBuf> {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/echo_module.wasm"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

#[test]
fn loads_a_real_module_and_reads_its_manifest() {
    let Some(path) = echo_wasm() else {
        eprintln!("skipping: echo_module.wasm not built for wasm32-wasip2");
        return;
    };
    let kernel = live_kernel();
    let rt = ModuleRuntime::load("echo", &path, Arc::downgrade(&kernel)).expect("echo should load");
    assert_eq!(rt.manifest.name, "echo");
    assert_eq!(rt.manifest.provides, vec!["echo.say"]);
}

#[test]
fn a_loaded_module_answers_and_rejects_through_handle() {
    let Some(path) = echo_wasm() else { return };
    let kernel = live_kernel();
    let mut rt = ModuleRuntime::load("echo", &path, Arc::downgrade(&kernel)).unwrap();

    let ok = rt.handle("echo.say", r#"{"text":"hi"}"#).unwrap();
    assert_eq!(ok, r#"{"text":"hi"}"#);

    // A handler error crosses the boundary as a message, not a trap.
    let err = rt.handle("echo.say", r#"{"text":""}"#).unwrap_err();
    assert_eq!(err, "Invalid: text must not be empty");

    // An unknown method is reported; the module keeps working afterwards.
    let unknown = rt.handle("echo.nope", "{}").unwrap_err();
    assert!(unknown.contains("echo.nope"), "{unknown}");
    assert!(rt.handle("echo.say", r#"{"text":"still here"}"#).is_ok());
}

#[test]
fn a_module_whose_configured_name_disagrees_with_its_manifest_is_rejected() {
    // Routing and diagnostics would otherwise disagree about what to call it.
    let Some(path) = echo_wasm() else { return };
    let kernel = live_kernel();
    let Err(err) = ModuleRuntime::load("not-echo", &path, Arc::downgrade(&kernel)) else {
        panic!("a mismatched name should be rejected");
    };
    assert!(err.contains("declares itself 'echo'"), "{err}");
}

#[test]
fn loading_the_same_module_twice_conflicts_in_the_registry() {
    // The DoD case, against a real manifest rather than a constructed one.
    let Some(path) = echo_wasm() else { return };
    let kernel = live_kernel();
    let first = ModuleRuntime::load("echo", &path, Arc::downgrade(&kernel)).unwrap();
    let mut registry = Registry::new();
    registry.register(first.manifest.clone()).unwrap();

    let mut clashing = first.manifest.clone();
    clashing.name = "echo2".to_string();
    let err = registry.register(clashing).unwrap_err().to_string();
    assert!(
        err.contains("echo.say") && err.contains("'echo'") && err.contains("'echo2'"),
        "{err}"
    );
}
