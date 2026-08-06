use rad::config::PermissionConfig;
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::RasCoreEvent;
use rad::process::ProcessManager;
use rad::wasm::WasmRuntime;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

struct MockNetwork;
impl rad::subsystems::NetworkSubsystem for MockNetwork {
    fn open_http_stream(
        &self,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: &str,
        _event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
        _llm_timeout_policy: Arc<Mutex<rad::ipc::TimeoutPolicy>>,
    ) -> Result<String, rad::error::UnifiedError> {
        Ok("mock_stream_id".to_string())
    }
}

fn write_skill(workspace: &std::path::Path, name: &str, content: &str) {
    let dir = workspace.join(".agents/skills").join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), content).unwrap();
}

/// `get_tools` discovery is exercised directly against a standalone
/// `WasmRuntime` (no full `Orchestrator` needed — it only touches
/// `ListDir`/`FileRead`, which just need `ctx.sandbox`).
fn setup_tool_provider_runtime(
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
) -> WasmRuntime {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        ..Default::default()
    };

    let sandbox = Arc::new(FsSandbox::new(
        workspace.to_path_buf(),
        snapshots.to_path_buf(),
        perms.fs_read_allow.clone(),
        perms.fs_write_allow.clone(),
    ));
    let process_manager = Arc::new(ProcessManager::new());
    let dag = Arc::new(Mutex::new(Dag::new()));
    let active_processes = Arc::new(Mutex::new(HashMap::new()));
    let network = Arc::new(MockNetwork);

    let wasm_path = "target/wasm32-wasip2/debug/skill_tool_provider.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag });
    let (event_tx, _event_rx) = std::sync::mpsc::channel();

    WasmRuntime::new(
        "skill-tool-provider".to_string(),
        std::path::Path::new(wasm_path),
        "tool-provider".to_string(),
        perms,
        sandbox as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network,
        active_processes,
        event_tx,
        None,
        false,
        15000,
    )
    .unwrap()
}

#[test]
fn test_get_tools_discovers_project_local_skill() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    write_skill(
        &workspace,
        "review-checklist",
        "---\ndescription: Runs the team's PR review checklist.\n---\n\nCheck for tests and docs.",
    );

    let mut runtime = setup_tool_provider_runtime(&workspace, &snapshots);
    let tools_json = runtime.get_tools().unwrap();
    let tools: serde_json::Value = serde_json::from_str(&tools_json).unwrap();
    let tools = tools.as_array().unwrap();

    assert_eq!(tools.len(), 1);
    let function = &tools[0]["function"];
    assert_eq!(function["name"], "review-checklist");
    assert_eq!(
        function["description"],
        "Runs the team's PR review checklist."
    );
}

#[test]
fn test_get_tools_skips_skill_missing_description() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    write_skill(
        &workspace,
        "broken",
        "---\nmode: inline\n---\n\nNo description.",
    );

    let mut runtime = setup_tool_provider_runtime(&workspace, &snapshots);
    let tools_json = runtime.get_tools().unwrap();
    let tools: serde_json::Value = serde_json::from_str(&tools_json).unwrap();

    assert_eq!(tools.as_array().unwrap().len(), 0);
}

// --- Execution tests: real production path (full Orchestrator + mocked
// LLM + rad-orchestrator + skill-tool-provider), matching
// `tool_loop_tests.rs`/`hitl_tests.rs`. `execute_tool`'s result only
// becomes readable once rad-orchestrator's own `execute_tool_sync`
// resolves the transferred `ExecutionHandle` — there is no shortcut
// through a standalone `WasmRuntime` for this half.
