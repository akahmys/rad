//! `modules/policy` through a real kernel. The unit tests in
//! `modules/policy/src/rules/tests.rs` cover the matching; these cover the part
//! those cannot reach — that the module actually reads `kernel.config`, which
//! is the one place the port from `GetExtensionConfig` could have gone wrong.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

/// Panics rather than skipping when the component is missing, for the reason
/// `kernel_dispatch_tests` records: a skipped test reports success.
fn wasm(name: &str) -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/{name}.wasm"));
        if p.exists() {
            return p;
        }
    }
    panic!("{name}.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2")
}

/// A kernel holding `policy` alone, with `config` as its module config.
fn kernel_with_policy(config: serde_json::Value) -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "policy",
        &wasm("policy_module"),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .unwrap_or_else(|e| panic!("policy should load: {e}"));
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("policy".to_string(), Arc::new(Mutex::new(rt)));
    shared.set_module_config("policy", config);
    shared
}

fn check(kernel: &Arc<KernelShared>, name: &str, arguments: &str) -> serde_json::Value {
    let payload = serde_json::json!({ "name": name, "arguments": arguments }).to_string();
    let reply = kernel
        .call("test", "policy", "policy.check", &payload)
        .expect("policy.check must answer");
    serde_json::from_str(&reply).expect("policy.check must return an object")
}

/// Opt-in, exactly as the extension was: a `policy` module with no config at
/// all is a no-op, not a lockout. This is what
/// `security_guard_policy_tests.rs` asserted end to end for the extension.
#[test]
fn an_unconfigured_policy_allows_everything() {
    let k = kernel_with_policy(serde_json::json!({}));
    let res = check(&k, "execute", r#"{"command":"blocked_command"}"#);
    assert_eq!(res["allow"], true, "{res}");
}

/// The one thing the unit tests cannot reach: the patterns arriving over
/// `kernel.config`. If the port from `GetExtensionConfig` were wrong, this is
/// the test that fails and the unit tests that would not notice.
#[test]
fn patterns_from_kernel_config_are_applied() {
    let k = kernel_with_policy(serde_json::json!({
        "block_command_patterns": ["blocked_command", "blocked.txt"]
    }));
    let res = check(&k, "execute", r#"{"command":"blocked_command --now"}"#);
    assert_eq!(res["allow"], false, "{res}");
}

/// A refusal has to say why, and name the tool. The model reads this string;
/// an opaque denial is one it will retry.
#[test]
fn a_refusal_names_the_tool_and_the_pattern() {
    let k = kernel_with_policy(serde_json::json!({
        "block_command_patterns": ["blocked.txt"]
    }));
    let res = check(&k, "write", r#"{"path":"blocked.txt"}"#);
    let reason = res["reason"].as_str().unwrap_or_default();
    assert!(
        reason.contains("write"),
        "reason must name the tool: {reason}"
    );
    assert!(
        reason.contains("blocked.txt"),
        "reason must name the pattern: {reason}"
    );
}

/// A configured policy still allows what it has no pattern for — the blocklist
/// is a blocklist, not an allowlist.
#[test]
fn a_configured_policy_allows_an_unmatched_call() {
    let k = kernel_with_policy(serde_json::json!({
        "block_command_patterns": ["blocked_command"]
    }));
    let res = check(&k, "execute", r#"{"command":"ls -la"}"#);
    assert_eq!(res["allow"], true, "{res}");
}

/// A denial is `Ok(allow: false)`, never a dispatch error. `modules/mcp` reads
/// a failed `call` as a denial too, so if this ever started erroring the two
/// would be indistinguishable and a crashed policy would look like a refusal.
#[test]
fn a_denial_is_a_successful_call() {
    let k = kernel_with_policy(serde_json::json!({
        "block_command_patterns": ["blocked_command"]
    }));
    let payload = serde_json::json!({
        "name": "execute",
        "arguments": r#"{"command":"blocked_command"}"#
    })
    .to_string();
    let reply = k.call("test", "policy", "policy.check", &payload);
    assert!(reply.is_ok(), "a denial must not surface as Err: {reply:?}");
}
