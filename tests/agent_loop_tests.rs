//! `modules/agent-loop`'s event intake, through a real component and a real
//! kernel.
//!
//! The fold itself is unit-tested in `modules/agent-loop/src/intake/tests.rs`.
//! What those cannot reach is the part that only exists across dispatch: that
//! the method names resolve, that the payload envelope matches what
//! `src/wasm/rpc_meta_llm_module.rs` builds, and that a `post` — not a `call` —
//! is enough to drive it.
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
        "agent-loop",
        &wasm("agent_loop_module"),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .unwrap_or_else(|e| panic!("agent-loop should load: {e}"));
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("agent-loop".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

/// Built exactly as `post_to_agent` builds it — the event as a JSON *string*
/// inside an envelope, not spliced in as an object. A mismatch here is the
/// failure this test exists to catch, and it cannot be caught by a unit test on
/// either side alone.
fn envelope(event: &serde_json::Value) -> String {
    serde_json::json!({ "event": event.to_string() }).to_string()
}

fn turn(kernel: &Arc<KernelShared>) -> serde_json::Value {
    let reply = kernel
        .call("test", "agent-loop", "agent.turn", "{}")
        .expect("agent.turn must answer");
    serde_json::from_str(&reply).expect("agent.turn must return JSON")
}

#[test]
fn a_posted_event_reaches_the_module_and_accumulates() {
    let k = kernel();
    k.post(
        "llm-openai",
        "agent-loop",
        "agent.event",
        &envelope(&serde_json::json!({ "ContentChunk": "hello" })),
    );

    let delivered = k.drain_posts();
    assert_eq!(delivered.len(), 1, "the post should have been delivered");
    assert!(delivered[0].1.is_ok(), "{:?}", delivered[0].1);

    assert_eq!(turn(&k)["text"], "hello");
}

/// A whole turn, in the order and shape the transport emits it.
#[test]
fn a_stream_of_posts_reassembles_the_turn() {
    let k = kernel();
    for event in [
        serde_json::json!({ "ContentChunk": "I will " }),
        serde_json::json!({ "ContentChunk": "write it." }),
        serde_json::json!({ "ToolCallChunk": {
            "index": 0, "id": "call_1", "name": "write", "arguments-chunk": "{\"path\":"
        }}),
        serde_json::json!({ "ToolCallChunk": { "index": 0, "arguments-chunk": "\"a.txt\"}" }}),
        serde_json::json!({ "CompletionComplete": {
            "prompt-tokens": 7, "completion-tokens": 11
        }}),
        serde_json::json!({ "type": "done" }),
    ] {
        k.post("llm-openai", "agent-loop", "agent.event", &envelope(&event));
    }

    let delivered = k.drain_posts();
    assert_eq!(delivered.len(), 6);
    for (posted, outcome) in &delivered {
        assert!(outcome.is_ok(), "{} failed: {outcome:?}", posted.method);
    }

    let t = turn(&k);
    assert_eq!(t["text"], "I will write it.");
    assert_eq!(t["done"], true);
    assert_eq!(t["tool_calls"][0]["name"], "write");
    assert_eq!(t["tool_calls"][0]["arguments"], r#"{"path":"a.txt"}"#);
    assert_eq!(t["prompt_tokens"], 7);
}

/// `agent.turn.start` is posted ahead of a generation, so a second turn must
/// not inherit the first's text. FIFO delivery is what makes ordering a
/// property rather than a hope, so the reset and the next event go in together.
#[test]
fn starting_a_turn_clears_the_previous_one() {
    let k = kernel();
    k.post(
        "llm-openai",
        "agent-loop",
        "agent.event",
        &envelope(&serde_json::json!({ "ContentChunk": "first turn" })),
    );
    k.post("llm-openai", "agent-loop", "agent.turn.start", "{}");
    k.post(
        "llm-openai",
        "agent-loop",
        "agent.event",
        &envelope(&serde_json::json!({ "ContentChunk": "second turn" })),
    );
    k.drain_posts();

    assert_eq!(turn(&k)["text"], "second turn");
}

/// A malformed event must surface in the drain's outcome rather than vanishing.
/// Nothing reads it — `post` cannot report delivery (§3.6.2) — but the host
/// logs what the drain returns, and that log line is the only evidence a turn
/// went wrong at the wire level.
#[test]
fn a_malformed_event_comes_back_as_a_failed_delivery() {
    let k = kernel();
    k.post(
        "llm-openai",
        "agent-loop",
        "agent.event",
        &serde_json::json!({ "event": "not json" }).to_string(),
    );

    let delivered = k.drain_posts();
    assert_eq!(delivered.len(), 1);
    let err = delivered[0]
        .1
        .as_ref()
        .expect_err("a malformed event must not report success");
    assert!(err.contains("not json"), "{err}");
}

/// `provider_of` is what the host checks before posting. If the method names
/// here and in `rpc_meta_llm_module.rs` ever drift, the host silently stops
/// posting and every test above still passes — this is the one that fails.
#[test]
fn the_module_declares_the_methods_the_host_posts_to() {
    let k = kernel();
    for method in ["agent.event", "agent.turn.start"] {
        assert_eq!(
            k.provider_of(method).as_deref(),
            Some("agent-loop"),
            "the host posts to '{method}' and nothing provides it"
        );
    }
}
