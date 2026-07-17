use crate::ipc::{RasCoreEvent, TimeoutPolicy};
use futures_util::StreamExt;
use parking_lot::Mutex;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;

#[cfg(test)]
mod tests;

/// Starts an HTTP streaming connection in a background thread and streams response tokens.
///
/// # Errors
///
/// Returns an error if url parsing or header construction fails.
pub fn open_http_stream<S: ::std::hash::BuildHasher>(
    url: &str,
    headers: HashMap<String, String, S>,
    body: &str,
    event_tx: Sender<RasCoreEvent>,
    timeout_policy: Arc<Mutex<TimeoutPolicy>>,
) -> Result<String, crate::error::UnifiedError> {
    let stream_id = format!("http_stream_{}", uuid_like_id());
    let url_owned = url.to_string();
    let body_owned = body.to_string();

    let mut header_map = HeaderMap::new();
    for (k, v) in headers {
        let name = HeaderName::from_bytes(k.as_bytes())
            .map_err(|e| crate::error::UnifiedError::l1(format!("Invalid header name: {e}"), "Http"))?;
        let value = HeaderValue::from_str(&v)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Invalid header value: {e}"), "Http"))?;
        header_map.insert(name, value);
    }

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = event_tx.send(RasCoreEvent::HttpErrorReceived {
                    message: format!("Error creating runtime: {e}"),
                });
                return;
            }
        };

        rt.block_on(async {
            if let Err(e) = run_http_stream_async(
                &url_owned,
                header_map,
                &body_owned,
                &event_tx,
                &timeout_policy,
            )
            .await
            {
                let _ = event_tx.send(RasCoreEvent::HttpErrorReceived {
                    message: format!("HTTP error: {e}"),
                });
            }
        });
    });

    Ok(stream_id)
}

fn uuid_like_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_micros());
    format!("{now}")
}

async fn run_http_stream_async(
    url: &str,
    headers: HeaderMap,
    body: &str,
    event_tx: &Sender<RasCoreEvent>,
    timeout_policy: &Arc<Mutex<TimeoutPolicy>>,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let (max_silent_wait, _) = get_timeout_values(timeout_policy);

    let req_future = client
        .post(url)
        .headers(headers)
        .body(body.to_string())
        .send();
    let response = connect_with_timeout(req_future, max_silent_wait, event_tx).await?;

    if !response.status().is_success() {
        return Err(format!("HTTP status error: {}", response.status()));
    }

    let stream = response.bytes_stream();
    read_stream_loop(stream, event_tx, timeout_policy).await
}

async fn connect_with_timeout(
    req_future: impl std::future::Future<Output = Result<reqwest::Response, reqwest::Error>>,
    max_silent_wait: Option<Duration>,
    event_tx: &Sender<RasCoreEvent>,
) -> Result<reqwest::Response, String> {
    if let Some(wait_dur) = max_silent_wait {
        if let Ok(res_res) = tokio::time::timeout(wait_dur, req_future).await {
            res_res.map_err(|e| format!("HTTP request failed: {e}"))
        } else {
            let duration_ms = u64::try_from(wait_dur.as_millis()).unwrap_or(u64::MAX);
            let _ = event_tx.send(RasCoreEvent::StreamTimeout {
                target: "llm".to_string(),
                duration_ms,
            });
            Err("Initial connection timed out".to_string())
        }
    } else {
        req_future
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))
    }
}

async fn read_stream_loop(
    mut stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    event_tx: &Sender<RasCoreEvent>,
    timeout_policy: &Arc<Mutex<TimeoutPolicy>>,
) -> Result<(), String> {
    loop {
        let (_, heartbeat) = get_timeout_values(timeout_policy);
        let next_chunk = read_next_chunk(&mut stream, heartbeat, event_tx).await?;

        let Some(chunk) = next_chunk else {
            break;
        };

        let text = String::from_utf8_lossy(&chunk).into_owned();
        let _ = event_tx.send(RasCoreEvent::HttpChunkReceived { chunk: text });
    }

    Ok(())
}

async fn read_next_chunk(
    stream: &mut (impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin),
    heartbeat: Option<Duration>,
    event_tx: &Sender<RasCoreEvent>,
) -> Result<Option<bytes::Bytes>, String> {
    if let Some(dur) = heartbeat {
        match tokio::time::timeout(dur, stream.next()).await {
            Ok(Some(item)) => Ok(Some(item.map_err(|e| format!("Stream read error: {e}"))?)),
            Ok(None) => Ok(None),
            Err(_) => {
                let duration_ms = u64::try_from(dur.as_millis()).unwrap_or(u64::MAX);
                let _ = event_tx.send(RasCoreEvent::StreamTimeout {
                    target: "llm".to_string(),
                    duration_ms,
                });
                Err("Stream read timed out".to_string())
            }
        }
    } else {
        match stream.next().await {
            Some(item) => Ok(Some(item.map_err(|e| format!("Stream read error: {e}"))?)),
            None => Ok(None),
        }
    }
}

fn get_timeout_values(policy: &Arc<Mutex<TimeoutPolicy>>) -> (Option<Duration>, Option<Duration>) {
    let guard = policy.lock();
    match *guard {
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

pub struct HttpManager;

impl crate::subsystems::NetworkSubsystem for HttpManager {
    fn open_http_stream(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: &str,
        event_tx: Sender<crate::ipc::RasCoreEvent>,
        llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    ) -> Result<String, crate::error::UnifiedError> {
        open_http_stream(url, headers, body, event_tx, llm_timeout_policy)
    }
}
