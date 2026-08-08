//! `net-open` and the HTTP body behind it.
//!
//! §3.1.2's rule puts this here rather than on `wasi:http`: Rust's `std` has no
//! HTTP client, so a module would have to import the WASI interface directly
//! and would break the day WASI 0.3 removed `wasi:io` from under it. The kernel
//! owns the syscall and absorbs that churn instead (§3.1.1).
//!
//! The request machinery is ported from `src/wasm/imports_http.rs` rather than
//! rewritten. Two things do not come with it:
//!
//! - **The permission mask.** Modules have no permission mechanism (§3.4).
//! - **`verify_rpc_exclude`.** The kernel holds no orchestrator handle, so the
//!   `security-guard` check the extension host runs before every request has no
//!   equivalent here. This is the *second* occurrence of that gap — `proc-spawn`
//!   is the first — and both belong to the `policy` module (§3.4.3, stage 7).
//!   Exposure today is the same as `proc-spawn`'s: a module's URL comes from
//!   the host's own configuration, not from the model.

use super::host::KernelState;
use super::stream::{Incoming, KernelStream, err};
use crate::ipc::TimeoutPolicy;
use crate::wasm::bindings::rad_kernel::rad::kernel::types;
use futures_util::StreamExt;
use parking_lot::Mutex;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;
use wasmtime::component::Resource;
use wasmtime_wasi::WasiView;

/// Matches the extension host. A peer that accepts the connection and then says
/// nothing is the failure this catches; the per-chunk heartbeat below catches
/// the same thing once bytes have started flowing.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

pub fn open(
    state: &mut KernelState,
    url: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
) -> Result<Resource<types::ByteStream>, types::Error> {
    let Some(shared) = state.shared.upgrade() else {
        return Err(err(503, "kernel is shutting down"));
    };
    if url.trim().is_empty() {
        return Err(err(400, "net-open requires a URL"));
    }

    // The URL, never the headers: an `Authorization` header carries the user's
    // API key, and this line would put it in the log of every request.
    crate::log_host!("[kernel] module '{}' opening {url}", state.module_name);

    let (tx, rx) = std::sync::mpsc::channel::<Result<Vec<u8>, String>>();
    let timeout_policy = Arc::clone(&shared.llm_timeout_policy);

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
        rt.block_on(run_stream(&url, headers, &body, &tx, &timeout_policy));
    });

    state
        .table()
        .push(KernelStream::Fallible(Incoming::new(rx)))
        .map_err(|e| err(500, format!("could not register the stream: {e}")))
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

/// Every failure leaves through `tx` as an `Err`, never as bytes and never as
/// silence. A guest that saw a failure as an empty read would take it for the
/// end of the response and report a truncated answer as a complete one.
async fn run_stream(
    url: &str,
    headers: Vec<(String, String)>,
    body: &[u8],
    tx: &Sender<Result<Vec<u8>, String>>,
    timeout_policy: &Arc<Mutex<TimeoutPolicy>>,
) {
    let client = match reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(Err(format!("Failed to build HTTP client: {e}")));
            return;
        }
    };

    let mut req_headers = reqwest::header::HeaderMap::new();
    for (k, v) in headers {
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
        .body(body.to_vec())
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

    read_body(response, tx, heartbeat).await;
}

/// Returning drops `tx`, and that disconnect is what tells the guest the body
/// is complete — the one path out of here that is not an error.
async fn read_body(
    response: reqwest::Response,
    tx: &Sender<Result<Vec<u8>, String>>,
    heartbeat: Option<Duration>,
) {
    let mut stream = response.bytes_stream();
    loop {
        let next = match heartbeat {
            Some(dur) => match tokio::time::timeout(dur, stream.next()).await {
                Ok(item) => item,
                Err(_) => {
                    let _ = tx.send(Err(format!(
                        "stream stalled: no data received for {}ms",
                        dur.as_millis()
                    )));
                    return;
                }
            },
            None => stream.next().await,
        };

        match next {
            Some(Ok(bytes)) => {
                // A send failure means the guest dropped the stream, so there
                // is no one left to read what the peer is still sending.
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
