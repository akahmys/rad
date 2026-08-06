//! Dispatch between real wasm modules: a successful hop, and the cycle that
//! must surface as an error rather than a deadlock on the caller's own lock.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn wasm(name: &str) -> Option<PathBuf> {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/{name}.wasm"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Loads `echo` and `relay` into one kernel.
fn kernel() -> Option<Arc<KernelShared>> {
    let (echo, relay) = (wasm("echo_module")?, wasm("relay_module")?);
    let shared = KernelShared::new();
    for (name, path) in [("echo", echo), ("relay", relay)] {
        let rt = ModuleRuntime::load(name, &path, Arc::downgrade(&shared))
            .unwrap_or_else(|e| panic!("{name} should load: {e}"));
        shared
            .registry
            .lock()
            .register(rt.manifest.clone())
            .unwrap();
        shared
            .modules
            .lock()
            .insert(name.to_string(), Arc::new(Mutex::new(rt)));
    }
    Some(shared)
}

fn hop(target: &str, method: &str, payload: &str) -> String {
    format!(
        r#"{{"target":"{target}","method":"{method}","payload":{}}}"#,
        serde_json::to_string(payload).unwrap()
    )
}

#[test]
fn a_module_can_call_another_module() {
    let Some(k) = kernel() else { return };
    let reply = k
        .call(
            "test",
            "relay",
            "relay.hop",
            &hop("echo", "echo.say", r#"{"text":"through"}"#),
        )
        .expect("relay -> echo should succeed");
    // relay wraps whatever echo returned.
    assert!(reply.contains("through"), "{reply}");
}

#[test]
fn a_cycle_is_an_error_naming_the_chain() {
    // §3.6.3. Without the check this deadlocks: relay's store lock is held by
    // the in-flight call, so re-entering it would wait on itself forever.
    let Some(k) = kernel() else { return };
    let err = k
        .call(
            "test",
            "relay",
            "relay.hop",
            &hop("relay", "relay.hop", "{}"),
        )
        .unwrap_err();
    assert!(err.contains("dispatch cycle"), "{err}");
    assert!(err.contains("relay"), "{err}");
}

#[test]
fn the_same_shape_via_post_completes() {
    // The DoD contrast: `post` queues rather than re-entering, so what
    // deadlocks as a `call` is ordinary as a `post`.
    let Some(k) = kernel() else { return };
    let reply = k
        .call(
            "test",
            "relay",
            "relay.hop_post",
            &hop("relay", "relay.hop_post", "{}"),
        )
        .expect("post must not re-enter");
    assert!(reply.contains("posted"), "{reply}");

    let delivered = k.drain_posts();
    assert_eq!(delivered.len(), 1, "one message should have been queued");
    assert_eq!(delivered[0].0.target, "relay");
}

#[test]
fn routing_by_method_finds_the_provider_without_naming_it() {
    // A caller asks for a capability, not a module.
    let Some(k) = kernel() else { return };
    let reply = k
        .call("test", "echo.say", "echo.say", r#"{"text":"by method"}"#)
        .unwrap();
    assert!(reply.contains("by method"), "{reply}");
}

#[test]
fn an_unroutable_method_reports_rather_than_panicking() {
    let Some(k) = kernel() else { return };
    let err = k.call("test", "nobody", "nobody.method", "{}").unwrap_err();
    assert!(err.contains("no module provides"), "{err}");
}

#[test]
fn the_call_stack_unwinds_so_a_second_call_still_works() {
    // A rejected cycle must not leave the chain on the stack.
    let Some(k) = kernel() else { return };
    let _ = k.call(
        "test",
        "relay",
        "relay.hop",
        &hop("relay", "relay.hop", "{}"),
    );
    let reply = k
        .call(
            "test",
            "relay",
            "relay.hop",
            &hop("echo", "echo.say", r#"{"text":"after"}"#),
        )
        .expect("the stack should have unwound");
    assert!(reply.contains("after"), "{reply}");
}
