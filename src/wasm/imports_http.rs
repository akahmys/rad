// `open_http_stream` implementation, split out of `imports_rpc.rs` to stay
// under the 300-line file limit.
//
// Requests carry timeouts derived from `WasmState::llm_timeout_policy`: a
// connect/first-byte wait and a per-chunk heartbeat. Without these, a peer
// that accepts the TCP connection but never writes a response (e.g. a
// wedged local LLM server) hangs the extension forever with no error
// surfaced to the user. Failures are sent through the channel as `Err` (via
// `HostStream::PipeReaderFallible`) rather than encoded as byte chunks, so
// they can't be silently swallowed by the SSE parser on the guest side.
//
// The `verify_rpc_exclude` call `open_http_stream` used to make is gone
// (AWU 970): no guest has imported `open-http-stream` since `ext/llm-connector`
// was deleted in AWU 969. Proven by probe, not by inspection — see
// `imports_process.rs`.
use crate::ipc::TimeoutPolicy;
use crate::wasm::{HostStream, WasmState, permissions};
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;

pub(crate) fn open_http_stream(
    state: &mut WasmState,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
) -> Result<wasmtime::component::Resource<HostStream>, String> {
    use wasmtime_wasi::WasiView;

    let cmd = rad_models::RasRpcCommand::OpenHttpStream {
        url: url.clone(),
        headers: std::collections::HashMap::new(),
        body: body.clone(),
    };

    permissions::check_permissions(&cmd, &state.permissions, state.sandbox.workspace_dir())
        .map_err(|e| format!("Permission denied in extension '{}': {e}", state.name))?;

    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, String>>();

    let mut header_map = std::collections::HashMap::new();
    for (k, v) in headers {
        header_map.insert(k, v);
    }

    let timeout_policy = state.llm_timeout_policy.clone();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("Error creating runtime: {e}")));
                return;
            }
        };

        rt.block_on(run_stream(&url, header_map, &body, &tx, &timeout_policy));
    });

    let res = state
        .table()
        .push(HostStream::PipeReaderFallible(Mutex::new(rx)))
        .map_err(|e| e.to_string())?;
    Ok(res)
}

fn timeout_values(policy: &Arc<Mutex<TimeoutPolicy>>) -> (Option<Duration>, Option<Duration>) {
    match *policy.lock() {
        TimeoutPolicy::Dynamic {
            heartbeat_timeout_ms,
            max_silent_wait_ms,
        } => (
            Some(Duration::from_millis(max_silent_wait_ms)),
            Some(Duration::from_millis(heartbeat_timeout_ms)),
        ),
        TimeoutPolicy::Infinite => (None, None),
    }
}

async fn run_stream(
    url: &str,
    header_map: std::collections::HashMap<String, String>,
    body: &str,
    tx: &std::sync::mpsc::Sender<Result<Vec<u8>, String>>,
    timeout_policy: &Arc<Mutex<TimeoutPolicy>>,
) {
    let client = match reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(Err(format!("Failed to build HTTP client: {e}")));
            return;
        }
    };

    let mut req_headers = reqwest::header::HeaderMap::new();
    for (k, v) in header_map {
        if let (Ok(name), Ok(value)) = (
            reqwest::header::HeaderName::from_bytes(k.as_bytes()),
            reqwest::header::HeaderValue::from_str(&v),
        ) {
            req_headers.insert(name, value);
        }
    }

    let (connect_wait, heartbeat) = timeout_values(timeout_policy);

    let req_future = client
        .post(url)
        .headers(req_headers)
        .body(body.to_string())
        .send();
    let response_res = match connect_wait {
        Some(dur) => match tokio::time::timeout(dur, req_future).await {
            Ok(res) => res,
            Err(_) => {
                let _ = tx.send(Err(format!(
                    "Connection to {url} timed out after {}ms with no response",
                    dur.as_millis()
                )));
                return;
            }
        },
        None => req_future.await,
    };

    let response = match response_res {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(Err(format!("HTTP request to {url} failed: {e}")));
            return;
        }
    };

    if !response.status().is_success() {
        let _ = tx.send(Err(format!(
            "HTTP status error: {} from {url}",
            response.status()
        )));
        return;
    }

    let mut stream = response.bytes_stream();
    loop {
        let next = match heartbeat {
            Some(dur) => match tokio::time::timeout(dur, stream.next()).await {
                Ok(item) => item,
                Err(_) => {
                    let _ = tx.send(Err(format!(
                        "LLM stream stalled: no data received for {}ms",
                        dur.as_millis()
                    )));
                    return;
                }
            },
            None => stream.next().await,
        };

        match next {
            Some(Ok(bytes)) => {
                if tx.send(Ok(bytes.to_vec())).is_err() {
                    return;
                }
            }
            Some(Err(e)) => {
                let _ = tx.send(Err(format!("Stream read error: {e}")));
                return;
            }
            None => return,
        }
    }
}
