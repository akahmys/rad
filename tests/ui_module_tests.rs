//! The host's terminal driving `modules/ui` (AWU 987).
//!
//! `modules/ui/src/screen/tests.rs` covers the buffering rules inside the
//! module. What only exists here is the seam: that `get_terminal()`'s calls
//! actually reach it, and that with no module loaded the host's own copy still
//! behaves as it always did.
//!
//! **`get_terminal()` is a process-wide singleton and `attach_kernel` replaces
//! what it points at**, so these tests cannot run concurrently — the same
//! reason `llm_command_tests.rs` serialises its environment. Each test builds
//! its own orchestrator, which re-attaches, so order does not matter once they
//! are serialised.
use parking_lot::Mutex;
use rad::config::{Config, CoreConfig, ModuleConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use rad::terminal::{TerminalState, get_terminal};
use std::sync::Arc;

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn orchestrator(dir: &std::path::Path, with_module: bool) -> Arc<Orchestrator> {
    let workspace = dir.join("workspace");
    std::fs::create_dir_all(&workspace).unwrap();
    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: dir.join("snapshots").to_string_lossy().to_string(),
            log: dir.join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        modules: if with_module {
            vec![ModuleConfig {
                name: "ui".to_string(),
                source: "target/wasm32-wasip2/debug/ui_module.wasm".to_string(),
                enabled: true,
                config: serde_json::Value::Null,
            }]
        } else {
            vec![]
        },
        ..Default::default()
    };
    Arc::new(Orchestrator::new(
        config,
        "ui-test".to_string(),
        Arc::new(Mutex::new(Dag::new())),
        None,
    ))
}

/// What the module says it is doing. Printing is not observable from a test —
/// the module writes to inherited stdout — so this is the part that can be
/// asserted, and it is also the part that can be wrong.
fn status(orch: &Arc<Orchestrator>) -> serde_json::Value {
    let reply = orch
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("test", "ui", "ui.status", "{}"))
        .expect("the kernel is loaded")
        .expect("ui.status must answer");
    serde_json::from_str(&reply).expect("ui.status must return JSON")
}

#[test]
fn the_hosts_terminal_calls_reach_the_module() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);

    get_terminal().set_state(TerminalState::Thinking);
    assert_eq!(status(&orch)["state"], "thinking");

    get_terminal().set_state(TerminalState::Streaming);
    assert_eq!(status(&orch)["state"], "streaming");
}

/// The rule the state machine exists for, driven from the host: a log arriving
/// mid-response is held back rather than landing in the middle of it.
#[test]
fn a_log_written_through_the_host_is_deferred_while_streaming() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);

    get_terminal().set_state(TerminalState::Streaming);
    get_terminal().write_log("held".to_string());
    assert_eq!(status(&orch)["deferred"], 1);

    // Going idle releases it, which is the half that would strand output if it
    // were missing.
    get_terminal().set_state(TerminalState::Idle);
    assert_eq!(status(&orch)["deferred"], 0);
}

/// A token moves the terminal on its own — the transition that erases the
/// thinking indicator before the first token prints.
#[test]
fn a_token_written_through_the_host_moves_the_module_to_streaming() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);

    get_terminal().set_state(TerminalState::Thinking);
    get_terminal().write_llm_token("hello");
    assert_eq!(status(&orch)["state"], "streaming");
}

/// With no module the host's own copy runs, which is what every rad without a
/// `ui` module does. Without this, the tests above would be equally consistent
/// with the fallback being broken.
#[test]
fn without_the_module_the_host_still_handles_its_own_terminal() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), false);

    assert!(
        orch.kernel
            .lock()
            .as_ref()
            .is_some_and(|k| k.provider_of("ui.state").is_none()),
        "this test is only meaningful with no ui module loaded"
    );

    // Nothing to assert against inside the module, so what is checked is that
    // the host path runs without panicking and the singleton is usable — the
    // buffering itself is covered by the module's own unit tests, and by the
    // host's original behaviour, which is unchanged.
    get_terminal().set_state(TerminalState::Streaming);
    get_terminal().write_log("local".to_string());
    get_terminal().set_state(TerminalState::Idle);
}

#[test]
fn the_module_declares_every_method_the_host_calls() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().unwrap();
    let orch = orchestrator(dir.path(), true);
    let kernel = orch.kernel.lock().clone().expect("the kernel is loaded");

    // `src/terminal.rs` calls exactly these three; a drift in any name makes
    // the host fall back silently and every other test here still pass.
    for method in ["ui.state", "ui.token", "ui.log"] {
        assert_eq!(
            kernel.provider_of(method).as_deref(),
            Some("ui"),
            "the host calls '{method}' and nothing provides it"
        );
    }
}
