//! `llm-transport-openai` (ARCHITECTURE-NEXT.md §4.1): the dialect table plus
//! `/v1/chat/completions`, as a kernel module.
//!
//! Ported from `ext/llm-connector`. Two things about the shape are worth
//! stating, because neither is obvious from the code alone.
//!
//! **It is pulled, not pushed.** §3.6.4's design has the transport `post`
//! chunks to `agent-loop`. That module does not exist until stage 8, and
//! nothing drives the kernel's post queue in production yet, so the host calls
//! `llm.generate` once and then `llm.next` repeatedly — the same loop
//! `src/wasm/rpc_meta_llm_connector.rs` already runs against the extension's
//! `event-stream.read()`. What changes in stage 6 is the call mechanism, not
//! the architecture.
//!
//! **It reads no environment.** `base_url` arrives resolved: the host applies
//! `RAD_TEST_PORT`, normalisation, and the default endpoint before calling, so
//! the request is a function of its arguments and nothing else. The dialect —
//! path, `{model}` substitution, headers — stays here, because that table is
//! §4.2's entire reason for this module to exist.
#![deny(clippy::pedantic)]

mod dialect;
mod session;
mod sse;
mod wire;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct GenerateReq {
    pub model: String,
    /// Already normalised, and already redirected if a test port is set. Empty
    /// is a caller bug rather than "use the default" — the host owns that
    /// decision and reports it with the message a user can act on.
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub dialect: Option<String>,
    pub messages: Vec<wire::Message>,
    #[serde(default)]
    pub tools: Vec<wire::Tool>,
}

#[derive(serde::Serialize)]
pub struct GenerateRes {
    /// Where the request actually went. Returned so a caller can log or assert
    /// on it without rebuilding the dialect's URL rules for itself.
    pub url: String,
}

#[derive(serde::Deserialize)]
pub struct NextReq {}

#[derive(serde::Serialize)]
pub struct NextRes {
    /// Everything parsed since the last call, oldest first.
    ///
    /// A batch rather than one event: the parser already queues whatever a
    /// frame produced, so returning the queue costs nothing and saves the host
    /// a call — and a store lock — per token.
    pub events: Vec<sse::LlmEvent>,
    /// No more events will come. The caller stops asking.
    pub done: bool,
}

fn generate(req: GenerateReq) -> Result<GenerateRes, Error> {
    let GenerateReq {
        model,
        base_url,
        api_key,
        dialect: dialect_name,
        messages,
        tools,
    } = req;

    if base_url.trim().is_empty() {
        return Err(Error::invalid("llm.generate needs a resolved base_url"));
    }

    let dialect = dialect::resolve(dialect_name.as_deref());
    let request = wire::ChatCompletionsRequest {
        // Azure puts the deployment name in the URL path, so the model string
        // is still needed after it has moved into the body.
        model: model.clone(),
        messages,
        stream: true,
        stream_options: Some(wire::StreamOptions {
            include_usage: true,
        }),
        tools: if tools.is_empty() { None } else { Some(tools) },
    };

    let body = serde_json::to_string(&request)
        .map_err(|e| Error::invalid(format!("JSON serialize error: {e}")))?;
    let url = dialect.url(&base_url, &model);
    let headers = dialect.headers(api_key.as_deref());

    let stream = crate::syscall::net_open(&url, &headers, body.as_bytes())
        .map_err(|e| Error::invalid(format!("net-open to {url} failed: {}", e.message)))?;
    session::start(stream, dialect);

    Ok(GenerateRes { url })
}

fn next(_req: NextReq) -> Result<NextRes, Error> {
    session::with(|session| {
        let events = session.pump()?;
        Ok(NextRes {
            done: session.finished() && events.is_empty(),
            events,
        })
    })
    .map_err(Error::invalid)
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "llm-openai",
    version: "0.1.0",
    methods: {
        "llm.generate" => generate,
        "llm.next" => next,
    }
}
