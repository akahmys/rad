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
                    std::thread::sleep(Duration::from_millis(50));
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
    event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
) -> (WasmRuntime, Arc<Mutex<Dag>>) {
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

    (runtime, dag)
}

#[test]
fn test_wasm_panic_self_healing_and_rehydration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // turn1: Requests a process that prints "CRASH_WASM" which triggers the backdoor panic
    // turn2: Standard completion after self-healing and process exited rehydration
    let turn2 = vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Recovered and finished.\"}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];
    let turn1 = vec![
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_panic\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"echo CRASH_WASM; sleep 1\\\"}\"}}]}}]}\n".to_string(),
        "data: [DONE]\n".to_string(),
    ];

    // Using Orchestrator directly to test the self-healing recovery loop
    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    let wasm_path = "target/wasm32-wasip2/debug/openai_orchestrator.wasm";
    
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

    config.extensions = vec![rad::config::ExtensionConfig {
        name: "openai-orchestrator".to_string(),
        enabled: true,
        source: wasm_path.to_string(),
        permissions: Some(perms),
        config: HashMap::new(),
    }];

    let dag = Arc::new(Mutex::new(Dag::new()));
    
    // Create initial node to start task from
    let _initial_node = {
        let mut dag_guard = dag.lock().unwrap();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        
        let snapshot_dir = snapshots.join(&n0);
        fs::create_dir_all(snapshot_dir).unwrap();
        n0
    };

    let _orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(config, "test_session".to_string(), dag.clone(), None));

    // Setup network mocks using environment or custom server if needed.
    // However, to keep it self-contained without real network, we verify that the orchestrator's
    // run_task_internal handles errors gracefully.
    // We execute run_task directly and expect it to execute the task.
    // Because we mock the network stream at the Orchestrator level or tests setup,
    // let's verify that when Wasm fails, it recovers.

    // Let's test the recovery loop manually by spawning a panic in runtime.on_event
    // and ensuring Orchestrator reloads it.
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let (mut runtime, _dag_mock) = setup_runtime(vec![turn2.clone(), turn1], &workspace, &snapshots, event_tx.clone());

    // Initial event
    runtime.on_event(&RasCoreEvent::HumanInputReceived {
        text: "start".to_string(),
    }).unwrap();

    let start_time = Instant::now();
    let mut completed = false;
    let mut panic_occurred = false;
    let mut actual_pgid = None;

    while start_time.elapsed() < Duration::from_secs(5) {
        if let Ok(event) = event_rx.recv_timeout(Duration::from_millis(50)) {
            println!("DEBUG TEST EVENT: {:?}", event);
            match event {
                RasCoreEvent::ProcessStdout { pgid, .. } | RasCoreEvent::ProcessStderr { pgid, .. } => {
                    actual_pgid = Some(pgid);
                    println!("DEBUG CAPTURED PGID: {:?}", actual_pgid);
                }
                _ => {}
            }

            match runtime.on_event(&event) {
                Ok(_) => {
                    if matches!(event, RasCoreEvent::TaskCompleted) {
                        completed = true;
                        break;
                    }
                }
                Err(e) => {
                    println!("Simulated crash caught in test driver: {e}. Re-hydrating...");
                    panic_occurred = true;
                    // Self-healing: recreate runtime with turn2 response
                    let (new_runtime, _) = setup_runtime(vec![turn2.clone()], &workspace, &snapshots, event_tx.clone());
                    runtime = new_runtime;

                    // Rehydrate active process info using captured actual pgid
                    let active_calls = vec![rad_models::PendingToolCallInfo {
                        id: "call_panic".to_string(),
                        name: "spawn_bash_process".to_string(),
                        arguments: "{\"command\":\"echo CRASH_WASM; sleep 1\"}".to_string(),
                        pgid: actual_pgid,
                    }];
                    println!("DEBUG SEND REHYDRATE WITH: {:?}", active_calls);
                    runtime.on_event(&RasCoreEvent::Rehydrate { active_calls }).unwrap();
                }
            }
        }
    }

    assert!(panic_occurred, "Wasm panic should have been triggered and caught");
    assert!(completed, "Task should complete successfully after Wasm self-healing");
}

fn run_mock_http_server(addr: &str, responses: Arc<Mutex<Vec<String>>>) -> std::thread::JoinHandle<()> {
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);

            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, headers.as_bytes());

            let resp = {
                let mut guard = responses.lock().unwrap();
                guard.pop()
            };
            if let Some(chunks_str) = resp {
                let _ = std::io::Write::write_all(&mut stream, chunks_str.as_bytes());
            }
            let _ = std::io::Write::flush(&mut stream);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}

#[test]
fn test_core_auto_self_healing_integration() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn2 = "data: {\"choices\":[{\"delta\":{\"content\":\"Recovered and completed.\"}}]}\n\n\
                 data: [DONE]\n\n".to_string();
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[\
                 {\"index\":0,\"id\":\"call_panic\",\"type\":\"function\",\"function\":{\"name\":\"spawn_bash_process\",\"arguments\":\"{\\\"command\\\":\\\"echo CRASH_WASM; sleep 1\\\"}\"}}\
                 ]}}]}\n\n\
                 data: [DONE]\n\n".to_string();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let responses = Arc::new(Mutex::new(vec![turn2, turn1]));
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
        std::env::set_var("RAD_YOLO", "true");
    }

    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: temp_dir.path().join("logs").to_string_lossy().to_string(),
    };
    let wasm_path = "target/wasm32-wasip2/debug/openai_orchestrator.wasm";
    
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

    config.extensions = vec![rad::config::ExtensionConfig {
        name: "openai-orchestrator".to_string(),
        enabled: true,
        source: wasm_path.to_string(),
        permissions: Some(perms),
        config: HashMap::new(),
    }];

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock().unwrap();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        
        let snapshot_dir = snapshots.join(&n0);
        fs::create_dir_all(snapshot_dir).unwrap();
        n0
    };

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(config, "test_session".to_string(), dag.clone(), None));

    let run_res = orchestrator.run_task("start".to_string());
    assert!(run_res.is_ok(), "Task spawning failed");

    let start_time = Instant::now();
    let mut completed = false;
    while start_time.elapsed() < Duration::from_secs(5) {
        if !orchestrator.is_running() {
            completed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }



    assert!(completed, "Core auto recovery task did not finish in time");

    let dag_guard = dag.lock().unwrap();
    println!("DEBUG DAG NODES COUNT: {}", dag_guard.nodes.len());
    for (id, node) in &dag_guard.nodes {
        println!("DEBUG NODE id={}, type={}, text='{}'", id, node.node_type, node.text);
    }

    let mut found_recovery_msg = false;
    for node in dag_guard.nodes.values() {
        if node.text.contains("Recovered and completed.") {
            found_recovery_msg = true;
        }
    }
    assert!(found_recovery_msg, "Rehydrated second turn message was not found in DAG");
}
