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
