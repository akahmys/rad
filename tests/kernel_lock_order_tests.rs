//! The cross-thread lock-order hazard AWU 968 recorded and deferred.
//!
//! `KernelShared::deliver` holds the target module's lock across the guest
//! call, so a module that calls another holds two locks at once. The cycle
//! check cannot see this: its stack is thread-local by design (AWU 968), and
//! two threads each taking one lock and reaching for the other form no cycle on
//! either stack. They deadlock underneath it.
//!
//! Until stage 8 no module pair could do it — the only forwarding fixture was
//! `relay`, and one method may be claimed by one module (§3.6.8), so it could
//! not be loaded twice to face itself. `modules/pong` exists to make the pair
//! expressible, because the decision to route `agent-loop`'s events through
//! `post` rather than `call` rests on this being real rather than theoretical.
//!
//! **These threads stay wedged for the life of the process.** That is why this
//! file is its own test binary with nothing else in it.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

/// Long enough that a machine under load does not report a deadlock that is
/// really just slowness, short enough to keep the suite quick.
const PATIENCE: Duration = Duration::from_secs(10);

/// How long the first thread stays inside its module before reaching for the
/// second, so both are holding a lock before either blocks.
const HOLD_MS: u64 = 300;

fn wasm(name: &str) -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/{name}.wasm"));
        if p.exists() {
            return p;
        }
    }
    panic!("{name}.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2")
}

fn load(shared: &Arc<KernelShared>, name: &str, artefact: &str) {
    let rt = ModuleRuntime::load(
        name,
        &wasm(artefact),
        &shared.engine,
        Arc::downgrade(shared),
    )
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

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    load(&shared, "relay", "relay_module");
    load(&shared, "pong", "pong_module");
    shared
}

/// `<module>.hop` forwarding to `target.method`, with an inner payload that is
/// never reached in the deadlocking case.
fn hop(target: &str, method: &str, hold_ms: u64) -> String {
    serde_json::json!({
        "target": target,
        "method": method,
        "payload": "{}",
        "hold_ms": hold_ms
    })
    .to_string()
}

/// Runs one `call` on its own thread and reports whether it finished.
fn spawn_call(
    kernel: &Arc<KernelShared>,
    module: &str,
    method: &str,
    payload: String,
) -> mpsc::Receiver<()> {
    let (tx, rx) = mpsc::channel();
    let kernel = Arc::clone(kernel);
    let (module, method) = (module.to_string(), method.to_string());
    std::thread::spawn(move || {
        let _ = kernel.call("test", &module, &method, &payload);
        let _ = tx.send(());
    });
    rx
}

/// **This test passing means the deadlock is still real.**
///
/// Thread A enters `pong` and holds it; thread B enters `relay` and immediately
/// reaches for `pong`; A then reaches for `relay`. Neither can proceed.
///
/// Epoch interruption does not save this: it preempts *guest* code, and both
/// threads are blocked in a host call waiting on a `Mutex` — the same reason
/// `src/kernel/proc.rs` bounds its `wait` by hand.
///
/// When this starts *failing*, the kernel has gained lock ordering or §3.6.1's
/// scheduler, and routing events through `post` is no longer load-bearing.
#[test]
fn opposite_lock_orders_across_two_threads_deadlock() {
    let k = kernel();

    // A: pong -> (hold) -> relay
    let a = spawn_call(&k, "pong", "pong.hop", hop("relay", "relay.hop", HOLD_MS));
    // Give A time to be inside `pong` before B takes `relay`.
    std::thread::sleep(Duration::from_millis(HOLD_MS / 3));
    // B: relay -> pong, immediately.
    let b = spawn_call(&k, "relay", "relay.hop", hop("pong", "pong.hop", 0));

    let a_done = a.recv_timeout(PATIENCE).is_ok();
    let b_done = b.recv_timeout(Duration::from_secs(1)).is_ok();

    assert!(
        !a_done && !b_done,
        "expected both directions to wedge; a_done={a_done} b_done={b_done}. \
         If both finished, the kernel no longer holds a module's lock across a \
         nested call and this hazard is closed — delete the test and the note \
         in PLANS.md that defers it."
    );
}

/// The control. Identical machinery, same modules, same holds — only the order
/// matches, so the two calls serialise instead of deadlocking.
///
/// Without this, the test above proves nothing: a harness that reported a
/// timeout unconditionally would look exactly the same.
#[test]
fn the_same_order_on_both_threads_completes() {
    let k = kernel();

    let a = spawn_call(&k, "pong", "pong.hop", hop("relay", "relay.hop", HOLD_MS));
    std::thread::sleep(Duration::from_millis(HOLD_MS / 3));
    let b = spawn_call(&k, "pong", "pong.hop", hop("relay", "relay.hop", 0));

    assert!(
        a.recv_timeout(PATIENCE).is_ok(),
        "same-order calls must not wedge; the harness cannot tell a deadlock \
         from a stall if this fails"
    );
    assert!(
        b.recv_timeout(PATIENCE).is_ok(),
        "the second same-order call must complete too"
    );
}
