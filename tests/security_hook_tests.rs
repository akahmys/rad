use rad::config::{Config, CoreConfig, ExecutionConfig, ExtensionConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn run_mock_http_server(
    addr: &str,
    responses: Arc<Mutex<Vec<String>>>,
) -> std::thread::JoinHandle<()> {
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0; 1024];
            let _ = std::io::Read::read(&mut stream, &mut buf);

            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, headers.as_bytes());

            let resp = {
                let mut guard = responses.lock();
                guard.pop()
            };
            if let Some(chunks_str) = resp {
                let _ = std::io::Write::write_all(&mut stream, chunks_str.as_bytes());
            }
            let _ = std::io::Write::flush(&mut stream);
            let _ = stream.shutdown(std::net::Shutdown::Write);
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}

#[test]
fn test_security_verification_hook_rejection() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    // Turn 2: LLM responds with a text message after rejection
    let turn2 = "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n\n\
                 data: [DONE]\n\n"
        .to_string();
    // Turn 1: LLM requests writing to a blocked file "blocked.txt"
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"write\",\"arguments\":\"{\\\"path\\\":\\\"blocked.txt\\\",\\\"content\\\":\\\"dangerous data\\\"}\"}}]}}]}\n\n\
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
        ..Default::default()
    };

    let mut config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp_dir.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        ..Default::default()
    };

    config.extensions = vec![
        ExtensionConfig {
            name: "openai-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/openai_orchestrator.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        ExtensionConfig {
            name: "security-guard".to_string(),
            enabled: true,
            role: "security".to_string(),
            source: "target/wasm32-wasip2/debug/security_guard.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        ExtensionConfig {
            name: "openai-connector".to_string(),
            enabled: true,
            role: "llm-connector".to_string(),
            source: "target/wasm32-wasip2/debug/openai_connector.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
    ];

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        let snapshot_dir = snapshots.join(&n0);
        fs::create_dir_all(snapshot_dir).unwrap();
        n0
    };

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "test_security_session".to_string(),
        dag.clone(),
        None,
    ));

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

    assert!(completed, "Task did not complete");

    // The blocked.txt file must NOT exist because it was rejected by the security guard
    let path = workspace.join("blocked.txt");
    assert!(!path.exists(), "File blocked.txt should NOT exist");

    // The rejection errors must be saved in the DAG
    let dag_guard = dag.lock();
    let mut found_fs_rejection = false;

    for node in dag_guard.nodes.values() {
        if node
            .text
            .contains("Operation rejected by security extension")
            && node.text.contains("call_1")
        {
            found_fs_rejection = true;
        }
    }

    assert!(
        found_fs_rejection,
        "FS rejection message must be saved in the DAG history"
    );
}
