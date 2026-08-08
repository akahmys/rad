//! `GenerateLlmStream` served by the `llm-openai` kernel module (AWU 968).
//!
//! The sibling of `rpc_meta_llm_connector.rs`, which does the same job through
//! the extension. Both feed the same event bus with the same JSON, because the
//! consumer — `rad-orchestrator` — is still an extension until stage 8.
//!
//! Two things move here from the extension, and both are the host's business
//! rather than the transport's:
//!
//! - **Resolving the endpoint.** `RAD_TEST_PORT`, `normalize_base_url`, the
//!   `api.openai.com` default, and the "no endpoint configured" error. The
//!   module gets a URL base and reads no environment, so what it requests is a
//!   function of its arguments.
//! - **Nothing else.** The dialect — path, `{model}` substitution, headers —
//!   stays in the module. Moving it here would put the dialect table back in the
//!   host and dissolve §4.2's reason for the module to exist.

use crate::wasm::rpc::RpcContext;
use std::sync::Arc;

#[cfg(test)]
mod tests;

/// The method the transport module provides. Unprefixed on purpose: tool
/// providers namespace theirs because tools are plural and aggregated, while
/// exactly one transport serves. Two modules claiming this is a startup
/// collision (§3.6.8), which is the right answer to "two transports installed".
pub(crate) const GENERATE: &str = "llm.generate";
pub(crate) const NEXT: &str = "llm.next";

/// How long to wait between `llm.next` calls once one comes back empty.
///
/// The module already blocks up to 100ms inside `read` before reporting
/// "nothing yet", so this only paces the case where it returned early with
/// nothing. Matches the extension path's sleep.
const POLL_IDLE: std::time::Duration = std::time::Duration::from_millis(5);

/// Whether a module is loaded that can serve generation.
pub(crate) fn is_available(orch: &crate::orchestrator::Orchestrator) -> bool {
    kernel_of(orch).is_some_and(|k| k.provider_of(GENERATE).is_some())
}

fn kernel_of(orch: &crate::orchestrator::Orchestrator) -> Option<Arc<crate::kernel::KernelShared>> {
    orch.kernel.lock().clone()
}

/// Where a request actually goes, resolved from the active `/llm` profile.
///
/// # Errors
///
/// Returns the message a user can act on when nothing is configured. The
/// wording is load-bearing: `tests/llm_connector_eager_load_tests.rs` asserts
/// its *absence*, so a silent rewording would make that test pass vacuously.
pub(crate) fn resolve_base_url(
    profile: &super::rpc_meta::ActiveLlmProfile,
) -> Result<String, String> {
    // Test infrastructure: redirects every call to a local mock server. It sits
    // here rather than in the module so that the module reads no environment at
    // all — and so that one place decides where a request goes.
    if let Ok(test_port) = std::env::var("RAD_TEST_PORT") {
        return Ok(format!("http://127.0.0.1:{test_port}"));
    }
    if let Some(base_url) = profile.base_url.as_deref().filter(|b| !b.trim().is_empty()) {
        return Ok(rad_models::normalize_base_url(base_url));
    }
    if profile.api_key.is_some() {
        return Ok("https://api.openai.com".to_string());
    }
    Err("No LLM endpoint configured. Set one up with /llm add <name> <url>.".to_string())
}

/// Opens the stream on the module and starts relaying its events.
///
/// # Errors
///
/// Returns a message if no endpoint is configured, if the payload cannot be
/// built, or if the module refuses the request.
pub(crate) fn generate(
    model: &str,
    messages_json: &str,
    tools_json: &str,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    let Some(orch) = ctx.orchestrator else {
        return Err("Orchestrator unavailable".to_string());
    };
    let Some(kernel) = kernel_of(orch) else {
        return Err("Kernel unavailable".to_string());
    };
    let Some(module) = kernel.provider_of(GENERATE) else {
        return Err(format!("no module provides '{GENERATE}'"));
    };

    let profile = super::rpc_meta::resolve_active_llm_profile(orch);
    let base_url = resolve_base_url(&profile)?;

    // Spliced as parsed JSON rather than re-modelled. The extension boundary
    // needed a Rust type for every field it carried; a dispatch payload is a
    // string, so the caller's own JSON goes through untouched.
    let messages: serde_json::Value = serde_json::from_str(messages_json)
        .map_err(|e| format!("Failed to parse messages JSON: {e}"))?;
    let tools: serde_json::Value =
        serde_json::from_str(tools_json).map_err(|e| format!("Failed to parse tools JSON: {e}"))?;

    let payload = serde_json::json!({
        "model": model,
        "base_url": base_url,
        "api_key": profile.api_key,
        "dialect": profile.dialect,
        "messages": messages,
        "tools": tools,
    })
    .to_string();

    // Never logged: `payload` carries the user's API key. `kernel.call` does not
    // log payloads either, which is what makes passing the key this way — rather
    // than through `kernel.config` or the environment — the narrow path.
    crate::log_host!("[kernel] {GENERATE} via module '{module}'");
    kernel
        .call("host", &module, GENERATE, &payload)
        .map_err(|e| format!("LLM Stream Generation Error: {e}"))?;

    relay_events(kernel, module, ctx.event_tx.clone());
    Ok(serde_json::Value::Null)
}

/// Drains `llm.next` on a background thread, forwarding each event onto the
/// core event bus in the shape `rad-orchestrator` already parses.
fn relay_events(
    kernel: Arc<crate::kernel::KernelShared>,
    module: String,
    event_tx: std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>,
) {
    std::thread::spawn(move || {
        loop {
            let reply = match kernel.call("host", &module, NEXT, "{}") {
                Ok(reply) => reply,
                Err(e) => {
                    send(
                        &event_tx,
                        &serde_json::json!({"type": "error", "payload": e}),
                    );
                    return;
                }
            };
            let batch: NextBatch = match serde_json::from_str(&reply) {
                Ok(batch) => batch,
                Err(e) => {
                    send(
                        &event_tx,
                        &serde_json::json!({
                            "type": "error",
                            "payload": format!("{NEXT} returned an unreadable reply: {e}")
                        }),
                    );
                    return;
                }
            };

            let idle = batch.events.is_empty();
            for event in &batch.events {
                send(&event_tx, event);
            }
            if batch.done {
                send(&event_tx, &serde_json::json!({ "type": "done" }));
                return;
            }
            if idle {
                std::thread::sleep(POLL_IDLE);
            }
        }
    });
}

#[derive(serde::Deserialize)]
struct NextBatch {
    events: Vec<serde_json::Value>,
    done: bool,
}

fn send(event_tx: &std::sync::mpsc::Sender<crate::ipc::RasCoreEvent>, event: &serde_json::Value) {
    let _ = event_tx.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
        event: event.to_string(),
    });
}
