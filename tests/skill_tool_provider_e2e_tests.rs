// End-to-end skill invocation: drives a full Orchestrator against a mock LLM
// server. Split from `skill_tool_provider_tests.rs`, which covers `get_tools`
// discovery directly, to keep both files under the 300-line limit.
use rad::config::{ExecutionConfig, ExtensionConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn write_skill(workspace: &std::path::Path, name: &str, content: &str) {
    let dir = workspace.join(".agents/skills").join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("SKILL.md"), content).unwrap();
}

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

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn run_skill_task(
    turns: Vec<String>,
    workspace: &std::path::Path,
    snapshots: &std::path::Path,
) -> Arc<Mutex<Dag>> {
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
        ExtensionConfig {
            name: "rad-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        ExtensionConfig {
            name: "llm-connector".to_string(),
            enabled: true,
            role: "llm-connector".to_string(),
            source: "target/wasm32-wasip2/debug/llm_connector.wasm".to_string(),
            permissions: Some(perms.clone()),
            config: HashMap::new(),
        },
        ExtensionConfig {
            name: "skill-tool-provider".to_string(),
            enabled: true,
            role: "tool-provider".to_string(),
            source: "target/wasm32-wasip2/debug/skill_tool_provider.wasm".to_string(),
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
        "test_skill_session".to_string(),
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
fn test_execute_tool_returns_skill_body_with_args_substituted() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    write_skill(
        &workspace,
        "greeter",
        "---\ndescription: Greets someone by name.\n---\n\nSay hello to $ARGUMENTS warmly.",
    );

    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_g\",\"type\":\"function\",\"function\":{\"name\":\"greeter\",\"arguments\":\"{\\\"args\\\":\\\"Alice\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();
    let turn2 =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Done.\"}}]}\n\ndata: [DONE]\n\n".to_string();

    let dag = run_skill_task(vec![turn2, turn1], &workspace, &snapshots);

    let dag_guard = dag.lock();
    let found = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Say hello to Alice warmly."));
    assert!(
        found,
        "Expected a DAG node containing the skill's substituted body"
    );
}

#[test]
fn test_execute_tool_errors_for_unknown_skill() {
    let _lock = TEST_MUTEX.lock().unwrap();

    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_x\",\"type\":\"function\",\"function\":{\"name\":\"does-not-exist\",\"arguments\":\"{}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();
    let turn2 =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Got the error.\"}}]}\n\ndata: [DONE]\n\n"
            .to_string();

    let dag = run_skill_task(vec![turn2, turn1], &workspace, &snapshots);

    let dag_guard = dag.lock();
    let found_error = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Unknown skill") || n.text.contains("does-not-exist"));
    assert!(
        found_error,
        "Expected the unknown-skill error to surface as a tool result in the DAG"
    );
}
