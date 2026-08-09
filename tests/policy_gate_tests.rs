//! The gate inside `modules/mcp`, and the two properties stage 7's design
//! rests on that were assumptions until here.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn wasm(name: &str) -> PathBuf {
    for profile in ["debug", "release"] {
        let p = PathBuf::from(format!("target/wasm32-wasip2/{profile}/{name}.wasm"));
        if p.exists() {
            return p;
        }
    }
    panic!("{name}.wasm not built for wasm32-wasip2; run cargo build --target wasm32-wasip2")
}

fn load(shared: &Arc<KernelShared>, name: &str, artefact: &str) {
    let rt = ModuleRuntime::load(
        name,
        &wasm(artefact),
        &shared.engine,
        Arc::downgrade(shared),
    )
    .unwrap_or_else(|e| panic!("{name} should load: {e}"));
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert(name.to_string(), Arc::new(Mutex::new(rt)));
}

/// `mcp` plus, optionally, `policy`. `RAD_TEST_PORT` puts `mcp` in the
/// synthetic-tool mode this suite drives.
fn kernel(policy: Option<serde_json::Value>) -> Arc<KernelShared> {
    // Before the component is instantiated, not after: `mcp` decides whether
    // it is in testmode from the environment it is handed at instantiation.
    // Setting this after `load` left the module in real-MCP mode, where the
    // synthetic `execute` does not exist — and the blocked-command test still
    // passed, because the gate refuses above the point where that matters.
    // Every test in this binary sets the same value, so the parallel threads
    // do not race to a different answer.
    unsafe { std::env::set_var("RAD_TEST_PORT", "1") };
    let shared = KernelShared::new();
    load(&shared, "mcp", "mcp_module");
    if let Some(config) = policy {
        load(&shared, "policy", "policy_module");
        shared.set_module_config("policy", config);
    }
    shared
}

fn call_tool(kernel: &Arc<KernelShared>, name: &str, arguments: &str) -> Result<String, String> {
    let payload = serde_json::json!({ "name": name, "arguments": arguments }).to_string();
    kernel.call("host", "mcp", "mcp.tools.call", &payload)
}

/// The property the whole "no check in `proc-spawn`" argument rests on
/// (§3.4.2). `testmode`'s `execute` runs its command through `bash -c`, which
/// is the one path where the model's own text reaches `proc-spawn`'s `argv` —
/// so if the gate did not sit above it, the kernel would spawn a blocked
/// command with nothing in the way. The marker file is the witness: it can
/// only exist if bash actually ran.
#[test]
fn a_blocked_command_never_reaches_proc_spawn() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("blocked_command_ran");
    let k = kernel(Some(serde_json::json!({
        "block_command_patterns": ["blocked_command"]
    })));

    let arguments =
        serde_json::json!({ "command": format!("touch '{}' # blocked_command", marker.display()) })
            .to_string();
    let err = call_tool(&k, "execute", &arguments)
        .expect_err("a blocked command must not run")
        .to_string();

    assert!(err.contains("Operation rejected by policy"), "{err}");
    assert!(
        !marker.exists(),
        "the gate let the command through to proc-spawn: {} exists",
        marker.display()
    );
}

/// The same command with no `policy` module loaded does run — without this the
/// test above would pass for the wrong reason, since a tool that never works
/// also never reaches `proc-spawn`.
#[test]
fn the_same_command_runs_when_no_policy_is_loaded() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("unblocked_command_ran");
    let k = kernel(None);

    let arguments =
        serde_json::json!({ "command": format!("touch '{}'", marker.display()) }).to_string();
    call_tool(&k, "execute", &arguments).expect("with no policy loaded the tool must run");

    assert!(
        marker.exists(),
        "the tool did not actually run, so the blocked-case test proves nothing"
    );
}

/// A configured policy still lets through what it has no pattern for, through
/// the gate and all the way to a real `proc-spawn`.
#[test]
fn an_unmatched_command_runs_with_a_policy_loaded() {
    let dir = tempfile::tempdir().unwrap();
    let marker = dir.path().join("allowed_command_ran");
    let k = kernel(Some(serde_json::json!({
        "block_command_patterns": ["blocked_command"]
    })));

    let arguments =
        serde_json::json!({ "command": format!("touch '{}'", marker.display()) }).to_string();
    call_tool(&k, "execute", &arguments).expect("an unmatched command must run");

    assert!(marker.exists(), "an allowed command did not run");
}

/// PLANS recorded the nesting as worth a test rather than an assumption. The
/// gate makes `host -> mcp -> policy` a two-frame stack, and
/// `a_blocked_command_never_reaches_proc_spawn` above already exercises it
/// from the inside. This asserts the same shape from the outside — entering
/// `mcp` as the *second* frame is ordinary, not a cycle — so that if the stack
/// bookkeeping ever regressed into treating depth as re-entry, the failure
/// names `mcp` rather than surfacing as a mysteriously refused tool call.
///
/// The re-entry direction (a `policy` that called back into `mcp` being
/// refused by name rather than deadlocking on `mcp`'s store lock) is covered
/// by `kernel_dispatch_tests::a_cycle_is_an_error_naming_the_chain`, which
/// drives it with `relay` — `policy` calls nobody, so it cannot demonstrate it.
#[test]
fn entering_mcp_as_a_nested_frame_is_not_a_cycle() {
    unsafe { std::env::set_var("RAD_TEST_PORT", "1") };
    let shared = KernelShared::new();
    load(&shared, "mcp", "mcp_module");
    load(&shared, "relay", "relay_module");

    let inner = serde_json::json!({
        "target": "mcp",
        "method": "mcp.tools.list",
        "payload": "{}"
    })
    .to_string();
    let reply = shared
        .call("host", "relay", "relay.hop", &inner)
        .expect("host -> relay -> mcp is two frames, not a cycle");
    assert!(
        reply.contains("execute"),
        "the nested call should carry mcp's real answer back: {reply}"
    );
}
