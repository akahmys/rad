// `open_http_stream` implementation, split out of `imports_rpc.rs` to stay
// under the 300-line file limit.
use crate::ipc::RasRpcRequest;
use crate::wasm::{HostStream, WasmState, permissions};
use futures_util::StreamExt;
use parking_lot::Mutex;

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

    let orchestrator = state.orchestrator.as_ref().and_then(|w| w.upgrade());
    if let Some(ref orch) = orchestrator {
        let req = RasRpcRequest {
            id: Some("wasm_call".to_string()),
            command: cmd.clone(),
        };
        let buf = serde_json::to_vec(&req)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;
        if let Err(e) = orch.verify_rpc_exclude(&state.name, &req, &buf) {
            return Err(format!("Security verification failed: {e}"));
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();

    // Convert headers to HashMap
    let mut header_map = std::collections::HashMap::new();
    for (k, v) in headers {
        header_map.insert(k, v);
    }

    // We can just use tokio to fetch and stream bytes
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(format!("Error creating runtime: {e}").into_bytes());
                return;
            }
        };

        let url_owned = url.clone();
        let body_owned = body.clone();

        rt.block_on(async {
            let client = reqwest::Client::new();
            let mut req_headers = reqwest::header::HeaderMap::new();
            for (k, v) in header_map {
                if let (Ok(name), Ok(value)) = (
                    reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                    reqwest::header::HeaderValue::from_str(&v),
                ) {
                    req_headers.insert(name, value);
                }
            }

            let response_res = client
                .post(&url_owned)
                .headers(req_headers)
                .body(body_owned)
                .send()
                .await;

            let response = match response_res {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(format!("HTTP request failed: {e}").into_bytes());
                    return;
                }
            };

            if !response.status().is_success() {
                let _ =
                    tx.send(format!("HTTP status error: {}", response.status()).into_bytes());
                return;
            }

            let mut stream = response.bytes_stream();
            while let Some(chunk_res) = stream.next().await {
                match chunk_res {
                    Ok(bytes) => {
                        if tx.send(bytes.to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(format!("Stream read error: {e}").into_bytes());
                        break;
                    }
                }
            }
        });
    });

    let res = state
        .table()
        .push(HostStream::PipeReader(Mutex::new(rx)))
        .map_err(|e| e.to_string())?;
    Ok(res)
}
