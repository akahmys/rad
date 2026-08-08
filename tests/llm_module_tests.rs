//! `modules/llm-openai` against a real server over real sockets (AWU 967).
//!
//! `ext/llm-connector` had one test file, `dialect/tests.rs`, and it never
//! opened a connection — nothing checked that a request was built correctly,
//! that an SSE frame became an event, or that a mid-response failure was
//! reported. Stage 5 hit the same gap in `mcp-tool-provider` and closing it is
//! what turned up three host bugs, so it gets closed here too: these drive the
//! module through the kernel, over `net-open`, against a server that speaks
//! chat-completions back.
//!
//! The response half. What the module puts *on* the wire is in
//! `llm_module_request_tests.rs`; one file covering both crossed CODING.md §2's
//! 300-line limit for test files.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

fn llm_wasm() -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!(
            "target/wasm32-wasip2/{profile}/llm_openai_module.wasm"
        ));
        if p.exists() {
            return p;
        }
    }
    panic!(
        "llm_openai_module.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2"
    )
}

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "llm-openai",
        &llm_wasm(),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .expect("llm-openai module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("llm-openai".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

fn call(k: &KernelShared, method: &str, payload: &str) -> Result<serde_json::Value, String> {
    k.call("test", "llm-openai", method, payload)
        .map(|reply| serde_json::from_str(&reply).unwrap())
}

/// Sends `frames` as one SSE response.
///
/// Each frame is written and flushed separately with a gap between them, so the
/// module sees a genuinely chunked stream rather than one buffered write.
fn serve_sse(frames: Vec<Vec<u8>>) -> u16 {
    serve(frames, "200 OK")
}

fn serve(frames: Vec<Vec<u8>>, status: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let port = listener.local_addr().unwrap().port();

    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 8192];
        let _ = stream.read(&mut buf);

        let _ = write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n"
        );
        let _ = stream.flush();
        for frame in frames {
            let _ = stream.write_all(&frame);
            let _ = stream.flush();
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = stream.shutdown(std::net::Shutdown::Write);
        std::thread::sleep(Duration::from_millis(50));
    });

    port
}

fn frame(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn generate(k: &KernelShared, port: u16) {
    let req = serde_json::json!({
        "model": "test-model",
        "base_url": format!("http://127.0.0.1:{port}"),
        "messages": [{"role": "user", "content": "hello"}],
    });
    call(k, "llm.generate", &req.to_string()).expect("llm.generate should succeed");
}

/// Drains `llm.next` until the module says it is done, returning every event.
fn drain(k: &KernelShared) -> Result<Vec<serde_json::Value>, String> {
    let mut all = Vec::new();
    for _ in 0..200 {
        let res = call(k, "llm.next", "{}")?;
        if let Some(events) = res["events"].as_array() {
            all.extend(events.iter().cloned());
        }
        if res["done"] == serde_json::Value::Bool(true) {
            return Ok(all);
        }
    }
    panic!("llm.next never reported done: {all:?}")
}

#[test]
fn a_streamed_response_arrives_as_events() {
    let port = serve_sse(vec![
        frame("data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}\n\n"),
        frame("data: {\"choices\":[{\"delta\":{\"content\":\", world\"}}]}\n\n"),
        frame("data: {\"usage\":{\"prompt_tokens\":7,\"completion_tokens\":3}}\n\n"),
        frame("data: [DONE]\n\n"),
    ]);
    let k = kernel();
    generate(&k, port);

    let events = drain(&k).expect("draining should succeed");
    assert_eq!(
        events,
        vec![
            serde_json::json!({"ContentChunk": "Hello"}),
            serde_json::json!({"ContentChunk": ", world"}),
            serde_json::json!({"CompletionComplete": {"prompt_tokens": 7, "completion_tokens": 3}}),
        ]
    );
}

/// A model answering in a non-ASCII language emits multi-byte characters, and
/// nothing aligns socket chunks to character boundaries. The extension called
/// `String::from_utf8` per chunk and failed the whole response when a character
/// straddled the split.
///
/// Verified to fail without the incremental decode: replacing `Session::decode`
/// with `String::from_utf8(chunk)` makes this report
/// "Invalid UTF-8 chunk received".
#[test]
fn a_multibyte_character_split_across_chunks_survives() {
    // "日本語" — three 3-byte characters. The split falls inside the second.
    let payload = "data: {\"choices\":[{\"delta\":{\"content\":\"日本語\"}}]}\n\n";
    let bytes = payload.as_bytes();
    let split = payload.find('本').unwrap() + 1;

    let port = serve_sse(vec![
        bytes[..split].to_vec(),
        bytes[split..].to_vec(),
        frame("data: [DONE]\n\n"),
    ]);
    let k = kernel();
    generate(&k, port);

    let events = drain(&k).expect("a split character must not fail the response");
    assert_eq!(events, vec![serde_json::json!({"ContentChunk": "日本語"})]);
}

/// A response that ends without `[DONE]` still has to terminate. The connection
/// closing is the other end-of-stream signal, and only the kernel's empty read
/// reports it.
#[test]
fn a_connection_that_closes_without_done_still_finishes() {
    let port = serve_sse(vec![frame(
        "data: {\"choices\":[{\"delta\":{\"content\":\"cut\"}}]}\n\n",
    )]);
    let k = kernel();
    generate(&k, port);

    let events = drain(&k).expect("a closed connection ends the stream");
    assert_eq!(events, vec![serde_json::json!({"ContentChunk": "cut"})]);
}

/// A server that closes straight after a complete `data:` line, with no
/// trailing newline, has still said something. The extension dropped it: its
/// parser only ever consumed up to a `\n`, so the last token of such a response
/// was lost silently.
///
/// Verified to fail without the end-of-body terminator in `Session::pump`: the
/// event never arrives and the assertion sees an empty list.
#[test]
fn a_final_line_with_no_trailing_newline_is_not_dropped() {
    let port = serve_sse(vec![frame(
        "data: {\"choices\":[{\"delta\":{\"content\":\"last\"}}]}",
    )]);
    let k = kernel();
    generate(&k, port);

    let events = drain(&k).expect("a closed connection ends the stream");
    assert_eq!(events, vec![serde_json::json!({"ContentChunk": "last"})]);
}

#[test]
fn a_failing_status_is_reported_rather_than_read_as_an_empty_answer() {
    let port = serve(vec![frame("nope")], "401 Unauthorized");
    let k = kernel();
    generate(&k, port);

    let err = drain(&k).expect_err("a 401 must not look like an empty response");
    assert!(err.contains("401"), "the status should survive: {err}");
}

/// Two turns in a row must not leak the first response into the second — the
/// session is one slot, and a stale parser would replay old tokens.
#[test]
fn a_second_generate_replaces_the_first() {
    let port_a = serve_sse(vec![
        frame("data: {\"choices\":[{\"delta\":{\"content\":\"first\"}}]}\n\n"),
        frame("data: [DONE]\n\n"),
    ]);
    let port_b = serve_sse(vec![
        frame("data: {\"choices\":[{\"delta\":{\"content\":\"second\"}}]}\n\n"),
        frame("data: [DONE]\n\n"),
    ]);
    let k = kernel();

    generate(&k, port_a);
    assert_eq!(
        drain(&k).unwrap(),
        vec![serde_json::json!({"ContentChunk": "first"})]
    );

    generate(&k, port_b);
    assert_eq!(
        drain(&k).unwrap(),
        vec![serde_json::json!({"ContentChunk": "second"})]
    );
}
