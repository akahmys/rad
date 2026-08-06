//! Context-window detection for local LLM endpoints, split out of
//! `command/llm.rs` to stay under the 300-line file limit. Budgeting the
//! detected value into a character count lives in `rad_models::llm_endpoint`
//! so `rad-orchestrator` (which actually builds the `context-tools` request)
//! can share the same conversion instead of re-deriving it.

/// Best-effort detection of the active model's context window, in tokens.
/// Local LLM servers don't expose this via the OpenAI-compatible
/// `/v1/chat/completions` path `rad` uses for generation, so this probes
/// backend-native metadata endpoints on the side: llama.cpp server's
/// `/props` (`default_generation_settings.n_ctx`), then Ollama's
/// `/api/show` (`model_info["<arch>.context_length"]`, key prefix varies
/// per model architecture). Returns `None` — not a guessed default — if
/// neither responds with a parseable value, so callers don't silently
/// treat "unknown" as "unlimited".
#[must_use]
fn detect_context_length(base_url: &str, model: Option<&str>) -> Option<u32> {
    let root = rad_models::normalize_base_url(base_url);
    let model = model.map(ToString::to_string);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .ok()?;
    rt.block_on(async {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(3))
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()?;

        if let Some(n) = probe_llama_cpp_context_length(&client, &root).await {
            return Some(n);
        }
        probe_ollama_context_length(&client, &root, model.as_deref()).await
    })
}

async fn probe_llama_cpp_context_length(client: &reqwest::Client, root: &str) -> Option<u32> {
    let body: serde_json::Value = client
        .get(format!("{root}/props"))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    body.get("default_generation_settings")
        .and_then(|s| s.get("n_ctx"))
        .or_else(|| body.get("n_ctx"))
        .and_then(serde_json::Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
}

async fn probe_ollama_context_length(
    client: &reqwest::Client,
    root: &str,
    model: Option<&str>,
) -> Option<u32> {
    let model = model?;
    let body: serde_json::Value = client
        .post(format!("{root}/api/show"))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()?;
    body.get("model_info")?
        .as_object()?
        .iter()
        .find(|(k, _)| k.ends_with(".context_length"))
        .and_then(|(_, v)| v.as_u64())
        .and_then(|n| u32::try_from(n).ok())
}

/// Combined result of probing an endpoint's liveness and context window.
pub struct EndpointStatus {
    pub reachable: Result<(), String>,
    pub context_length: Option<u32>,
}

/// Checks an LLM endpoint's liveness and (if reachable) its context window
/// in one call. Used uniformly by the startup health check, `/llm add`, and
/// `/llm test` — these were previously two separate probes hitting the same
/// server at different call sites, with context-length detection skipped
/// unless the caller happened to also check liveness first. Skipping
/// context-length detection when unreachable avoids paying a second
/// connect-timeout wait for a server already known to be down.
#[must_use]
pub fn check_endpoint(base_url: &str, model: Option<&str>) -> EndpointStatus {
    let reachable = probe_endpoint(base_url);
    let context_length = if reachable.is_ok() {
        detect_context_length(base_url, model)
    } else {
        None
    };
    EndpointStatus {
        reachable,
        context_length,
    }
}

/// Probes an LLM endpoint with a real HTTP round-trip (GET `/v1/models`),
/// not just a TCP connect. A peer that accepts connections but never writes
/// a response (e.g. a wedged local model server) would pass a TCP-only
/// check yet hang every real request; this catches that case within a few
/// seconds instead.
///
/// # Errors
///
/// Returns an error if the async runtime or HTTP client can't be built, or
/// if the request fails to connect or times out.
fn probe_endpoint(url_str: &str) -> Result<(), String> {
    let target = format!("{}/v1/models", rad_models::normalize_base_url(url_str));

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {e}"))?;

    rt.block_on(async {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(3))
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

        // Any response at all (even a 404/401) proves the server is alive
        // and answering; only a connect failure or timeout is a real FAILED.
        client
            .get(&target)
            .send()
            .await
            .map(|_| ())
            .map_err(|e| format!("HTTP request failed: {e}"))
    })
}
