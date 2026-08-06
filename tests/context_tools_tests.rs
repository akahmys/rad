// End-to-end coverage for `context-tools` through the real WASM component
// boundary. Before AWU 915's follow-up (unifying `context-tools` onto the
// shared `rad.wit` types), no integration test ever loaded `context-tools`
// as an extension at all — `ext/context-tools`'s own unit tests only
// exercise the guest-side logic natively, never through
// `WasmRuntime::call_extension_method` / the real host-rpc bridge. These
// tests specifically guard against a regression in that bridge (WIT
// resolution, permission checks, the `GetRepoMap` delegation added in the
// same change).
use rad::config::{ExecutionConfig, PermissionConfig};
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

fn setup_context_tools_runtime(
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
) -> WasmRuntime {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            ..Default::default()
        }),
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
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let (event_tx, _event_rx) = std::sync::mpsc::channel();

    WasmRuntime::new(
        "context-tools".to_string(),
        std::path::Path::new("target/wasm32-wasip2/debug/context_tools.wasm"),
        "context-tools".to_string(),
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
fn test_context_tools_optimize_via_call_extension_method() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let mut runtime = setup_context_tools_runtime(&workspace, &snapshots);

    let req = serde_json::json!({
        "messages": [
            {"node_id": "0", "role": "user", "content": "goal"},
            {"node_id": "1", "role": "assistant", "content": "reply"},
        ],
        "max_history": 1,
        "max_content_chars": null,
    });
    let res = runtime
        .call_extension_method("optimize", &req.to_string())
        .unwrap();
    let val: serde_json::Value = serde_json::from_str(&res).unwrap();
    let optimized = val["optimized_messages"].as_array().unwrap();
    assert_eq!(optimized.len(), 1);
    assert!(val["summary"].as_str().unwrap().contains("Windowed"));
}

#[test]
fn test_context_tools_get_repo_map_uses_real_semantic_map() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    fs::write(
        workspace.join("main.rs"),
        "pub struct User { name: String }\nimpl User {\n    pub fn get_name(&self) -> &str { &self.name }\n}\n",
    )
    .unwrap();

    let mut runtime = setup_context_tools_runtime(&workspace, &snapshots);

    let res = runtime.call_extension_method("get-repo-map", "").unwrap();
    // context-tools now delegates to the real semantic (tree-sitter) repo
    // map via the shared `GetRepoMap` RPC instead of shelling out to
    // `tree -L 2` through its old bespoke raw-shell command type.
    assert!(res.contains("File: main.rs"));
    assert!(res.contains("pub struct User"));
}
