// Split out of `self_healing_tests.rs` to stay under the 300-line file
// limit — that file's remaining test (`test_wasm_panic_self_healing_and_rehydration`)
// drives a `WasmRuntime` directly with a `MockNetwork`, while this one
// drives a full `Orchestrator` against a real mock HTTP server; the two
// share no helpers, so the split is clean.
use rad::config::{ExecutionConfig, PermissionConfig};
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
                 data: [DONE]\n\n"
        .to_string();
    // `execute` (not `bash`) matches the tool `mcp-tool-provider` actually
    // advertises in `RAD_TEST_PORT` test mode — see the extensions list
    // below. There is no host-side "built-in tool" fallback for unmatched
    // tool names (removed as dead/unreachable code; see PLANS.md), so the
    // tool name here must be one a registered tool-provider really serves.
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[\
                 {\"index\":0,\"id\":\"call_panic\",\"type\":\"function\",\"function\":{\"name\":\"execute\",\"arguments\":\"{\\\"command\\\":\\\"echo CRASH_WASM; sleep 1\\\"}\"}}\
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
        hitl_enabled: false,
        verification_command: None,
        ..Default::default()
    };
    let wasm_path = "target/wasm32-wasip2/debug/rad_orchestrator.wasm";

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
            source: wasm_path.to_string(),
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
        // Real tool-provider so the "execute" tool call above resolves to
        // an actual provider instead of the (now removed) host-side
        // built-in-tool fallback. `RAD_TEST_PORT` (already set above for
        // the mock LLM server) also switches this extension into its
        // self-contained test-tool mode.
        rad::config::ExtensionConfig {
            name: "mcp-tool-provider".to_string(),
            enabled: true,
            role: "tool-provider".to_string(),
            source: "target/wasm32-wasip2/debug/mcp_tool_provider.wasm".to_string(),
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
    let mut completed = false;
    // 8s (not the original 5s): this test now also registers
    // mcp-tool-provider (needed once the host-side built-in-tool fallback
    // was removed — see PLANS.md), so the self-healing retry recompiles
    // one more Wasm component than before, occasionally pushing a cold
    // run past a tighter deadline under load.
    while start_time.elapsed() < Duration::from_secs(30) {
        if !orchestrator.is_running() {
            completed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        !orchestrator.is_running(),
        "task did not finish within the wait budget; assertions below would \
         report a half-completed run as a logic failure"
    );

    assert!(completed, "Core auto recovery task did not finish in time");

    let dag_guard = dag.lock();
    println!("DEBUG DAG NODES COUNT: {}", dag_guard.nodes.len());
    for (id, node) in &dag_guard.nodes {
        println!(
            "DEBUG NODE id={}, type={}, text='{}'",
            id, node.node_type, node.text
        );
    }

    let mut found_recovery_msg = false;
    for node in dag_guard.nodes.values() {
        if node.text.contains("Recovered and completed.") {
            found_recovery_msg = true;
        }
    }
    assert!(
        found_recovery_msg,
        "Rehydrated second turn message was not found in DAG"
    );
}
