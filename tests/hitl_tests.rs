use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::RasCoreEvent;
use rad::process::ProcessManager;
use rad::wasm::WasmRuntime;

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct MockNetwork {
    responses: Arc<Mutex<Vec<Vec<String>>>>,
}

impl rad::subsystems::NetworkSubsystem for MockNetwork {
    fn open_http_stream(
        &self,
        _url: &str,
        _headers: HashMap<String, String>,
        _body: &str,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
        _llm_timeout_policy: Arc<Mutex<rad::ipc::TimeoutPolicy>>,
    ) -> Result<String, String> {
        let mut guard = self.responses.lock().unwrap();
        if let Some(chunks) = guard.pop() {
            let tx = event_tx.clone();
            std::thread::spawn(move || {
                for chunk in chunks {
                    let _ = tx.send(RasCoreEvent::HttpChunkReceived { chunk });
                    std::thread::sleep(Duration::from_millis(10));
                }
            });
        }
        Ok("mock_stream_id".to_string())
    }
}

fn setup_runtime(
    responses: Vec<Vec<String>>,
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
    hitl_enabled: bool,
) -> (WasmRuntime, std::sync::mpsc::Receiver<RasCoreEvent>, Arc<Mutex<Dag>>) {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec![],
        }),
        network: Some(rad::config::NetworkConfig {
            allow_network: true,
            allow_domains: vec!["127.0.0.1".to_string()],
        }),
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

    let network = Arc::new(MockNetwork {
        responses: Arc::new(Mutex::new(responses)),
    });

    let wasm_path = "target/wasm32-wasip2/debug/openai_orchestrator.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let (event_tx, event_rx) = std::sync::mpsc::channel();

    let runtime = WasmRuntime::new(
        "test-extension".to_string(),
        std::path::Path::new(wasm_path),
        perms,
        sandbox as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network,
        active_processes,
        event_tx,
        None,
        hitl_enabled,
    )
    .unwrap();

    (runtime, event_rx, dag)
}

#[test]
fn test_hitl_approval_flow() {
    // === Case 1: Approval Granted ===
    unsafe {
        std::env::set_var("RAD_YOLO", "false");
        std::env::set_var("RAD_TEST_APPROVE", "y");
    }

    let temp_dir_granted = tempfile::tempdir().unwrap();
    let workspace_granted = temp_dir_granted.path().join("workspace");
    let snapshots_granted = temp_dir_granted.path().join("snapshots");
    fs::create_dir_all(&workspace_granted).unwrap();
    fs::create_dir_all(&snapshots_granted).unwrap();

    let turn2_granted = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1_granted = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_x\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"approved\\\\\\\" > test_hitl.txt\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    let (mut runtime_granted, event_rx_granted, _dag) = setup_runtime(
        vec![turn2_granted, turn1_granted],
        &workspace_granted,
        &snapshots_granted,
        true,
    );

    runtime_granted.on_event(&RasCoreEvent::HumanInputReceived {
        text: "start".to_string(),
    }).unwrap();

    let start_time = Instant::now();
    let mut completed = false;
    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = event_rx_granted.recv_timeout(Duration::from_millis(50)) {
            runtime_granted.on_event(&event).unwrap();
            if matches!(event, RasCoreEvent::TaskCompleted) {
                completed = true;
                break;
            }
        }
    }

    assert!(completed, "Task did not complete");

    let path_granted = workspace_granted.join("test_hitl.txt");
    assert!(path_granted.exists(), "File should exist because tool execution was approved");
    let content = fs::read_to_string(path_granted).unwrap();
    assert_eq!(content.trim(), "approved");

    // === Case 2: Approval Rejected ===
    unsafe {
        std::env::set_var("RAD_YOLO", "false");
        std::env::set_var("RAD_TEST_APPROVE", "n");
    }

    let temp_dir_rejected = tempfile::tempdir().unwrap();
    let workspace_rejected = temp_dir_rejected.path().join("workspace");
    let snapshots_rejected = temp_dir_rejected.path().join("snapshots");
    fs::create_dir_all(&workspace_rejected).unwrap();
    fs::create_dir_all(&snapshots_rejected).unwrap();

    let turn2_rejected = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Understood, it was rejected.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1_rejected = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_y\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"rejected\\\\\\\" > test_hitl.txt\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    let (mut runtime_rejected, event_rx_rejected, dag_rejected) = setup_runtime(
        vec![turn2_rejected, turn1_rejected],
        &workspace_rejected,
        &snapshots_rejected,
        true,
    );

    runtime_rejected.on_event(&RasCoreEvent::HumanInputReceived {
        text: "start".to_string(),
    }).unwrap();

    let start_time_rejected = Instant::now();
    let mut completed_rejected = false;
    while start_time_rejected.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = event_rx_rejected.recv_timeout(Duration::from_millis(50)) {
            runtime_rejected.on_event(&event).unwrap();
            if matches!(event, RasCoreEvent::TaskCompleted) {
                completed_rejected = true;
                break;
            }
        }
    }

    assert!(completed_rejected, "Task did not complete");

    let path_rejected = workspace_rejected.join("test_hitl.txt");
    assert!(!path_rejected.exists(), "File should NOT exist because tool execution was rejected");

    let dag_guard = dag_rejected.lock().unwrap();
    let mut found_rejection = false;
    for node in dag_guard.nodes.values() {
        if node.text.contains("User rejected execution of tool") {
            found_rejection = true;
            break;
        }
    }
    assert!(found_rejection, "Rejection message must be saved in the DAG history");

    // Clean up
    unsafe {
        std::env::remove_var("RAD_YOLO");
        std::env::remove_var("RAD_TEST_APPROVE");
    }
}

#[test]
fn test_yolo_mode_auto_approval() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn2 = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1 = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_yolo\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"yolo\\\\\\\" > test_yolo.txt\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    // hitl_enabled = false (YOLO mode)
    let (mut runtime, event_rx, _dag) = setup_runtime(
        vec![turn2, turn1],
        &workspace,
        &snapshots,
        false,
    );

    runtime.on_event(&RasCoreEvent::HumanInputReceived {
        text: "start".to_string(),
    }).unwrap();

    let start_time = Instant::now();
    let mut completed = false;
    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
            runtime.on_event(&event).unwrap();
            if matches!(event, RasCoreEvent::TaskCompleted) {
                completed = true;
                break;
            }
        }
    }

    assert!(completed, "Task did not complete in YOLO mode");

    let path = workspace.join("test_yolo.txt");
    assert!(path.exists(), "File should exist because tool execution was auto-approved in YOLO mode");
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content.trim(), "yolo");
}
