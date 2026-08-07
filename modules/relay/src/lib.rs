//! Forwards whatever it is given to another module, by `call` or by `post`.
//!
//! Its only purpose is to make dispatch observable from the guest side: a
//! successful hop, and the A -> A cycle that §3.6.3 requires to surface as an
//! error before it can become a deadlock on the caller's own store lock.
#![deny(clippy::pedantic)]

use rad_sdk::Error;

#[derive(serde::Deserialize)]
struct HopReq {
    target: String,
    method: String,
    payload: String,
}

#[derive(serde::Serialize)]
struct HopRes {
    reply: String,
}

/// Forwards synchronously and returns whatever came back.
fn hop(req: HopReq) -> Result<HopRes, Error> {
    let HopReq {
        target,
        method,
        payload,
    } = req;
    match crate::dispatch::call(&target, &method, &payload) {
        Ok(reply) => Ok(HopRes { reply }),
        Err(e) => Err(Error::io(e)),
    }
}

/// Forwards asynchronously. Returns immediately — `post` cannot report
/// delivery, by design, so this cannot fail either.
fn hop_post(req: HopReq) -> HopRes {
    let HopReq {
        target,
        method,
        payload,
    } = req;
    crate::dispatch::post(&target, &method, &payload);
    HopRes {
        reply: "posted".to_string(),
    }
}

#[derive(serde::Deserialize)]
struct SpinReq {}

#[derive(serde::Serialize)]
struct SpinRes {
    never: bool,
}

/// Never returns. Exists so the kernel's preemption can be demonstrated rather
/// than asserted — a cooperative abort flag cannot stop this, which is the
/// whole argument for epoch interruption (§3.6.5).
fn spin(_req: SpinReq) -> Result<SpinRes, Error> {
    let mut n: u64 = 0;
    loop {
        n = n.wrapping_add(1);
        std::hint::black_box(n);
    }
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "relay",
    version: "0.1.0",
    methods: {
        "relay.hop"      => hop,
        "relay.hop_post" => rad_sdk::infallible(hop_post),
        "relay.spin"     => spin,
    }
}
