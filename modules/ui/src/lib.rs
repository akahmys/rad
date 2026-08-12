//! Terminal output, as a kernel module (ARCHITECTURE-NEXT.md §9.3 stage 9).
//!
//! `ui-repl` in §5's tree, minus the input half. Reading a line blocks, and
//! suspending only the caller is what §3.6.1's async wasmtime is for — which
//! stage 8 deferred, so the REPL loop stays in the host and only what it
//! *prints* moves here.
//!
//! The state machine moves as one piece rather than the printing alone: the
//! decision to defer a log or emit it depends on whether a response is
//! streaming, and splitting that across host and module would put two halves of
//! one decision in two places.
#![deny(clippy::pedantic)]

mod screen;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct TokenReq {
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct LogReq {
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct StateReq {
    /// "idle", "thinking" or "streaming".
    pub state: String,
}

#[derive(serde::Deserialize)]
pub struct StatusReq {}

#[derive(serde::Serialize)]
pub struct OkRes {
    pub ok: bool,
}

#[derive(serde::Serialize)]
pub struct StatusRes {
    pub state: String,
    pub deferred: usize,
}

fn token(req: TokenReq) -> OkRes {
    let TokenReq { text } = req;
    screen::write_token(&text);
    OkRes { ok: true }
}

fn log(req: LogReq) -> OkRes {
    screen::write_log(req.text);
    OkRes { ok: true }
}

fn set_state(req: StateReq) -> Result<OkRes, Error> {
    let StateReq { state } = req;
    let parsed = screen::State::parse(&state)
        .ok_or_else(|| Error::invalid(format!("unknown terminal state '{state}'")))?;
    screen::set_state(parsed);
    Ok(OkRes { ok: true })
}

/// What the terminal is doing, and how much is waiting. Printing is not
/// observable across dispatch; this is.
fn status(_req: StatusReq) -> StatusRes {
    StatusRes {
        state: screen::state().name().to_string(),
        deferred: screen::deferred_count(),
    }
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "ui",
    version: "0.1.0",
    methods: {
        "ui.token"  => rad_sdk::infallible(token),
        "ui.log"    => rad_sdk::infallible(log),
        "ui.state"  => set_state,
        "ui.status" => rad_sdk::infallible(status),
    }
}
