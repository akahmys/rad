use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

// Both tests below set the process-global `RAD_TEST_PORT` env var, which
// `cargo test`'s default parallel execution would otherwise race between
// threads in this same binary (same pattern as `llm_command_tests.rs`'s
// `TEST_MUTEX`).
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

fn perms() -> PermissionConfig {
    PermissionConfig {
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
    }
}

/// Registers rad-orchestrator, plus the `llm-openai` and `mcp` kernel
/// modules (the latter in its self-contained `RAD_TEST_PORT` test-tool mode) and drives one task
/// through the real `Orchestrator::run_task`, sharing `dag` with the
/// caller so multiple calls can simulate independent sessions recovering
/// the same conversation history (same shape as `hitl_tests.rs` and
/// `self_healing_core_auto_tests.rs`; there is no host-side built-in-tool
/// fallback left to lean on — see PLANS.md — so tool calls must resolve to
/// an actually-registered provider).
fn run_session(
    turns: Vec<String>,
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
    dag: Arc<Mutex<Dag>>,
    session_id: &str,
    instruction: &str,
) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let responses = Arc::new(Mutex::new(turns));
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
    }

    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: workspace.join("../logs").to_string_lossy().to_string(),
        hitl_enabled: false,
        verification_command: None,
        ..Default::default()
    };
    config.extensions = vec![rad::config::ExtensionConfig {
        name: "rad-orchestrator".to_string(),
        enabled: true,
        role: "orchestrator".to_string(),
        source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
        permissions: Some(perms()),
        config: HashMap::new(),
    }];
    // Tools come from the `mcp` kernel module, which under `RAD_TEST_PORT`
    // offers the synthetic read/write/execute set this suite drives. It was
    // `mcp-tool-provider` until AWU 965. `Orchestrator::new` boots whatever
    // `modules` declares, so there is nothing to wire up here.
    config.modules = vec![
        rad::config::ModuleConfig {
            name: "mcp".to_string(),
            source: "target/wasm32-wasip2/debug/mcp_module.wasm".to_string(),
            enabled: true,
            config: serde_json::Value::Null,
        },
        rad::config::ModuleConfig {
            name: "llm-openai".to_string(),
            source: "target/wasm32-wasip2/debug/llm_openai_module.wasm".to_string(),
            enabled: true,
            config: serde_json::Value::Null,
        },
    ];

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        session_id.to_string(),
        dag,
        None,
    ));

    let run_res = orchestrator.run_task(instruction.to_string());
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
}

#[test]
fn test_tool_loop_autonomy() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn2 =
        "data: {\"choices\":[{\"delta\":{\"content\":\"I have written the file.\"}}]}\n\ndata: [DONE]\n\n"
            .to_string();
    // `execute` (not `write`) — `mcp-tool-provider`'s `RAD_TEST_PORT` mode
    // `write` tool always writes the literal string "test", ignoring
    // `content`, so an `execute`-based shell command is used to get exact
    // file contents for the assertion below.
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_w\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo -n 'hello from LLM' > test_out.txt\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
        n0
    };

    run_session(
        vec![turn2, turn1],
        &workspace,
        &snapshots,
        dag,
        "test_autonomy_session",
        "start",
    );

    let path = workspace.join("test_out.txt");
    let content = fs::read_to_string(path).unwrap();
    assert_eq!(content, "hello from LLM");
}

#[test]
fn test_context_recovery_with_tool_execution() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
        n0
    };

    // 1. First session: request a file write, then let the task finish.
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_w1\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo -n 'first write' > test_rec.txt\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();
    let turn1b =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Wrote it.\"}}]}\n\ndata: [DONE]\n\n"
            .to_string();
    run_session(
        vec![turn1b, turn1],
        &workspace,
        &snapshots,
        dag.clone(),
        "test_recovery_session_1",
        "start session 1",
    );

    let path = workspace.join("test_rec.txt");
    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "first write");

    // 2. Second session, sharing the same `dag` (simulating recovery of
    // conversation history across a restart) — no further tool calls.
    let turn2 = "data: {\"choices\":[{\"delta\":{\"content\":\"Completed task with context!\"}}]}\n\ndata: [DONE]\n\n".to_string();
    run_session(
        vec![turn2],
        &workspace,
        &snapshots,
        dag.clone(),
        "test_recovery_session_2",
        "continue session 2",
    );

    let dag_guard = dag.lock();
    assert!(
        dag_guard
            .nodes
            .values()
            .any(|n| n.text.contains("first write"))
    );
}
