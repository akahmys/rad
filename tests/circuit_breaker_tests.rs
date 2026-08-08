//! Verifies the consecutive-same-tool-failure circuit breaker in
//! `rad-orchestrator`'s `execute_pending_calls`. Failing tool calls are
//! simulated by having the mocked LLM invoke `mcp-tool-provider`'s
//! `RAD_TEST_PORT`-mode `execute` tool with a command that echoes an
//! `Error:`-prefixed string — the same prefix convention real MCP tool
//! failures are normalized to (see `mcp-tool-provider`'s `execute_tool`).
use rad::config::{ExecutionConfig, ExtensionConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn run_mock_http_server(
    addr: &str,
    responses: Arc<Mutex<Vec<String>>>,
) -> std::thread::JoinHandle<()> {
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut buf = [0; 4096];
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
            std::thread::sleep(Duration::from_millis(50));
        }
    })
}

/// One SSE turn calling `execute` with `command`, using `call_id` so each
/// turn's tool-call id is distinct.
fn tool_call_turn(call_id: &str, command: &str) -> String {
    let escaped = command.replace('\\', "\\\\").replace('"', "\\\"");
    let args = format!("{{\\\"command\\\":\\\"{escaped}\\\"}}");
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"tool_calls\":[{{\"index\":0,\"id\":\"{call_id}\",\"type\":\"function\",\"function\":{{\"name\":\"execute\",\"arguments\":\"{args}\"}}}}]}}}}]}}\n\ndata: [DONE]\n\n"
    )
}

fn text_turn(content: &str) -> String {
    format!(
        "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{content}\"}}}}]}}\n\ndata: [DONE]\n\n"
    )
}

/// Runs one task to completion against the mocked LLM. `turns` is in
/// serve-order; the mock server pops from the end, so it's reversed here.
/// Returns the DAG plus the still-unconsumed responses, so a test can
/// assert the breaker stopped *before* the model got another turn.
fn run_task(turns: Vec<String>) -> (Arc<Mutex<Dag>>, Arc<Mutex<Vec<String>>>) {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut reversed = turns;
    reversed.reverse();
    let responses = Arc::new(Mutex::new(reversed));
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"), responses.clone());
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
    }

    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: temp_dir.path().join("logs").to_string_lossy().to_string(),
        hitl_enabled: false,
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

    config.extensions = vec![ExtensionConfig {
        name: "rad-orchestrator".to_string(),
        enabled: true,
        role: "orchestrator".to_string(),
        source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
        permissions: Some(perms.clone()),
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

    let dag = Arc::new(Mutex::new(Dag::new()));
    {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
    }

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "test_breaker_session".to_string(),
        dag.clone(),
        None,
    ));

    assert!(orchestrator.run_task("start".to_string()).is_ok());

    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(20) {
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
    assert!(!orchestrator.is_running(), "task did not finish in time");

    (dag, responses)
}

fn count_failed_tool_results(dag: &Arc<Mutex<Dag>>) -> usize {
    dag.lock()
        .nodes
        .values()
        .filter(|n| n.node_type == "tool" && n.text.contains("simulated failure"))
        .count()
}

#[test]
fn test_breaker_stops_task_after_four_consecutive_same_tool_failures() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let fail = || tool_call_turn("c", "echo -n 'Error: simulated failure'");
    // 5 failing turns are offered, but the breaker (threshold 4) should
    // stop the task after the 4th — leaving the 5th unserved.
    let turns = vec![
        fail(),
        fail(),
        fail(),
        fail(),
        fail(),
        text_turn("should never be reached"),
    ];

    let (dag, remaining) = run_task(turns);

    assert_eq!(
        count_failed_tool_results(&dag),
        4,
        "breaker should have allowed exactly 4 failures before stopping"
    );
    assert_eq!(
        remaining.lock().len(),
        2,
        "the 5th failing turn and the trailing text turn should never have been requested"
    );
    assert!(
        !dag.lock()
            .nodes
            .values()
            .any(|n| n.text.contains("should never be reached")),
        "task should have stopped before consuming the final turn"
    );
}

#[test]
fn test_breaker_does_not_trip_below_threshold() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let fail = || tool_call_turn("c", "echo -n 'Error: simulated failure'");
    // 3 consecutive failures is under the threshold of 4, so the task
    // should run to its natural end.
    let turns = vec![fail(), fail(), fail(), text_turn("finished normally")];

    let (dag, remaining) = run_task(turns);

    assert_eq!(count_failed_tool_results(&dag), 3);
    assert!(
        remaining.lock().is_empty(),
        "all turns including the final text turn should have been served"
    );
    assert!(
        dag.lock()
            .nodes
            .values()
            .any(|n| n.text.contains("finished normally")),
        "task should have completed normally without tripping the breaker"
    );
}

#[test]
fn test_success_resets_the_consecutive_failure_streak() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let fail = || tool_call_turn("c", "echo -n 'Error: simulated failure'");
    let succeed = || tool_call_turn("c", "echo -n 'all good'");
    // 3 failures, then a success resetting the streak, then 3 more
    // failures — 6 failures total but never 4 *in a row*, so the task
    // should still reach its final turn.
    let turns = vec![
        fail(),
        fail(),
        fail(),
        succeed(),
        fail(),
        fail(),
        fail(),
        text_turn("survived the streak reset"),
    ];

    let (dag, remaining) = run_task(turns);

    assert_eq!(count_failed_tool_results(&dag), 6);
    assert!(
        remaining.lock().is_empty(),
        "a success in the middle should have reset the streak, letting every turn run"
    );
    assert!(
        dag.lock()
            .nodes
            .values()
            .any(|n| n.text.contains("survived the streak reset")),
        "task should have completed normally after the streak reset"
    );
}
