// Regression test for a bug found during real-world dogfeeding: `WasmRuntime`
// snapshots the host process's environment once at Wasm instance creation
// (`WasiCtxBuilder::inherit_env()`), so a *later* `std::env::set_var` on the
// host is invisible to an already-running instance. Since Phase 47-3 made
// extension loading eager (at startup, before any task runs), `llm-connector`
// used to read its base_url/api_key from env vars that were only set once a
// task actually started — so the very first task after boot always failed
// with "No LLM endpoint configured", only succeeding after a self-healing
// respawn recreated the instance. The fix (this session) has the host
// resolve base_url/api_key from config and pass them as explicit call
// arguments on every `generate_stream` invocation instead — this test
// exercises the exact eager-loading order and a *real* (non-`RAD_TEST_PORT`)
// `llm.endpoints` config to prove the first attempt now succeeds outright.
use rad::config::{
    Config, CoreConfig, ExtensionConfig, LlmConfig, LlmEndpointProfile, PermissionConfig,
};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn run_mock_http_server(addr: &str) -> std::thread::JoinHandle<()> {
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);

            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, headers.as_bytes());
            let resp = "data: {\"choices\":[{\"delta\":{\"content\":\"Task complete.\"}}]}\n\ndata: [DONE]\n\n";
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
            let _ = std::io::Write::flush(&mut stream);
            let _ = stream.shutdown(std::net::Shutdown::Write);
        }
    })
}

#[test]
fn test_first_task_after_eager_load_succeeds_with_real_base_url_config() {
    // Deliberately does NOT set RAD_TEST_PORT — that env var is a separate
    // test-infrastructure bypass unrelated to this bug. This test exercises
    // the real `llm.endpoints[active].base_url` resolution path a real user
    // hits.
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"));

    unsafe {
        std::env::set_var("RAD_YOLO", "true");
    }

    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        network: Some(rad::config::NetworkConfig {
            allow_network: true,
            allow_domains: vec!["127.0.0.1".to_string()],
        }),
        ..Default::default()
    };

    let mut endpoints = HashMap::new();
    endpoints.insert(
        "local".to_string(),
        LlmEndpointProfile {
            base_url: format!("http://127.0.0.1:{port}"),
            api_key: None,
            model: Some("test-model".to_string()),
            context_length: None,
            dialect: None,
        },
    );

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp_dir.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
        llm: LlmConfig {
            active: Some("local".to_string()),
            endpoints,
        },
        extensions: vec![
            ExtensionConfig {
                name: "rad-orchestrator".to_string(),
                source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
                enabled: true,
                role: "orchestrator".to_string(),
                permissions: Some(perms.clone()),
                config: HashMap::new(),
            },
            ExtensionConfig {
                name: "llm-connector".to_string(),
                source: "target/wasm32-wasip2/debug/llm_connector.wasm".to_string(),
                enabled: true,
                role: "llm-connector".to_string(),
                permissions: Some(perms),
                config: HashMap::new(),
            },
        ],
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
        n0
    };

    let orchestrator = Arc::new(Orchestrator::new(
        config,
        "test_session".to_string(),
        dag.clone(),
        None,
    ));

    // Mirrors main.rs: eagerly initialize all Wasm runtimes (llm-connector
    // included) *before* any task runs — the exact ordering that triggered
    // the stale-env-snapshot bug.
    let (throwaway_tx, _throwaway_rx) = std::sync::mpsc::channel();
    orchestrator.get_or_init_runtimes(&throwaway_tx).unwrap();

    let run_res = orchestrator.run_task("say hello".to_string());
    assert!(run_res.is_ok(), "Task spawning failed");

    // A loaded CI runner instantiating Wasm components is far slower than a warm
    // local machine. The assertion matters as much as the budget: without it the
    // loop falls through on timeout and a *later* assertion fails instead,
    // reporting a half-finished run as a logic error.
    let start_time = Instant::now();
    let mut completed = false;
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
    assert!(completed, "Task timed out");

    let dag_guard = dag.lock();
    let found_error = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("No LLM endpoint configured"));
    assert!(
        !found_error,
        "First task after eager load should not hit the stale-env-snapshot bug: {:?}",
        dag_guard
            .nodes
            .values()
            .map(|n| &n.text)
            .collect::<Vec<_>>()
    );

    let found_completion = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Task complete."));
    assert!(
        found_completion,
        "Expected the mock LLM response to reach the DAG"
    );
}
