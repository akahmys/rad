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
use std::time::{Duration, Instant};

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
    allowed_mcp: Vec<String>,
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
) -> (WasmRuntime, std::sync::mpsc::Receiver<RasCoreEvent>) {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            ..Default::default()
        }),
        network: None,
        allowed_mcp_servers: allowed_mcp,
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

    let wasm_path = "target/wasm32-wasip2/debug/rad_orchestrator.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag });
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

    (runtime, event_rx)
}

#[test]
fn test_mcp_permission_denied() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // allowed_mcp_servers is empty
    let (mut runtime, _event_rx) = setup_runtime(vec![], &workspace, &snapshots);

    // Try to verify RPC for unauthorized server
    let req = rad::ipc::RasRpcRequest {
        id: Some("call_1".to_string()),
        command: RasRpcCommand::SpawnMcpServer {
            name: "unauthorized-mcp".to_string(),
            command: "cat".to_string(),
            args: vec![],
        },
    };
    let wit_cmd = rad::wasm::bindings::wit::RasRpcCommand::from(req.command);

    let state = runtime.store.data_mut();
    let res = rad::wasm::bindings::RadExtensionImports::host_rpc(state, wit_cmd);

    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .contains("MCP permission denied: server 'unauthorized-mcp' is not whitelisted")
    );
}

#[test]
fn test_mcp_echo_communication() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // Whitelist "echo-mcp"
    let (mut runtime, event_rx) =
        setup_runtime(vec!["echo-mcp".to_string()], &workspace, &snapshots);

    // 1. Spawn MCP Server (command: "cat" to act as echo server)
    let req_spawn = rad::ipc::RasRpcRequest {
        id: Some("call_spawn".to_string()),
        command: RasRpcCommand::SpawnMcpServer {
            name: "echo-mcp".to_string(),
            command: "cat".to_string(),
            args: vec![],
        },
    };
    let req_bytes = serde_json::to_vec(&req_spawn).unwrap();
    runtime.verify_rpc(&req_bytes).unwrap();

    // Perform execution directly via Host RPC mock verification
    let wit_cmd_spawn = rad::wasm::bindings::wit::RasRpcCommand::from(req_spawn.command);

    let state = runtime.store.data_mut();
    let res_spawn = rad::wasm::bindings::RadExtensionImports::host_rpc(state, wit_cmd_spawn);
    assert!(res_spawn.is_ok());

    // 2. Send Message to MCP
    let msg = "{\"jsonrpc\":\"2.0\",\"method\":\"ping\",\"id\":1}";
    let req_send = rad::ipc::RasRpcRequest {
        id: Some("call_send".to_string()),
        command: RasRpcCommand::SendMcpRequest {
            name: "echo-mcp".to_string(),
            message: msg.to_string(),
        },
    };
    let req_send_bytes = serde_json::to_vec(&req_send).unwrap();
    runtime.verify_rpc(&req_send_bytes).unwrap();

    let wit_cmd_send = rad::wasm::bindings::wit::RasRpcCommand::from(req_send.command);

    let state = runtime.store.data_mut();
    let res_send = rad::wasm::bindings::RadExtensionImports::host_rpc(state, wit_cmd_send);
    assert!(res_send.is_ok());

    // 3. Verify Event Received (McpResponse)
    let start_time = Instant::now();
    let mut received_response = None;
    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(RasCoreEvent::McpResponse { name, message, .. }) =
            event_rx.recv_timeout(Duration::from_millis(50))
        {
            assert_eq!(name, "echo-mcp");
            received_response = Some(message);
            break;
        }
    }

    assert!(
        received_response.is_some(),
        "Should have received McpResponse"
    );
    assert_eq!(received_response.unwrap(), msg);
}
