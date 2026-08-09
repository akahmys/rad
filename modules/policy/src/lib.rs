//! Approval and refusal policy, as a kernel module (ARCHITECTURE-NEXT.md
//! §3.4.3). Ported from `ext/security-guard`.
//!
//! **It has no enforcement power, and that is the design.** A module that
//! declines to ask is not stopped by anything here. What this defends against
//! is not a malicious module — the user installed those, and "installed" is
//! already a trust decision — but the model acting on untrusted input it read
//! (§3.4.1). `modules/mcp` asks before it runs a tool because it is
//! first-party and cooperative, not because it is compelled.
//!
//! The limit that belongs next to this file, not in a document nobody opens:
//! **once a tool call reaches an MCP server, rad constrains nothing.** That
//! server is an OS process with the user's full privileges. The effective
//! defence is which servers get registered, and that decision is outside rad
//! (§3.4.4).
#![deny(clippy::pedantic)]

mod rules;

#[derive(serde::Deserialize)]
pub struct CheckReq {
    /// Carried for the refusal message and for `allowed_tools` later
    /// (§4.5.2); the blocklist itself matches on `arguments` only.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: String,
}

#[derive(serde::Serialize)]
pub struct CheckRes {
    pub allow: bool,
    pub reason: String,
}

/// A refusal is `Ok(allow: false)`, never `Err`.
///
/// The split is load-bearing on the calling side: `mcp` treats a failed
/// `dispatch.call` as a denial, because a policy module that crashed or went
/// missing mid-run must not read as approval. Reserving `Err` for that leaves
/// "the policy answered, and the answer was no" unambiguous.
fn check(req: CheckReq) -> CheckRes {
    let CheckReq { name, arguments } = req;
    match rules::check(&arguments) {
        Some(reason) => CheckRes {
            allow: false,
            reason: format!("tool '{name}': {reason}"),
        },
        None => CheckRes {
            allow: true,
            reason: String::new(),
        },
    }
}

rad_sdk::module! {
    wit: "../../wit/kernel/kernel.wit",
    name: "policy",
    version: "0.1.0",
    methods: {
        "policy.check" => rad_sdk::infallible(check),
    }
}
