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

/// `agent.messages` across dispatch: the DAG in, an OpenAI-shaped message array
/// out, system prompt first.
///
/// The walk and the filter are unit-tested; what only exists here is the
/// envelope — that the host's `GetDag` JSON deserialises into the module's own
/// `Dag` shape. The module declares a narrower struct than `rad_models::Dag`
/// (no `next_node_index`, no `semantic_references`), so this is where a field
/// the host emits and the module rejects would show up.
#[test]
fn agent_messages_turns_a_dag_into_a_request_message_list() {
    let k = kernel();
    // Exactly what `RasRpcCommand::GetDag` returns, extra fields and all.
    let dag = serde_json::json!({
        "nodes": {
            "n0": { "id": "n0", "parent_ids": [], "node_type": "user", "text": "hello" },
            "n1": { "id": "n1", "parent_ids": ["n0"], "node_type": "assistant",
                    "text": "hi", "semantic_references": null }
        },
        "current_node_id": "n1",
        "next_node_index": 2
    });

    let reply = k
        .call(
            "test",
            "agent-loop",
            "agent.messages",
            &serde_json::json!({ "dag": dag }).to_string(),
        )
        .expect("agent.messages must answer");
    let msgs: serde_json::Value = serde_json::from_str(&reply).unwrap();
    let msgs = msgs.as_array().expect("an array of messages");

    assert_eq!(msgs.len(), 3, "system + two turns: {msgs:?}");
    assert_eq!(msgs[0]["role"], "system");
    assert!(
        msgs[0]["content"]
            .as_str()
            .unwrap_or_default()
            .contains("expert coding assistant"),
        "the system prompt is not the one the model has been tuned against"
    );
    assert_eq!(msgs[1]["role"], "user");
    assert_eq!(msgs[1]["content"], "hello");
    assert_eq!(msgs[2]["role"], "assistant");
}

/// A DAG that is not one comes back as an error rather than an empty
/// conversation — an empty message list would reach the backend and be blamed
/// on the model.
#[test]
fn a_malformed_dag_is_refused_rather_than_yielding_an_empty_conversation() {
    let k = kernel();
    let err = k
        .call(
            "test",
            "agent-loop",
            "agent.messages",
            &serde_json::json!({ "dag": "not a dag" }).to_string(),
        )
        .expect_err("a malformed dag must not answer with a message list");
    assert!(err.contains("bad dag"), "{err}");
}

/// `kernel.dag` — the (A) decision from AWU 980: host-owned runtime state
/// reaches a module as a kernel method, beside `kernel.config` and
/// `kernel.modules`.
///
/// Scaffolding with a known expiry: stage 9 makes `dag` a module and this goes
/// with it. Tested anyway, because "temporary" and "unverified" are not the
/// same word.
#[test]
fn agent_messages_fetches_the_dag_from_the_kernel_when_none_is_given() {
    let k = kernel();
    let dag = Arc::new(Mutex::new(rad::dag::Dag::new()));
    {
        let mut guard = dag.lock();
        let n0 = guard.create_node("", "user").unwrap();
        guard.set_node_text(&n0, "from the kernel").unwrap();
    }
    *k.dag.lock() = Some(Arc::clone(&dag));

    let reply = k
        .call("test", "agent-loop", "agent.messages", "{}")
        .expect("agent.messages must answer without being handed a dag");
    let msgs: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(msgs[1]["content"], "from the kernel", "{msgs}");
}

/// A kernel with no conversation says so rather than answering with an empty
/// one. An empty conversation would reach the backend and be blamed on the
/// model; every module test in `tests/` builds a kernel this way, so the case
/// is real rather than hypothetical.
#[test]
fn a_kernel_with_no_dag_refuses_rather_than_inventing_an_empty_one() {
    let k = kernel();
    let err = k
        .call("test", "agent-loop", "agent.messages", "{}")
        .expect_err("a kernel with no dag must not answer with a message list");
    assert!(err.contains("no conversation"), "{err}");
}

/// The live conversation, not a copy taken when the kernel booted. A module
/// reading a stale DAG would build every request from the first turn.
#[test]
fn kernel_dag_reflects_changes_made_after_it_was_attached() {
    let k = kernel();
    let dag = Arc::new(Mutex::new(rad::dag::Dag::new()));
    *k.dag.lock() = Some(Arc::clone(&dag));

    {
        let mut guard = dag.lock();
        let n0 = guard.create_node("", "user").unwrap();
        guard.set_node_text(&n0, "added afterwards").unwrap();
    }

    let reply = k
        .call("test", "agent-loop", "agent.messages", "{}")
        .unwrap();
    let msgs: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(msgs[1]["content"], "added afterwards", "{msgs}");
}
