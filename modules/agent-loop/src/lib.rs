//! The reasoning and tool loop, as a kernel module (ARCHITECTURE-NEXT.md §4).
//!
//! Ported from `ext/rad-orchestrator` across AWU 979-984. This is the first
//! unit: the event intake, and nothing else. The extension still runs the turn;
//! what changes here is that the transport's events also reach a module, over
//! `post`, so the path is real and tested before any decision-making moves onto
//! it.
//!
//! **Events arrive by `post`, never by `call`.** §3.6.2 gives the structural
//! reason, and `tests/kernel_lock_order_tests.rs` gives the measured one: the
//! kernel holds a module's lock across a nested call, so a second thread
//! calling in while the event-loop thread calls out deadlocks beneath the cycle
//! check. `post` only touches the queue, and the event-loop thread drains it
//! (AWU 978).
#![deny(clippy::pedantic)]

mod intake;

use rad_sdk::Error;

#[derive(serde::Deserialize)]
pub struct EventReq {
    /// The transport's event, as JSON text — the same bytes
    /// `RasCoreEvent::LlmConnectorEvent` carries to the extension.
    pub event: String,
}

#[derive(serde::Serialize)]
pub struct EventRes {
    pub absorbed: bool,
}

#[derive(serde::Deserialize)]
pub struct TurnReq {}

/// A malformed event is reported, not swallowed.
///
/// It arrives by `post`, so nothing reads this error — `post` cannot report
/// delivery by design (§3.6.2), and the host logs what the drain returns. That
/// log line is the only evidence a turn went wrong at the wire level, which is
/// why the message names the offending bytes.
fn event(req: EventReq) -> Result<EventRes, Error> {
    let EventReq { event } = req;
    intake::absorb(&event).map_err(Error::invalid)?;
    Ok(EventRes { absorbed: true })
}

/// The turn accumulated so far. Read by the tests now, and by the loop itself
/// once AWU 981 moves the state machine across.
fn turn(_req: TurnReq) -> serde_json::Value {
    intake::snapshot()
}

/// Starts a fresh turn, discarding whatever the last one left.
fn turn_start(_req: TurnReq) -> serde_json::Value {
    intake::reset();
    intake::snapshot()
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "agent-loop",
    version: "0.1.0",
    methods: {
        "agent.event" => event,
        "agent.turn" => rad_sdk::infallible(turn),
        "agent.turn.start" => rad_sdk::infallible(turn_start),
    }
}
