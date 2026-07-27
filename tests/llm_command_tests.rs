// Coverage for the `/llm` subcommand registry redesign (Phase 47-1b):
// flag-based `add` (fixing the old positional-only form that couldn't set
// `api_key` without `model`), `delete`, and registered-profile-names
// taking priority over reserved subcommand keywords.
//
// `Orchestrator.config` is `pub(crate)`, so these (external, `tests/`-crate)
// tests observe state through `/llm list`'s rendered output rather than
// reaching into config fields directly.
//
// IMPORTANT: `add`/`test`/`model`/`delete` all persist via
// `save_global_config`, which targets `RAD_TEST_CONFIG_HOME` when set
// (falling back to the real `~/.rad/config.json` otherwise — see AWU 918).
// `RAD_TEST_CONFIG_HOME` is process-global env state, so every test here
// holds `TEST_MUTEX` for its full duration to serialize against the others
// in this binary (same pattern as `tests/multi_extension_tests.rs`). A
// prior version of this file didn't do this and clobbered a real
// developer's `~/.rad/config.json` with tempdir paths and dummy profiles.
use parking_lot::Mutex;
use rad::command::llm::execute_llm;
use rad::config::{Config, CoreConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use std::sync::Arc;
use tempfile::tempdir;

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

// A loopback port nothing listens on: connection attempts fail fast
// (connection refused) instead of timing out, keeping these tests quick.
const DEAD_URL: &str = "http://127.0.0.1:1";

/// Bundles the orchestrator with the guards that must outlive it: the
/// process-wide `RAD_TEST_CONFIG_HOME` mutex lock, and the two tempdirs
/// (workspace, fake config-home) so they aren't deleted mid-test.
struct TestCtx {
    _lock: std::sync::MutexGuard<'static, ()>,
    _workspace_tmp: tempfile::TempDir,
    _config_home_tmp: tempfile::TempDir,
    orchestrator: Arc<Orchestrator>,
}

fn test_orchestrator() -> TestCtx {
    let lock = TEST_MUTEX.lock().unwrap();

    let workspace_tmp = tempdir().unwrap();
    let workspace = workspace_tmp.path().to_path_buf();
    let snapshot = workspace_tmp.path().join("snapshots");
    let log = workspace_tmp.path().join("logs");
    std::fs::create_dir_all(&snapshot).unwrap();
    std::fs::create_dir_all(&log).unwrap();

    let config_home_tmp = tempdir().unwrap();
    // Safety: single-threaded with respect to this var, guaranteed by
    // holding `TEST_MUTEX` for this whole `TestCtx`'s lifetime.
    unsafe {
        std::env::set_var("RAD_TEST_CONFIG_HOME", config_home_tmp.path());
    }

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshot.to_string_lossy().to_string(),
            log: log.to_string_lossy().to_string(),
            hitl_enabled: false,
            verification_command: None,
            ..Default::default()
        },
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let orchestrator = Arc::new(Orchestrator::new(config, "test_session".to_string(), dag, None));

    TestCtx {
        _lock: lock,
        _workspace_tmp: workspace_tmp,
        _config_home_tmp: config_home_tmp,
        orchestrator,
    }
}

fn list(orch: &Orchestrator) -> String {
    execute_llm("list", orch)
}

#[test]
fn test_llm_add_sets_api_key_without_model() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;

    // The old positional-only `add <name> <url> [model] [api_key]` form
    // made this impossible without also supplying a model.
    let msg = execute_llm(&format!("add local {DEAD_URL} --api-key secret123"), orch);
    assert!(msg.contains("Added LLM profile"), "unexpected message: {msg}");

    let listing = list(orch);
    assert!(listing.contains("local"));
    assert!(listing.contains("[auth: yes]"));
    assert!(!listing.contains("[model:"), "no model was set: {listing}");
}

#[test]
fn test_llm_add_accepts_flags_in_either_order() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let msg = execute_llm(&format!("add local {DEAD_URL} --api-key secret --model qwen"), orch);
    assert!(msg.contains("Added LLM profile"), "unexpected message: {msg}");

    let listing = list(orch);
    assert!(listing.contains("[model: qwen]"));
    assert!(listing.contains("[auth: yes]"));
}

#[test]
fn test_llm_add_rejects_unrecognized_flag() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let msg = execute_llm(&format!("add local {DEAD_URL} --bogus x"), orch);
    assert!(msg.contains("Unrecognized argument"), "unexpected message: {msg}");
    assert!(!list(orch).contains("local"));
}

#[test]
fn test_llm_delete_removes_profile() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let _ = execute_llm(&format!("add local {DEAD_URL}"), orch);
    assert!(list(orch).contains("local"));

    let msg = execute_llm("delete local", orch);
    assert!(msg.contains("Removed LLM profile"), "unexpected message: {msg}");
    assert!(!list(orch).contains("local"));
}

#[test]
fn test_llm_delete_unknown_profile_errors() {
    let ctx = test_orchestrator();
    let msg = execute_llm("delete nonexistent", &ctx.orchestrator);
    assert!(msg.contains("not found"), "unexpected message: {msg}");
}

#[test]
fn test_llm_delete_active_profile_falls_back() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let _ = execute_llm(&format!("add only {DEAD_URL}"), orch);
    assert!(list(orch).contains("only \x1b[1;32m(active)"));

    let _ = execute_llm("delete only", orch);
    let listing = list(orch);
    assert!(!listing.contains("only"));
}

#[test]
fn test_llm_profile_name_takes_priority_over_reserved_keyword() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    // Register a second profile so "test" switching to it is observable.
    let _ = execute_llm(&format!("add primary {DEAD_URL}"), orch);
    let _ = execute_llm(&format!("add test {DEAD_URL}"), orch);
    // `add` sets `active` only if it was previously unset, so it's still
    // "primary" at this point.
    assert!(list(orch).contains("primary \x1b[1;32m(active)"));

    // A bare `/llm test` must switch to the profile named "test", not run
    // the health-check subcommand, since an exact profile-name match wins.
    let msg = execute_llm("test", orch);
    assert!(msg.contains("Switched active LLM server profile"), "unexpected message: {msg}");
    assert!(list(orch).contains("test \x1b[1;32m(active)"));

    // The explicit escape hatch always reaches the real subcommand
    // regardless of any profile name.
    let msg = execute_llm("switch primary", orch);
    assert!(msg.contains("Switched active LLM server profile"), "unexpected message: {msg}");
    assert!(list(orch).contains("primary \x1b[1;32m(active)"));
}

#[test]
fn test_llm_list_shows_all_subcommands_even_when_empty() {
    let ctx = test_orchestrator();
    let msg = execute_llm("", &ctx.orchestrator);
    assert!(msg.contains("/llm add"));
    assert!(msg.contains("/llm delete"));
    assert!(msg.contains("/llm test"));
    assert!(msg.contains("/llm model"));
    assert!(msg.contains("/llm context"));
}

#[test]
fn test_llm_context_sets_manual_override_on_active_profile() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let _ = execute_llm(&format!("add local {DEAD_URL}"), orch);

    let msg = execute_llm("context 8192", orch);
    assert!(msg.contains("Set context window"), "unexpected message: {msg}");
    assert!(msg.contains("8192"));

    let listing = list(orch);
    assert!(listing.contains("[ctx: 8192 tok]"), "unexpected listing: {listing}");
}

#[test]
fn test_llm_context_rejects_non_numeric_argument() {
    let ctx = test_orchestrator();
    let orch = &ctx.orchestrator;
    let _ = execute_llm(&format!("add local {DEAD_URL}"), orch);

    let msg = execute_llm("context not-a-number", orch);
    assert!(msg.contains("Usage: /llm context"), "unexpected message: {msg}");
}

#[test]
fn test_llm_context_errors_when_no_active_profile() {
    let ctx = test_orchestrator();
    let msg = execute_llm("context 8192", &ctx.orchestrator);
    assert!(msg.contains("No active LLM profile"), "unexpected message: {msg}");
}
