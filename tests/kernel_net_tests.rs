//! `net-open` and the fallible `byte-stream` behind it, driven from a real
//! module (AWU 966).
//!
//! The point of nearly every test here is one distinction: an empty read means
//! the response is over, and *nothing else* may be reported that way. A
//! transport failure reported as empty is a truncated answer presented as a
//! complete one, and a slow first byte reported as empty is an answer thrown
//! away before it arrives. The extension host avoids both by blocking forever
//! in `recv()`, which the kernel cannot do — so it has to distinguish them
//! explicitly, and these tests are what hold that in place.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

fn net_wasm() -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/net_module.wasm"));
        if p.exists() {
            return p;
        }
    }
    panic!("net_module.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2")
}

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load("net", &net_wasm(), &shared.engine, Arc::downgrade(&shared))
        .expect("net module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("net".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

fn fetch(k: &KernelShared, payload: serde_json::Value) -> Result<serde_json::Value, String> {
    k.call("test", "net", "net.fetch", &payload.to_string())
        .map(|reply| serde_json::from_str(&reply).unwrap())
}

/// How the mock peer behaves once it has a connection.
enum Behaviour {
    /// Status line, then the body, then close.
    Respond { status: &'static str, body: String },
    /// 200 and the first half, then a pause longer than the kernel's 100ms read
    /// poll, then the rest. The pause is the whole point: it forces a `PENDING`
    /// in the middle of a response that does complete.
    Stall { first: String, rest: String },
}

/// A one-shot HTTP peer on an ephemeral port. Returns the port so the caller
/// never has to guess one — a fixed port makes tests collide when the suite
/// runs in parallel.
fn serve(behaviour: Behaviour) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind an ephemeral port");
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        // Read the request far enough to let the client finish sending; the
        // content is irrelevant to what is under test.
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf);

        match behaviour {
            Behaviour::Respond { status, body } => {
                let _ = write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            }
            Behaviour::Stall { first, rest } => {
                let len = first.len() + rest.len();
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{first}"
                );
                let _ = stream.flush();
                std::thread::sleep(Duration::from_millis(350));
                let _ = stream.write_all(rest.as_bytes());
            }
        }
        let _ = stream.flush();
        let _ = stream.shutdown(std::net::Shutdown::Write);
        // Leave the socket open briefly so the client reads everything before
        // the close races the last write.
        std::thread::sleep(Duration::from_millis(50));
        drop::<TcpStream>(stream);
    });
    port
}

fn ok_body(body: &str) -> u16 {
    serve(Behaviour::Respond {
        status: "200 OK",
        body: body.to_string(),
    })
}

#[test]
fn a_module_opens_a_request_and_reads_the_body() {
    let k = kernel();
    let port = ok_body("hello from a server");
    let res = fetch(
        &k,
        serde_json::json!({ "url": format!("http://127.0.0.1:{port}/") }),
    )
    .expect("net.fetch should succeed");
    assert_eq!(res["body"], "hello from a server");
}

/// The failure the whole `Fallible` variant exists for. A non-2xx response has
/// no body the guest should use, and if the kernel let it arrive as an empty
/// read the module would report a successful, empty answer.
///
/// Verified to fail with the error path removed: replacing the `Err` arm in
/// `KernelStream::Fallible` with `Ok(Vec::new())` makes this test report
/// `body: ""` and a successful call.
#[test]
fn a_non_success_status_reaches_the_module_as_an_error() {
    let k = kernel();
    let port = serve(Behaviour::Respond {
        status: "404 Not Found",
        body: "nope".to_string(),
    });
    let err = fetch(
        &k,
        serde_json::json!({ "url": format!("http://127.0.0.1:{port}/") }),
    )
    .expect_err("a 404 must not look like an empty body");
    assert!(err.contains("404"), "the status should survive: {err}");
}

#[test]
fn a_refused_connection_reaches_the_module_as_an_error() {
    let k = kernel();
    // Bound and immediately dropped, so the port is almost certainly free and
    // nothing is listening on it.
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let err = fetch(
        &k,
        serde_json::json!({ "url": format!("http://127.0.0.1:{port}/") }),
    )
    .expect_err("a refused connection must not look like an empty body");
    assert!(err.contains("failed"), "{err}");
}

/// A pause mid-response must not end the read. `read` waits 100ms and the peer
/// here waits 350ms, so the guest sees at least one `PENDING` and must keep
/// going — the case that separates "nothing yet" from "nothing more".
///
/// Verified to fail with the distinction removed: mapping the `Timeout` arm to
/// `Ok(Vec::new())` — which is what the process-side `Reader` legitimately does
/// — truncates the body to "first half " and the assertion below catches it.
#[test]
fn a_pause_mid_response_is_not_the_end_of_the_response() {
    let k = kernel();
    let port = serve(Behaviour::Stall {
        first: "first half ".to_string(),
        rest: "second half".to_string(),
    });
    let res = fetch(
        &k,
        serde_json::json!({ "url": format!("http://127.0.0.1:{port}/") }),
    )
    .expect("net.fetch should succeed");
    assert_eq!(res["body"], "first half second half");
    assert!(
        res["pending"].as_u64().unwrap() > 0,
        "the pause should have produced at least one PENDING, or this test \
         proves nothing: {res}"
    );
}

/// `read(max)` is a request for at most `max` bytes, and the host decides chunk
/// sizes — so a chunk can arrive larger than `max`. The remainder has to be
/// held, not dropped.
///
/// Verified to fail without the leftover buffer: with `data.truncate(max)` and
/// no `Incoming`, a 4,000-byte body read 64 bytes at a time comes back as 64
/// bytes, because everything past the first `max` of each chunk is discarded.
#[test]
fn a_read_smaller_than_the_chunk_loses_nothing() {
    let k = kernel();
    let body: String = std::iter::repeat_n("0123456789abcdef", 250).collect();
    assert_eq!(body.len(), 4000);
    let port = ok_body(&body);

    let res = fetch(
        &k,
        serde_json::json!({ "url": format!("http://127.0.0.1:{port}/"), "max": 64 }),
    )
    .expect("net.fetch should succeed");
    assert_eq!(
        res["body"].as_str().unwrap().len(),
        4000,
        "bytes past `max` were discarded rather than held"
    );
    assert_eq!(res["body"], body);
    assert!(
        res["reads"].as_u64().unwrap() >= 4000 / 64,
        "a 64-byte read cannot have drained 4000 bytes in fewer than 62 calls: {res}"
    );
}

#[test]
fn an_empty_url_is_rejected() {
    let k = kernel();
    let err =
        fetch(&k, serde_json::json!({ "url": "" })).expect_err("an empty URL has nothing to open");
    assert!(err.contains("requires a URL"), "{err}");
}

/// Headers reach the peer. The dialect table in AWU 967 exists to build exactly
/// one of these — an `Authorization` line — so a header that silently went
/// nowhere would strand every authenticated request.
#[test]
fn headers_and_body_reach_the_peer() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(String::new()));
    let seen_writer = Arc::clone(&seen);
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        *seen_writer.lock() = String::from_utf8_lossy(&buf[..n]).into_owned();
        let _ = write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok"
        );
        let _ = stream.flush();
        std::thread::sleep(Duration::from_millis(50));
    });

    let k = kernel();
    let res = fetch(
        &k,
        serde_json::json!({
            "url": format!("http://127.0.0.1:{port}/"),
            "headers": [["X-Rad-Test", "present"]],
            "body": r#"{"probe":true}"#,
        }),
    )
    .expect("net.fetch should succeed");
    assert_eq!(res["body"], "ok");

    let request = seen.lock().clone();
    assert!(
        request.to_lowercase().contains("x-rad-test: present"),
        "the header did not reach the peer: {request}"
    );
    assert!(
        request.contains(r#"{"probe":true}"#),
        "the body did not reach the peer: {request}"
    );
}
