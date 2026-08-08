//! `proc-spawn` and the `process` / `byte-stream` resources, driven from a real
//! module (AWU 963).
//!
//! The first syscall with an implementation behind it. Everything else in
//! `src/kernel/host.rs` still answers 501, so these tests are also the check
//! that the resource plumbing — tables, handles, drop — works at all.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn spawn_wasm() -> PathBuf {
    ["debug", "release"]
        .iter()
        .map(|p| PathBuf::from(format!("target/wasm32-wasip2/{p}/spawn_module.wasm")))
        .find(|p| p.exists())
        .expect("spawn_module.wasm not built for wasm32-wasip2")
}

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "spawn",
        &spawn_wasm(),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .expect("spawn module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("spawn".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

fn call(k: &KernelShared, method: &str, payload: &str) -> Result<serde_json::Value, String> {
    k.call("test", "spawn", method, payload)
        .map(|reply| serde_json::from_str(&reply).unwrap())
}

#[test]
fn a_module_runs_a_command_and_reads_its_output() {
    let k = kernel();
    let res = call(&k, "spawn.run", r#"{"argv":["echo","hello from a child"]}"#)
        .expect("spawn.run should succeed");
    assert_eq!(res["stdout"].as_str().unwrap().trim(), "hello from a child");
    assert_eq!(res["exit_code"], 0);
}

/// The argv list is passed through, not parsed. `spawn_bash_process` splits a
/// command string on whitespace and mangles quoted arguments — a bug that once
/// corrupted every tool result. An argv list has no such failure mode, and this
/// is the case that would expose it: one argument containing spaces.
#[test]
fn an_argument_containing_spaces_arrives_as_one_argument() {
    let k = kernel();
    let res = call(&k, "spawn.run", r#"{"argv":["echo","-n","a b c"]}"#)
        .expect("spawn.run should succeed");
    assert_eq!(res["stdout"], "a b c");
}

#[test]
fn a_nonzero_exit_is_reported_rather_than_raised() {
    let k = kernel();
    let res = call(&k, "spawn.run", r#"{"argv":["sh","-c","exit 3"]}"#)
        .expect("a failing command is still a successful spawn");
    assert_eq!(res["exit_code"], 3);
}

/// The MCP shape: write to a child that stays alive, read the reply back.
/// `cat` echoes a line without exiting, which is what makes this different
/// from `spawn.run` — nothing ever closes the stream.
#[test]
fn a_module_writes_to_a_live_child_and_reads_the_reply() {
    let k = kernel();
    let res = call(&k, "spawn.pipe", r#"{"argv":["cat"],"input":"round trip"}"#)
        .expect("spawn.pipe should succeed");
    assert_eq!(res["line"], "round trip");
}

#[test]
fn a_missing_program_is_an_error_naming_it() {
    let k = kernel();
    let err = call(&k, "spawn.run", r#"{"argv":["rad_no_such_program_xyz"]}"#)
        .expect_err("spawning a nonexistent program must fail");
    assert!(err.contains("proc-spawn failed"), "{err}");
}

#[test]
fn an_empty_argv_is_rejected() {
    let k = kernel();
    let err =
        call(&k, "spawn.run", r#"{"argv":[]}"#).expect_err("an empty argv has no program to run");
    assert!(err.contains("program name"), "{err}");
}

/// A child that outlives the call must not be left behind. The `process`
/// resource's `drop` kills the group, so once the module returns nothing is
/// still running — otherwise every MCP-style spawn would leak a process.
///
/// `sleep`, not `cat`: `cat` exits on its own the moment its stdin pipe closes,
/// which happens as the module's resources drop. A test written with `cat`
/// passes whether or not the kernel kills anything, which is how the first
/// version of this test was wrong.
#[test]
fn a_child_that_ignores_stdin_is_killed_when_the_module_drops_its_handle() {
    let k = kernel();
    // Distinctive duration so `pgrep -f` cannot match an unrelated sleep.
    let res = call(&k, "spawn.leak", r#"{"argv":["sleep","2471"]}"#);
    assert!(res.is_ok(), "{res:?}");

    std::thread::sleep(std::time::Duration::from_millis(300));
    let survivors = std::process::Command::new("pgrep")
        .args(["-f", "sleep 2471"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    assert!(
        survivors.is_empty(),
        "the child survived the call (pids: {survivors})"
    );
}
