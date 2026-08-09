//! Forwards to another module, like `relay`, but can be told to hold its own
//! store for a while first.
//!
//! That delay is the whole reason this exists. The hazard AWU 968 recorded —
//! two threads taking two modules' locks in opposite orders — needs both
//! threads to be *inside* their first module before either reaches for the
//! second. Without a way to widen that window the interleaving is a race, and a
//! test that only sometimes reproduces a deadlock is worse than none: it cannot
//! be trusted when it passes.
#![deny(clippy::pedantic)]

use rad_sdk::Error;

#[derive(serde::Deserialize)]
struct HopReq {
    target: String,
    method: String,
    payload: String,
    /// Milliseconds to stay inside this module before forwarding. The kernel
    /// holds this module's lock for the whole of `handle`, so this is a
    /// direct lever on how long that lock is held.
    #[serde(default)]
    hold_ms: u64,
}

#[derive(serde::Serialize)]
struct HopRes {
    reply: String,
}

fn hop(req: HopReq) -> Result<HopRes, Error> {
    let HopReq {
        target,
        method,
        payload,
        hold_ms,
    } = req;
    if hold_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(hold_ms));
    }
    match crate::dispatch::call(&target, &method, &payload) {
        Ok(reply) => Ok(HopRes { reply }),
        Err(e) => Err(Error::io(e)),
    }
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "pong",
    version: "0.1.0",
    methods: {
        "pong.hop" => hop,
    }
}
