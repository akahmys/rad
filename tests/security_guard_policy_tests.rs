// Split out of `multi_extension_tests.rs` to stay under the 300-line file
// limit — this file is specifically about `security-guard`'s config-driven
// blocklist policy (Phase 48), while `multi_extension_tests.rs` covers the
// broader multi-extension verification chain.
use parking_lot::Mutex;
use rad::config::{Config, CoreConfig, ExecutionConfig, ExtensionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
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

/// The blocklist policy is opt-in: an extension registered with no `config`
/// at all fetches an empty policy (`GetExtensionConfig` returns `{}`) and
/// blocks nothing, rather than falling back to any hardcoded literal. This
/// is the counterpart to `multi_extension_tests::test_multi_extension_isolated_roles`,
/// which proves the same wiring actually blocks when patterns ARE configured.
#[test]
fn test_security_guard_blocklist_is_opt_in_and_blocks_nothing_when_unconfigured() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn2 = "data: {\"choices\":[{\"delta\":{\"content\":\"Task completed safely.\"}}]}\n\n\
                 data: [DONE]\n\n"
        .to_string();
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[\
                 {\"index\":0,\"id\":\"call_write\",\"type\":\"function\",\"function\":{\"name\":\"write\",\"arguments\":\"{\\\"path\\\":\\\"blocked.txt\\\",\\\"content\\\":\\\"harmless content\\\"}\"}}\
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

    let mut config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: temp_dir.path().join("logs").to_string_lossy().to_string(),
            ..Default::default()
        },
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

    // Same shape as `test_multi_extension_isolated_roles`, but the
    // security-guard extension is registered with an empty `config` — no
    // blocklist patterns at all.
    config.extensions = vec![
        ExtensionConfig {
            name: "rad-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
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
    ];
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
    let _initial_node = {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
        let snapshot_dir = snapshots.join(&n0);
        fs::create_dir_all(snapshot_dir).unwrap();
        n0
    };

    let orchestrator = Arc::new(Orchestrator::new(
        config,
        "test_multi_role_unconfigured".to_string(),
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

    assert!(completed, "Orchestrator task timed out");

    // With no configured blocklist patterns, the write goes through.
    let path = workspace.join("blocked.txt");
    assert!(
        path.exists(),
        "File blocked.txt SHOULD exist when the security guard has no configured policy"
    );
}
