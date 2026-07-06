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
    )
    .unwrap();

    (runtime, event_rx, dag)
}

#[test]
fn test_security_verification_hook_rejection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // 1. First turn: LLM requests writing to a blocked file "blocked.txt"
    // 2. Second turn: LLM requests executing a blocked command "blocked_command"
    let turn2 = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1 = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"file_write\",\"arguments\":\"{\\\"path\\\":\\\"blocked.txt\\\",\\\"content\\\":\\\"dangerous data\\\"}\"}},{\"index\":1,\"id\":\"call_2\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"blocked_command\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    let (mut runtime, event_rx, dag) = setup_runtime(vec![turn2, turn1], &workspace, &snapshots);

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

    assert!(completed, "Task did not complete");

    // The blocked.txt file must NOT exist because it was rejected by the verification hook
    let path = workspace.join("blocked.txt");
    assert!(!path.exists(), "File blocked.txt should NOT exist");

    // The rejection errors must be saved in the DAG
    let dag_guard = dag.lock().unwrap();
    println!("=== DEBUG DAG NODES ===");
    for (id, node) in &dag_guard.nodes {
        println!("Node ID: {}, Type: {}, Text: {}", id, node.node_type, node.text);
    }
    println!("=======================");
    let mut found_fs_rejection = false;
    let mut found_proc_rejection = false;

    for node in dag_guard.nodes.values() {
        if node.text.contains("Operation rejected by security extension") {
            if node.text.contains("call_1") {
                found_fs_rejection = true;
            }
            if node.text.contains("call_2") {
                found_proc_rejection = true;
            }
        }
    }

    assert!(found_fs_rejection, "FS rejection message must be saved in the DAG history");
    assert!(found_proc_rejection, "Process rejection message must be saved in the DAG history");
}
