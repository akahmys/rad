use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::{RasCoreEvent, RasRpcCommand};
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

fn setup_runtime(
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
) -> (
    WasmRuntime,
    std::sync::mpsc::Receiver<RasCoreEvent>,
    Arc<Mutex<Dag>>,
) {
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

    let wasm_path = "target/wasm32-wasip2/debug/openai_orchestrator.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let runtime = WasmRuntime::new(
        "test-extension".to_string(),
        std::path::Path::new(wasm_path),
        "orchestrator".to_string(),
        perms,
        sandbox as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network,
        active_processes,
        event_tx,
        None,
        false,
    )
    .unwrap();

    (runtime, event_rx, dag)
}

#[test]
fn test_get_repo_map_via_rpc() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // Create a mock Rust file in the workspace
    let mock_file_path = workspace.join("main.rs");
    let mock_code = r#"
        pub struct User {
            name: String,
        }

        impl User {
            pub fn get_name(&self) -> &str {
                &self.name
            }
        }
    "#;
    fs::write(&mock_file_path, mock_code).unwrap();

    let (mut runtime, _event_rx, _dag) = setup_runtime(&workspace, &snapshots);

    // Call GetRepoMap via Host RPC
    let req = rad::ipc::RasRpcRequest {
        id: Some("call_repomap".to_string()),
        command: RasRpcCommand::GetRepoMap,
    };
    let wit_cmd = rad::wasm::bindings::wit::RasRpcCommand::from(req.command);
    let state = runtime.store.data_mut();
    let res = rad::wasm::bindings::RadExtensionImports::host_rpc(state, wit_cmd);

    assert!(res.is_ok());
    let res_json = res.unwrap();
    let repo_map_str: String = serde_json::from_str(&res_json).unwrap();

    assert!(repo_map_str.contains("File: main.rs"));
    assert!(repo_map_str.contains("pub struct User"));
    assert!(repo_map_str.contains("impl User"));
}

#[test]
fn test_dag_node_semantic_references() {
    let mut dag = Dag::new();
    let n0 = dag.create_node("", "user").unwrap();

    // Verify default value is None
    {
        let node = dag.nodes.get(&n0).unwrap();
        assert_eq!(node.semantic_references, None);
    }

    // Set semantic references
    let mock_refs = "File: main.rs\n  pub struct User".to_string();
    dag.set_node_semantic_references(&n0, Some(mock_refs.clone()))
        .unwrap();

    // Verify it was saved
    {
        let node = dag.nodes.get(&n0).unwrap();
        assert_eq!(node.semantic_references, Some(mock_refs));
    }
}
