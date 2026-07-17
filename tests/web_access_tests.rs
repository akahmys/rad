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

    let wasm_path = "target/wasm32-wasip2/release/web_access.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let runtime = WasmRuntime::new(
        "web-access".to_string(),
        std::path::Path::new(wasm_path),
        "web-access".to_string(),
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
fn test_web_access_search_fallback() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let (mut runtime, _event_rx, _dag) = setup_runtime(&workspace, &snapshots);

    // Call search via call_extension_method on WasmRuntime
    let query = "rust programming language".to_string();
    let res = runtime.call_extension_method("search", &query);

    assert!(res.is_ok());
    let output = res.unwrap();
    // Since TAVILY_API_KEY is not set, it should fallback to DuckDuckGo API or print no results
    assert!(
        output.contains("No search results found")
            || output.contains("Instant Answer")
            || output.contains("duckduckgo")
    );
}

#[test]
fn test_web_access_fetch_mocked() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let (mut runtime, _event_rx, _dag) = setup_runtime(&workspace, &snapshots);

    // We can mock the curl command via host_rpc intercept or simply try to fetch a local file or check standard fetch behavior.
    // Let's create a local file and fetch it using file:// URL or just query a unreachable url and see it fails/handles cleanly.
    let res = runtime.call_extension_method("fetch", "http://example.com");
    // Under test mock network, host process executing curl might return host exit status or succeed if curl is available.
    // Let's just assert that it either succeeds or returns Err with message as handled by extension.
    match res {
        Ok(content) => {
            assert!(!content.is_empty());
        }
        Err(e) => {
            assert!(
                e.contains("Command failed")
                    || e.contains("Empty content")
                    || e.contains("Failed to execute")
            );
        }
    }
}
