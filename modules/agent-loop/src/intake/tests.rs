//! The fold, driven directly. Nothing here crosses dispatch — the wiring is
//! covered by `tests/agent_loop_tests.rs`, which drives a real component.
use super::{Turn, absorb, reset, snapshot};

/// Every test shares one `thread_local` turn, and `cargo test` runs these on
/// one thread each but reuses threads across tests. Resetting first makes each
/// independent of what ran before it on the same thread.
fn fresh() {
    reset();
}

#[test]
fn content_chunks_concatenate_in_order() {
    fresh();
    absorb(r#"{"ContentChunk":"Hel"}"#).unwrap();
    absorb(r#"{"ContentChunk":"lo"}"#).unwrap();
    assert_eq!(snapshot()["text"], "Hello");
}

#[test]
fn a_done_event_marks_the_turn_done() {
    fresh();
    assert_eq!(snapshot()["done"], false);
    absorb(r#"{"type":"done"}"#).unwrap();
    assert_eq!(snapshot()["done"], true);
}

/// The extension's wording, kept because the string reaches the model.
#[test]
fn an_error_without_a_payload_still_reports_one() {
    fresh();
    absorb(r#"{"type":"error"}"#).unwrap();
    assert_eq!(snapshot()["error"], "unknown error");
}

#[test]
fn an_error_event_carries_its_payload() {
    fresh();
    absorb(r#"{"type":"error","payload":"context length exceeded"}"#).unwrap();
    assert_eq!(snapshot()["error"], "context length exceeded");
}

/// `id` and `name` arrive on the first chunk for a slot and are absent on the
/// rest. Overwriting them with a later `None` would erase the call's identity —
/// the bug this ordering exists to avoid.
#[test]
fn tool_call_chunks_reassemble_without_losing_id_or_name() {
    fresh();
    absorb(
        r#"{"ToolCallChunk":{"index":0,"id":"call_1","name":"write","arguments-chunk":"{\"pa"}}"#,
    )
    .unwrap();
    absorb(r#"{"ToolCallChunk":{"index":0,"arguments-chunk":"th\":\"a.txt\"}"}}"#).unwrap();

    let snap = snapshot();
    let call = &snap["tool_calls"][0];
    assert_eq!(call["id"], "call_1");
    assert_eq!(call["name"], "write");
    assert_eq!(call["arguments"], r#"{"path":"a.txt"}"#);
}

/// Slots are named by the transport and need not arrive in order.
#[test]
fn a_higher_index_arriving_first_does_not_drop_the_lower_slot() {
    fresh();
    absorb(r#"{"ToolCallChunk":{"index":1,"id":"b","name":"second","arguments-chunk":"{}"}}"#)
        .unwrap();
    absorb(r#"{"ToolCallChunk":{"index":0,"id":"a","name":"first","arguments-chunk":"{}"}}"#)
        .unwrap();

    let snap = snapshot();
    assert_eq!(snap["tool_calls"][0]["name"], "first");
    assert_eq!(snap["tool_calls"][1]["name"], "second");
}

#[test]
fn reasoning_chunks_accumulate_separately_from_content() {
    fresh();
    absorb(r#"{"ReasoningChunk":"thinking"}"#).unwrap();
    absorb(r#"{"ContentChunk":"answer"}"#).unwrap();
    let snap = snapshot();
    assert_eq!(snap["reasoning"], "thinking");
    assert_eq!(snap["text"], "answer");
}

#[test]
fn usage_is_recorded_from_the_completion_event() {
    fresh();
    absorb(r#"{"CompletionComplete":{"prompt-tokens":12,"completion-tokens":34}}"#).unwrap();
    let snap = snapshot();
    assert_eq!(snap["prompt_tokens"], 12);
    assert_eq!(snap["completion_tokens"], 34);
}

/// Malformed bytes are an error, not a silently dropped event: a turn that
/// quietly lost a chunk is far harder to diagnose than one that says so.
#[test]
fn a_malformed_event_is_an_error_naming_the_bytes() {
    fresh();
    let err = absorb("not json").unwrap_err();
    assert!(err.contains("not json"), "{err}");
}

#[test]
fn starting_a_turn_discards_the_previous_one() {
    fresh();
    absorb(r#"{"ContentChunk":"old"}"#).unwrap();
    absorb(r#"{"type":"done"}"#).unwrap();
    reset();
    let snap = snapshot();
    assert_eq!(snap["text"], "");
    assert_eq!(snap["done"], false);
}

/// The struct's own default, checked once so the tests above can rely on
/// `fresh()` meaning what they assume.
#[test]
fn a_new_turn_is_empty() {
    let turn = Turn::default();
    let snap = serde_json::to_value(&turn).unwrap();
    assert_eq!(snap["text"], "");
    assert_eq!(snap["tool_calls"].as_array().unwrap().len(), 0);
    assert_eq!(snap["error"], serde_json::Value::Null);
}
