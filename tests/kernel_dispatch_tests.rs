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
        let rt = ModuleRuntime::load(name, &path, &shared.engine, Arc::downgrade(&shared))
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

#[test]
fn a_module_that_never_returns_is_interrupted() {
    // §3.6.5. `src/esc_abort.rs`'s cooperative flag cannot stop this loop — it
    // never checks anything. Epoch interruption can, which is the argument for
    // preferring it over a flag once third-party modules exist.
    let Some(k) = kernel() else { return };

    // A deadline short enough to keep the test quick. Same mechanism as the
    // production budget, just fewer ticks.
    {
        let modules = k.modules.lock();
        let relay = modules.get("relay").unwrap().clone();
        drop(modules);
        relay.lock().deadline_ticks = 4; // ~200ms, same mechanism as production
    }

    let started = std::time::Instant::now();
    let err = k.call("test", "relay", "relay.spin", "{}").unwrap_err();
    let elapsed = started.elapsed();

    assert!(
        err.contains("interrupted") && err.contains("relay.spin"),
        "the error should say what happened and to which method: {err}"
    );
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "took {elapsed:?}; the deadline did not fire"
    );
}

#[test]
fn the_kernel_still_works_after_interrupting_a_module() {
    // An interrupted module must not take the process with it, or preemption
    // would only trade one failure for another.
    let Some(k) = kernel() else { return };
    {
        let modules = k.modules.lock();
        let relay = modules.get("relay").unwrap().clone();
        drop(modules);
        relay.lock().deadline_ticks = 4; // ~200ms, same mechanism as production
    }
    let _ = k.call("test", "relay", "relay.spin", "{}");

    let reply = k
        .call("test", "echo", "echo.say", r#"{"text":"still alive"}"#)
        .expect("other modules must be unaffected");
    assert!(reply.contains("still alive"), "{reply}");
}

#[test]
fn boot_loads_modules_from_config_and_serves_kernel_config() {
    // The DoD path end to end: a config entry becomes a routable module, and
    // its config comes back through `kernel.config` — the kernel answering
    // dispatch like any other target, so a module never special-cases the host.
    let Some(path) = wasm("echo_module") else {
        return;
    };
    let (k, loaded) = rad::kernel::boot(&[rad::config::ModuleConfig {
        name: "echo".to_string(),
        source: path.to_string_lossy().to_string(),
        enabled: true,
        config: serde_json::json!({"greeting": "hi"}),
    }]);
    assert_eq!(loaded, vec!["echo"]);

    let reply = k
        .call("test", "echo", "echo.say", r#"{"text":"configured"}"#)
        .unwrap();
    assert!(reply.contains("configured"), "{reply}");

    let config = k.call("echo", "kernel", "kernel.config", "{}").unwrap();
    assert!(config.contains("greeting"), "{config}");
}

#[test]
fn a_disabled_module_is_not_loaded() {
    let Some(path) = wasm("echo_module") else {
        return;
    };
    let (_k, loaded) = rad::kernel::boot(&[rad::config::ModuleConfig {
        name: "echo".to_string(),
        source: path.to_string_lossy().to_string(),
        enabled: false,
        config: serde_json::Value::Null,
    }]);
    assert!(loaded.is_empty());
}

#[test]
fn a_broken_module_is_skipped_rather_than_aborting_startup() {
    // One bad third-party module must not stop rad from running.
    let Some(good) = wasm("echo_module") else {
        return;
    };
    let (_k, loaded) = rad::kernel::boot(&[
        rad::config::ModuleConfig {
            name: "missing".to_string(),
            source: "/nonexistent/module.wasm".to_string(),
            enabled: true,
            config: serde_json::Value::Null,
        },
        rad::config::ModuleConfig {
            name: "echo".to_string(),
            source: good.to_string_lossy().to_string(),
            enabled: true,
            config: serde_json::Value::Null,
        },
    ]);
    assert_eq!(loaded, vec!["echo"], "the good module should still load");
}
