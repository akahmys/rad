use super::*;
use std::net::TcpListener;
use std::io::Write;
use std::thread;
use std::sync::mpsc;
use std::time::Instant;
use crate::ipc::TimeoutPolicy;

#[test]
fn test_http_streaming_success() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let response = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n5\r\nhello\r\n5\r\nworld\r\n0\r\n\r\n";
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    let (tx, rx) = mpsc::channel();
    let policy = Arc::new(Mutex::new(TimeoutPolicy::Infinite));
    let url = format!("http://127.0.0.1:{port}/stream");

    let stream_id = open_http_stream(&url, HashMap::new(), "", tx, policy).unwrap();
    assert!(!stream_id.is_empty());

    let mut tokens = Vec::new();
    while let Ok(event) = rx.recv_timeout(Duration::from_secs(2)) {
        match event {
            RasCoreEvent::TokenReceived { token } => tokens.push(token),
            _ => tokens.push(format!("{:?}", event)),
        }
    }

    let full_text = tokens.join("");
    assert!(full_text.contains("hello"), "Expected 'hello' in full_text, but got: {full_text}");
    assert!(full_text.contains("world"), "Expected 'world' in full_text, but got: {full_text}");
}

#[test]
fn test_http_streaming_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let response_headers = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n";
            let _ = stream.write_all(response_headers.as_bytes());
            let _ = stream.flush();

            thread::sleep(Duration::from_millis(400));
            let _ = stream.write_all(b"5\r\nhello\r\n0\r\n\r\n");
            let _ = stream.flush();
        }
    });

    let (tx, rx) = mpsc::channel();
    let policy = Arc::new(Mutex::new(TimeoutPolicy::Dynamic {
        heartbeat_timeout_ms: 50,
        max_silent_wait_ms: 5000,
    }));

    let url = format!("http://127.0.0.1:{port}/stream");
    let _ = open_http_stream(&url, HashMap::new(), "", tx, policy);

    let mut timeout_occurred = false;
    let mut received_events = Vec::new();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            received_events.push(format!("{event:?}"));
            if matches!(event, RasCoreEvent::StreamTimeout { ref target, .. } if target == "llm") {
                timeout_occurred = true;
                break;
            }
        }
    }
    assert!(timeout_occurred, "Timeout did not occur. Received events: {received_events:?}");
}

#[test]
fn test_http_streaming_dynamic_policy_update() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let response_headers = "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nContent-Type: text/plain\r\n\r\n";
            let _ = stream.write_all(response_headers.as_bytes());
            let _ = stream.flush();

            // First chunk sent immediately
            let _ = stream.write_all(b"5\r\nhello\r\n");
            let _ = stream.flush();

            // Second chunk delayed
            thread::sleep(Duration::from_millis(500));
            let _ = stream.write_all(b"5\r\nworld\r\n0\r\n\r\n");
            let _ = stream.flush();
        }
    });

    let (tx, rx) = mpsc::channel();
    let policy = Arc::new(Mutex::new(TimeoutPolicy::Dynamic {
        heartbeat_timeout_ms: 250,
        max_silent_wait_ms: 5000,
    }));

    let url = format!("http://127.0.0.1:{port}/stream");
    let _ = open_http_stream(&url, HashMap::new(), "", tx, policy.clone());

    // Wait for the first chunk to be received
    let mut hello_received = false;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if let Ok(RasCoreEvent::TokenReceived { token }) = rx.recv_timeout(Duration::from_millis(100))
            && token.contains("hello")
        {
            hello_received = true;
            break;
        }
    }
    assert!(hello_received, "Failed to receive first chunk 'hello'");

    // Update the policy to be more strict
    if let Ok(mut guard) = policy.lock() {
        *guard = TimeoutPolicy::Dynamic {
            heartbeat_timeout_ms: 50,
            max_silent_wait_ms: 5000,
        };
    }

    let mut timeout_occurred = false;
    let mut received_events = Vec::new();
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(1) {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
            received_events.push(format!("{event:?}"));
            if matches!(event, RasCoreEvent::StreamTimeout { ref target, .. } if target == "llm") {
                timeout_occurred = true;
                break;
            }
        }
    }
    assert!(timeout_occurred, "Timeout did not occur in dynamic update test. Received events: {received_events:?}");
}
