//! What `modules/llm-openai` puts on the wire, and what it refuses (AWU 967).
//!
//! The request half of the module's integration tests; the response half is in
//! `llm_module_tests.rs`. Split because one file covering both crossed
//! CODING.md §2's 300-line limit for test files. The harness is duplicated
//! rather than shared: every one of the thirteen test files here that needs a
//! mock server writes its own, and introducing `tests/common/` for a single
//! caller would be inventing a convention rather than following one.
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

/// Answers `[DONE]` immediately and records the request it was sent.
fn recording_server() -> (u16, Arc<Mutex<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(String::new()));
    let seen_writer = Arc::clone(&seen);

    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 8192];
        let n = stream.read(&mut buf).unwrap_or(0);
        *seen_writer.lock() = String::from_utf8_lossy(&buf[..n]).into_owned();

        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: [DONE]\n\n"
        );
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Write);
        std::thread::sleep(Duration::from_millis(50));
    });

    (port, seen)
}

/// Reads to the end so the server thread finishes before assertions run on what
/// it recorded.
fn drain(k: &KernelShared) {
    for _ in 0..200 {
        let Ok(res) = call(k, "llm.next", "{}") else {
            return;
        };
        if res["done"] == serde_json::Value::Bool(true) {
            return;
        }
    }
}

/// The request body and headers the peer actually receives. Everything about
/// the dialect — path, auth scheme, `stream: true` — is only observable here.
#[test]
fn the_request_matches_what_the_dialect_describes() {
    let (port, seen) = recording_server();
    let k = kernel();

    let req = serde_json::json!({
        "model": "test-model",
        "base_url": format!("http://127.0.0.1:{port}"),
        "api_key": "sk-test-key",
        "messages": [{"role": "user", "content": "hello"}],
        "tools": [{
            "type": "function",
            "function": {
                "name": "read_file",
                "description": "reads",
                "parameters": {"type": "object"}
            }
        }],
    });
    let res = call(&k, "llm.generate", &req.to_string()).expect("llm.generate should succeed");
    drain(&k);

    assert_eq!(
        res["url"],
        format!("http://127.0.0.1:{port}/v1/chat/completions")
    );

    let request = seen.lock().clone();
    assert!(
        request.starts_with("POST /v1/chat/completions "),
        "wrong path: {request}"
    );
    assert!(
        request
            .to_lowercase()
            .contains("authorization: bearer sk-test-key"),
        "the dialect's auth header did not arrive: {request}"
    );

    let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(body).expect("a JSON body");
    assert_eq!(body["stream"], true);
    assert_eq!(body["stream_options"]["include_usage"], true);
    assert_eq!(body["model"], "test-model");
    assert_eq!(body["messages"][0]["content"], "hello");
    // `parameters` is an object, not a string containing JSON. The WIT boundary
    // forced the double encode; nothing does now.
    assert_eq!(body["tools"][0]["function"]["parameters"]["type"], "object");
}

/// No key configured means no auth header at all, rather than an empty one —
/// a local llama.cpp rejects `Authorization: Bearer ` outright.
#[test]
fn no_api_key_sends_no_auth_header() {
    let (port, seen) = recording_server();
    let k = kernel();

    let req = serde_json::json!({
        "model": "m",
        "base_url": format!("http://127.0.0.1:{port}"),
        "messages": [{"role": "user", "content": "hi"}],
    });
    call(&k, "llm.generate", &req.to_string()).expect("llm.generate should succeed");
    drain(&k);

    let request = seen.lock().to_lowercase();
    assert!(
        !request.contains("authorization:"),
        "an empty auth header was sent: {request}"
    );
}

#[test]
fn next_without_generate_is_an_error_rather_than_an_empty_answer() {
    let k = kernel();
    let err = call(&k, "llm.next", "{}").expect_err("there is nothing to read from");
    assert!(err.contains("no generation is in flight"), "{err}");
}

/// Resolving the endpoint is the host's job (`RAD_TEST_PORT`, normalisation,
/// the default). An empty value means that did not happen, which is a caller
/// bug and not a cue to invent a default here.
#[test]
fn an_empty_base_url_is_rejected() {
    let k = kernel();
    let err = call(
        &k,
        "llm.generate",
        r#"{"model":"m","base_url":"  ","messages":[]}"#,
    )
    .expect_err("an unresolved base_url must not reach the network");
    assert!(err.contains("resolved base_url"), "{err}");
}
