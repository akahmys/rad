//! The production drain, driven through the real `process_event_loop`.
//!
//! A companion file rather than an integration test because the loop is
//! `pub(crate)`: reaching it from `tests/` would mean widening its visibility
//! to suit the test, which is the wrong direction.
use crate::config::{Config, CoreConfig, ModuleConfig};
use crate::dag::Dag;
use crate::ipc::RasCoreEvent;
use crate::orchestrator::Orchestrator;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::time::Duration;

fn module(name: &str, artefact: &str) -> ModuleConfig {
    ModuleConfig {
        name: name.to_string(),
        source: format!("target/wasm32-wasip2/debug/{artefact}.wasm"),
        enabled: true,
        config: serde_json::Value::Null,
    }
}

/// An orchestrator whose kernel holds the `spawn` fixture, and nothing else.
///
/// `mcp` in testmode was the first attempt and the obvious choice. It needs
/// `RAD_TEST_PORT`, which is process-global, and setting it here broke four
/// `rpc_meta_llm_module` tests running in parallel in this same binary — the
/// pollution `tests/llm_command_tests.rs` carries a `TEST_MUTEX` to avoid. A
/// serialising mutex would have worked; needing no shared global at all is
/// better, and `spawn` touches a file while reading no environment.
fn orchestrator(temp: &tempfile::TempDir) -> Arc<Orchestrator> {
    let workspace = temp.path().join("ws");
    let snapshots = temp.path().join("sn");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&snapshots).unwrap();

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        modules: vec![module("spawn", "spawn_module")],
        ..Default::default()
    };
    Arc::new(Orchestrator::new(
        config,
        "drain_test".to_string(),
        Arc::new(Mutex::new(Dag::new())),
        None,
    ))
}

/// The payload for a `spawn.run` that creates `marker`. The file is the
/// witness: it exists only if the post was actually delivered into the module
/// and the syscall ran.
fn touch(marker: &std::path::Path) -> String {
    serde_json::json!({
        "argv": ["touch", marker.display().to_string()]
    })
    .to_string()
}

/// Nothing drove `drain_posts` outside a test until AWU 978 — `post` existed
/// and queued, and the queue was never emptied in a running rad.
#[test]
fn a_queued_post_is_delivered_by_the_event_loop() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("posted_and_delivered");
    let orch = orchestrator(&temp);

    {
        let kernel = orch
            .kernel
            .lock()
            .clone()
            .expect("kernel should have booted");
        kernel.post("test", "spawn", "spawn.run", &touch(&marker));
    }

    // Ends the loop shortly after, so the test cannot hang if the drain never
    // happens — it fails on the assertion instead, which says more.
    let (event_tx, event_rx) = channel::<RasCoreEvent>();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(600));
        let _ = event_tx.send(RasCoreEvent::TaskCompleted);
    });

    orch.process_event_loop(&event_rx, &HashMap::new())
        .expect("the loop should end cleanly");

    assert!(
        marker.exists(),
        "the queued post never reached the module: {} does not exist",
        marker.display()
    );
}

/// The loop must not end just because no event arrived — `recv_timeout`
/// replaced a blocking `recv`, and treating a timeout as disconnection would
/// end every task after one tick.
#[test]
fn a_tick_with_no_event_does_not_end_the_loop() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("delivered_after_many_ticks");
    let orch = orchestrator(&temp);

    let (event_tx, event_rx) = channel::<RasCoreEvent>();
    let kernel = orch
        .kernel
        .lock()
        .clone()
        .expect("kernel should have booted");
    let payload = touch(&marker);
    std::thread::spawn(move || {
        // Long after the first tick would have fired.
        std::thread::sleep(Duration::from_millis(400));
        kernel.post("test", "spawn", "spawn.run", &payload);
        std::thread::sleep(Duration::from_millis(400));
        let _ = event_tx.send(RasCoreEvent::TaskCompleted);
    });

    orch.process_event_loop(&event_rx, &HashMap::new())
        .expect("the loop should end cleanly");

    assert!(
        marker.exists(),
        "a post queued several ticks in was never delivered, so the loop had \
         already given up waiting"
    );
}

/// The drain after the handlers is not a latency trim, and this is why.
///
/// When events arrive faster than `TICK`, `recv_timeout` returns `Ok` on every
/// iteration and the timeout branch never runs. With only that branch draining,
/// the queue starves for as long as the stream lasts — and an LLM turn is
/// exactly such a stream. Removing the post-handler drain fails this test and
/// leaves the other two passing, which is the whole reason it exists.
#[test]
fn a_steady_event_stream_does_not_starve_the_post_queue() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("delivered_under_a_busy_stream");
    let orch = orchestrator(&temp);

    {
        let kernel = orch
            .kernel
            .lock()
            .clone()
            .expect("kernel should have booted");
        kernel.post("test", "spawn", "spawn.run", &touch(&marker));
    }

    let (event_tx, event_rx) = channel::<RasCoreEvent>();
    std::thread::spawn(move || {
        // Comfortably faster than TICK, so the timeout branch never fires.
        for _ in 0..40 {
            let sent = event_tx.send(RasCoreEvent::HttpChunkReceived {
                chunk: "x".to_string(),
            });
            if sent.is_err() {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        // Immediately, leaving no idle gap for a tick to rescue the drain.
        let _ = event_tx.send(RasCoreEvent::TaskCompleted);
    });

    orch.process_event_loop(&event_rx, &HashMap::new())
        .expect("the loop should end cleanly");

    assert!(
        marker.exists(),
        "the post was never delivered while events kept arriving, so the only \
         drain was the idle one"
    );
}
