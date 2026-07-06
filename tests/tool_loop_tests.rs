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
use wasmtime::{Engine, Module};

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

#[test]
fn test_tool_loop_autonomy() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

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
        workspace.clone(),
        snapshots,
        perms.fs_read_allow.clone(),
        perms.fs_write_allow.clone(),
    ));
    let process_manager = Arc::new(ProcessManager::new());
    let dag = Arc::new(Mutex::new(Dag::new()));
    let active_processes = Arc::new(Mutex::new(HashMap::new()));

    // Define multi-turn responses (pop from the end, so order is reversed)
    let turn2 = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"I have written the file.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1 = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_w\",\"type\":\"function\",\"function\":{\"name\":\"file_write\",\"arguments\":\"{\\\"path\\\":\\\"test_out.txt\\\",\\\"content\\\":\\\"hello from LLM\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    let responses = Arc::new(Mutex::new(vec![turn2, turn1]));
    let network = Arc::new(MockNetwork { responses });

    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    let engine = Engine::new(&config).unwrap();
    
    let wasm_path = "target/wasm32-unknown-unknown/debug/openai_orchestrator.wasm";
    let module = Module::from_file(&engine, wasm_path).unwrap();

    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    
    let mut runtime = WasmRuntime::new_with_module(
        "test-extension".to_string(),
        &module,
        perms,
        sandbox.clone() as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager.clone() as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network,
        active_processes.clone(),
        event_tx,
        None,
    )
    .unwrap();

    // 1. Send human input to start the loop
    runtime.on_event(&RasCoreEvent::HumanInputReceived {
        text: "start".to_string(),
    }).unwrap();

    // 2. Poll the events and forward them back to wasm to continue execution loop
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

    assert!(completed, "Task did not complete within timeout");

    let path = workspace.join("test_out.txt");
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, "hello from LLM");
}

#[test]
fn test_context_recovery_with_tool_execution() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

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
        workspace.clone(),
        snapshots,
        perms.fs_read_allow.clone(),
        perms.fs_write_allow.clone(),
    ));
    let process_manager = Arc::new(ProcessManager::new());
    
    // Shared DAG to simulate context recovery
    let dag = Arc::new(Mutex::new(Dag::new()));
    let active_processes = Arc::new(Mutex::new(HashMap::new()));

    // Turn 1: request file write
    let turn1 = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_w1\",\"type\":\"function\",\"function\":{\"name\":\"file_write\",\"arguments\":\"{\\\"path\\\":\\\"test_rec.txt\\\",\\\"content\\\":\\\"first write\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let responses1 = Arc::new(Mutex::new(vec![turn1]));
    let network1 = Arc::new(MockNetwork { responses: responses1 });

    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    let engine = Engine::new(&config).unwrap();
    let wasm_path = "target/wasm32-unknown-unknown/debug/openai_orchestrator.wasm";
    let module = Module::from_file(&engine, wasm_path).unwrap();

    // 1. First execution session
    {
        let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let mut runtime = WasmRuntime::new_with_module(
            "test-extension".to_string(),
            &module,
            perms.clone(),
            sandbox.clone() as Arc<dyn rad::subsystems::FsSubsystem>,
            process_manager.clone() as Arc<dyn rad::subsystems::ProcessSubsystem>,
            dag_subsystem,
            network1,
            active_processes.clone(),
            event_tx,
            None,
        )
        .unwrap();

        runtime.on_event(&RasCoreEvent::HumanInputReceived {
            text: "start session 1".to_string(),
        }).unwrap();

        // Process events for 2 seconds to allow tool execution
        let start_time = Instant::now();
        while start_time.elapsed() < Duration::from_secs(2) {
            if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
                runtime.on_event(&event).unwrap();
            }
        }
    }

    let path = workspace.join("test_rec.txt");
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "first write");

    // 2. Second session (context recovery from the same DAG instance)
    let turn2 = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Completed task with context!\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let responses2 = Arc::new(Mutex::new(vec![turn2]));
    let network2 = Arc::new(MockNetwork { responses: responses2 });

    {
        let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let mut runtime = WasmRuntime::new_with_module(
            "test-extension".to_string(),
            &module,
            perms,
            sandbox.clone() as Arc<dyn rad::subsystems::FsSubsystem>,
            process_manager.clone() as Arc<dyn rad::subsystems::ProcessSubsystem>,
            dag_subsystem,
            network2,
            active_processes,
            event_tx,
            None,
        )
        .unwrap();

        runtime.on_event(&RasCoreEvent::HumanInputReceived {
            text: "continue session 2".to_string(),
        }).unwrap();

        let start_time = Instant::now();
        let mut completed = false;
        while start_time.elapsed() < Duration::from_secs(3) {
            if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
                runtime.on_event(&event).unwrap();
                if matches!(event, RasCoreEvent::TaskCompleted) {
                    completed = true;
                    break;
                }
            }
        }
        assert!(completed, "Task did not complete in session 2");
    }

    let dag_guard = dag.lock().unwrap();
    assert!(dag_guard.nodes.values().any(|n| n.text.contains("first write")));
}
