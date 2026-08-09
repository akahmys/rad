//! Asks `policy` before a tool runs (ARCHITECTURE-NEXT.md §3.4.3).
//!
//! This is the whole of rad's tool-execution policy surface — one hook, here,
//! rather than the five the extension host carried. Four of those five guarded
//! imports that no guest had called since AWU 965 and 969; AWU 970 removed
//! them after probing, and AWU 972 moved the one that was live.
//!
//! **The two kernel syscalls deliberately have no check of their own.** §3.4.2
//! discarded a `syscall-gate` role and the reasoning holds in the code:
//! model-derived data reaches neither `proc-spawn`'s `argv` nor `net-open`'s
//! URL. This module spawns servers named in `kernel.config` and sends the
//! model's tool calls over an already-running server's *stdin*, where they are
//! not a syscall at all. The one path that does put model text in `argv` is
//! `testmode`'s `bash -c`, and it is downstream of this gate — which
//! `tests/policy_gate_tests.rs` asserts rather than assumes.
use std::cell::RefCell;

thread_local! {
    /// `None` until the first tool call. Cached because it cannot change
    /// afterwards: every module is registered during boot, before anything can
    /// ask for a tool.
    static POLICY_LOADED: RefCell<Option<bool>> = const { RefCell::new(None) };
}

fn policy_loaded() -> bool {
    POLICY_LOADED.with(|cell| {
        let mut slot = cell.borrow_mut();
        *slot.get_or_insert_with(|| {
            let Ok(raw) = crate::dispatch::call("kernel", "kernel.modules", "{}") else {
                return false;
            };
            serde_json::from_str::<Vec<String>>(&raw)
                .is_ok_and(|names| names.iter().any(|n| n == "policy"))
        })
    })
}

/// `Ok(())` to proceed, `Err(reason)` to refuse.
///
/// Asking `kernel.modules` first is what separates "no policy is configured"
/// from "the policy said no", without matching on an error string. It matters
/// because the two answers must go opposite ways: an absent policy allows —
/// the mechanism is opt-in, exactly as the extension was — while a policy that
/// is present but unreachable refuses. A crashed policy that read as approval
/// would be a gate that disappears precisely when something is wrong.
pub(crate) fn check(name: &str, arguments: &str) -> Result<(), String> {
    if !policy_loaded() {
        return Ok(());
    }
    let payload = serde_json::json!({ "name": name, "arguments": arguments }).to_string();
    let reply = crate::dispatch::call("policy", "policy.check", &payload)
        .map_err(|e| format!("policy is loaded but did not answer: {e}"))?;
    let verdict: serde_json::Value = serde_json::from_str(&reply)
        .map_err(|e| format!("policy returned something that is not JSON: {e}"))?;
    // A reply missing `allow` refuses, for the same reason an unreachable
    // policy does: the answer was not "yes".
    if verdict.get("allow").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(());
    }
    // "Operation rejected by ..." is the shape the DAG and the model have seen
    // since the extension era; only the actor's name changes, because the
    // security extension no longer exists.
    Err(format!(
        "Operation rejected by policy: {}",
        verdict
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("no reason given")
    ))
}
