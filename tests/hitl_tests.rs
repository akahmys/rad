use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Both tests below set process-global `RAD_YOLO`/`RAD_TEST_APPROVE` env
// vars, which `cargo test`'s default parallel execution would otherwise
// race between threads in this same binary (same pattern as
// `llm_command_tests.rs`'s `TEST_MUTEX`).
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

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
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}

/// Registers rad-orchestrator + llm-connector + mcp-tool-provider (in its
/// self-contained `RAD_TEST_PORT` test-tool mode) and drives a task through
/// the real `Orchestrator::run_task` — the same shape as
/// `self_healing_core_auto_tests.rs`. A hand-rolled single-`WasmRuntime`
/// setup can't exercise a real tool call at all: there is no host-side
/// built-in-tool fallback left (removed as dead code — see PLANS.md), so
/// "bash"/"execute" must resolve to an actual registered tool provider,
/// which requires the runtimes to be wired through a real `Orchestrator`.
fn run_hitl_task(
    turn2: String,
    turn1: String,
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
    hitl_enabled: bool,
) -> Arc<Mutex<Dag>> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let responses = Arc::new(Mutex::new(vec![turn2, turn1]));
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
    }

    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: workspace.join("../logs").to_string_lossy().to_string(),
        hitl_enabled,
        verification_command: None,
        ..Default::default()
    };

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

    config.extensions = vec![
        rad::config::ExtensionConfig {
            name: "rad-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        rad::config::ExtensionConfig {
            name: "llm-connector".to_string(),
            enabled: true,
            role: "llm-connector".to_string(),
            source: "target/wasm32-wasip2/debug/llm_connector.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        rad::config::ExtensionConfig {
            name: "mcp-tool-provider".to_string(),
            enabled: true,
            role: "tool-provider".to_string(),
            source: "target/wasm32-wasip2/debug/mcp_tool_provider.wasm".to_string(),
            permissions: Some(perms),
            config: HashMap::new(),
        },
    ];

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
        n0
    };

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "test_session".to_string(),
        dag.clone(),
        None,
    ));

    let run_res = orchestrator.run_task("start".to_string());
    assert!(run_res.is_ok(), "Task spawning failed");

    // A loaded CI runner instantiating Wasm components is far slower than a warm
    // local machine. The assertion matters as much as the budget: without it the
    // loop falls through on timeout and a *later* assertion fails instead,
    // reporting a half-finished run as a logic error.
    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(30) {
        if !orchestrator.is_running() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        !orchestrator.is_running(),
        "task did not finish within the wait budget; assertions below would \
         report a half-completed run as a logic failure"
    );

    dag
}

#[test]
fn test_hitl_approval_flow() {
    let _lock = TEST_MUTEX.lock().unwrap();

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

    let turn2_granted =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n\ndata: [DONE]\n\n"
            .to_string();
    // `execute` (not `bash`) matches the tool `mcp-tool-provider` actually
    // advertises in `RAD_TEST_PORT` test mode.
    let turn1_granted = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_x\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"approved\\\\\\\" > test_hitl.txt\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();

    run_hitl_task(
        turn2_granted,
        turn1_granted,
        &workspace_granted,
        &snapshots_granted,
        true,
    );

    let path_granted = workspace_granted.join("test_hitl.txt");
    assert!(
        path_granted.exists(),
        "File should exist because tool execution was approved"
    );
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

    let turn2_rejected = "data: {\"choices\":[{\"delta\":{\"content\":\"Understood, it was rejected.\"}}]}\n\ndata: [DONE]\n\n".to_string();
    let turn1_rejected = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_y\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"rejected\\\\\\\" > test_hitl.txt\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();

    let dag_rejected = run_hitl_task(
        turn2_rejected,
        turn1_rejected,
        &workspace_rejected,
        &snapshots_rejected,
        true,
    );

    let path_rejected = workspace_rejected.join("test_hitl.txt");
    assert!(
        !path_rejected.exists(),
        "File should NOT exist because tool execution was rejected"
    );

    let dag_guard = dag_rejected.lock();
    let mut found_rejection = false;
    for node in dag_guard.nodes.values() {
        if node.text.contains("User rejected execution of tool") {
            found_rejection = true;
            break;
        }
    }
    assert!(
        found_rejection,
        "Rejection message must be saved in the DAG history"
    );
    drop(dag_guard);

    // Clean up
    unsafe {
        std::env::remove_var("RAD_YOLO");
        std::env::remove_var("RAD_TEST_APPROVE");
    }
}

#[test]
fn test_yolo_mode_auto_approval() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn2 =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Task finished.\"}}]}\n\ndata: [DONE]\n\n"
            .to_string();
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_yolo\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo \\\\\\\"yolo\\\\\\\" > test_yolo.txt\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();

    // hitl_enabled = false (YOLO mode)
    run_hitl_task(turn2, turn1, &workspace, &snapshots, false);

    let path = workspace.join("test_yolo.txt");
    assert!(
        path.exists(),
        "File should exist because tool execution was auto-approved in YOLO mode"
    );
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content.trim(), "yolo");
}
