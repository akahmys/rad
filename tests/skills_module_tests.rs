//! `modules/skills` over the kernel boundary, against a real `.agents/skills/`
//! tree on disk.
//!
//! Discovery moved from `ListDir`/`FileRead` host RPCs to `std::fs`, so what
//! the module can reach is now decided by the host's WASI preopens. That is
//! only demonstrable with actual files, which is what this covers.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn skills_wasm() -> PathBuf {
    ["debug", "release"]
        .iter()
        .map(|p| PathBuf::from(format!("target/wasm32-wasip2/{p}/skills_module.wasm")))
        .find(|p| p.exists())
        .expect("skills_module.wasm not built for wasm32-wasip2")
}

/// Serialises these tests. They share one directory in the crate root, and
/// cargo runs tests in a thread pool by default — the same `TEST_MUTEX` pattern
/// the rest of the suite uses for process-global state.
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The module resolves `.agents/skills` relative to its working directory, so
/// the test has to write into the crate root.
///
/// It removes only the directories it created. Wiping `.agents/skills`
/// wholesale would delete a developer's real skills — a test that destroys user
/// data to clean up after itself is worse than one that leaves a mess.
struct SkillFixture {
    created: Vec<PathBuf>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl SkillFixture {
    fn new(skills: &[(&str, &str)]) -> Self {
        let guard = TEST_MUTEX
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let root = PathBuf::from(".agents/skills");
        let mut created = Vec::new();
        for (name, content) in skills {
            let dir = root.join(name);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("SKILL.md"), content).unwrap();
            created.push(dir);
        }
        Self {
            created,
            _guard: guard,
        }
    }
}

impl Drop for SkillFixture {
    fn drop(&mut self) {
        for dir in &self.created {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

fn kernel() -> Arc<KernelShared> {
    let shared = KernelShared::new();
    let rt = ModuleRuntime::load(
        "skills",
        &skills_wasm(),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .expect("skills module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("skills".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

#[test]
fn discovers_skills_from_disk_and_executes_one() {
    let _fixture = SkillFixture::new(&[(
        "rad_test_review",
        "---\ndescription: Review a PR.\n---\n\nCheck $ARGUMENTS carefully.\n",
    )]);
    let k = kernel();

    let listed = k
        .call("test", "skills", "skills.tools.list", "{}")
        .expect("listing should succeed");
    assert!(
        listed.contains("rad_test_review") && listed.contains("Review a PR."),
        "{listed}"
    );

    let called = k
        .call(
            "test",
            "skills",
            "skills.tools.call",
            r#"{"name":"rad_test_review","arguments":"{\"args\":\"the diff\"}"}"#,
        )
        .expect("execution should succeed");
    // Substituted, and returned directly — the extension had to escape this
    // through `echo -n` because its WIT return type was an execution handle.
    assert!(called.contains("Check the diff carefully."), "{called}");
}

#[test]
fn a_legacy_subagent_mode_line_no_longer_blocks_execution() {
    // The extension refused these with "not implemented". `mode` is gone
    // (ARCHITECTURE-NEXT.md §1.2), so the skill simply runs.
    let _fixture = SkillFixture::new(&[(
        "rad_test_legacy",
        "---\ndescription: Legacy.\nmode: subagent\n---\n\nStill works.\n",
    )]);
    let k = kernel();
    let called = k
        .call(
            "test",
            "skills",
            "skills.tools.call",
            r#"{"name":"rad_test_legacy","arguments":"{}"}"#,
        )
        .expect("a legacy mode line must not block execution");
    assert!(called.contains("Still works."), "{called}");
}

#[test]
fn an_unknown_skill_is_reported_by_name() {
    let _fixture = SkillFixture::new(&[]);
    let k = kernel();
    let err = k
        .call(
            "test",
            "skills",
            "skills.tools.call",
            r#"{"name":"no_such_skill","arguments":"{}"}"#,
        )
        .unwrap_err();
    assert!(err.contains("no_such_skill"), "{err}");
}

// --- The host-side aggregation layer (`kernel::tools`) -----------------------
//
// `skills.tools.list` above proves the module answers. These prove the host
// finds that answer without being told which module to ask, which is what the
// tool path needs: the registry maps one method to one module, so providers
// namespace their methods and the host walks the modules.

/// A module that provides no tools at all — the aggregate must skip it rather
/// than fail. Every non-tool module (`context-tools`, and every module a third
/// party writes) is in this position.
fn echo_wasm() -> PathBuf {
    ["debug", "release"]
        .iter()
        .map(|p| PathBuf::from(format!("target/wasm32-wasip2/{p}/echo_module.wasm")))
        .find(|p| p.exists())
        .expect("echo_module.wasm not built for wasm32-wasip2")
}

fn kernel_with_echo() -> Arc<KernelShared> {
    let shared = kernel();
    let rt = ModuleRuntime::load(
        "echo",
        &echo_wasm(),
        &shared.engine,
        Arc::downgrade(&shared),
    )
    .expect("echo module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("echo".to_string(), Arc::new(Mutex::new(rt)));
    shared
}

#[test]
fn the_aggregate_tool_list_carries_module_tools_in_openai_shape() {
    let _fixture = SkillFixture::new(&[(
        "rad_test_agg",
        "---\ndescription: Aggregated.\n---\n\nBody.\n",
    )]);
    let k = kernel_with_echo();

    let tools = rad::kernel::tools::list(&k);
    let found = tools
        .iter()
        .find(|t| t.pointer("/function/name").and_then(|n| n.as_str()) == Some("rad_test_agg"))
        .unwrap_or_else(|| panic!("rad_test_agg missing from {tools:?}"));
    // The connector sends this straight to the model, so the shape matters as
    // much as the presence.
    assert_eq!(found.get("type").and_then(|t| t.as_str()), Some("function"));
    assert_eq!(
        found
            .pointer("/function/description")
            .and_then(|d| d.as_str()),
        Some("Aggregated.")
    );
}

#[test]
fn execute_routes_to_the_owning_module_by_tool_name() {
    let _fixture = SkillFixture::new(&[(
        "rad_test_route",
        "---\ndescription: Routed.\n---\n\nRan with $ARGUMENTS.\n",
    )]);
    let k = kernel_with_echo();

    let out = rad::kernel::tools::execute(&k, "rad_test_route", r#"{"args":"input"}"#)
        .expect("a module owns this tool")
        .expect("and running it should succeed");
    // Trailing newline included: the body is returned verbatim, exactly as
    // the extension's `echo -n '<body>'` did.
    assert_eq!(out, "Ran with input.\n");
}

/// `None`, not `Err`. During the migration extensions still provide most tools,
/// and turning "no module has it" into a failure would break every one of them.
#[test]
fn execute_declines_a_tool_no_module_owns() {
    let _fixture = SkillFixture::new(&[]);
    let k = kernel_with_echo();
    assert!(rad::kernel::tools::execute(&k, "bash", "{}").is_none());
}

/// User-global skills, which live outside the working directory.
///
/// This is the case every test above missed: they all write to
/// `.agents/skills` under the crate root, which the `"."` preopen covers. With
/// no `$HOME` preopen and no `$HOME` in the guest environment,
/// `~/.rad/skills` was simply unreachable — the module returned an empty list
/// and every existing test still passed. The extension being replaced could
/// read it, so losing it would have been a silent regression.
///
/// `HOME` is redirected to a temporary directory rather than written to for
/// real: the point is to prove the mechanism, not to touch a developer's actual
/// skills.
#[test]
fn user_global_skills_under_home_are_discoverable() {
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let home = tempfile::tempdir().unwrap();
    let dir = home.path().join(".rad/skills/rad_test_global");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("SKILL.md"),
        "---\ndescription: From HOME.\n---\n\nGlobal body.\n",
    )
    .unwrap();

    let original = std::env::var("HOME").ok();
    // SAFETY: the module reads `$HOME` at load time, so it has to be set in
    // this process. Serialised by `TEST_MUTEX` above.
    unsafe { std::env::set_var("HOME", home.path()) };
    let listed = std::panic::catch_unwind(|| {
        let shared = KernelShared::new();
        let rt = ModuleRuntime::load(
            "skills",
            &skills_wasm(),
            &shared.engine,
            Arc::downgrade(&shared),
        )
        .expect("skills module should load");
        shared
            .registry
            .lock()
            .register(rt.manifest.clone())
            .unwrap();
        shared
            .modules
            .lock()
            .insert("skills".to_string(), Arc::new(Mutex::new(rt)));
        shared.call("test", "skills", "skills.tools.list", "{}")
    });
    // Restored before asserting, so a failure does not leave `HOME` pointing at
    // a deleted directory for the rest of the binary.
    unsafe {
        match original {
            Some(h) => std::env::set_var("HOME", h),
            None => std::env::remove_var("HOME"),
        }
    }

    let listed = listed
        .expect("loading must not panic")
        .expect("listing should succeed");
    assert!(
        listed.contains("rad_test_global") && listed.contains("From HOME."),
        "user-global skills unreachable: {listed}"
    );
}
