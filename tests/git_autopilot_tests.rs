use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

fn run_mock_http_server(addr: &str) -> std::thread::JoinHandle<()> {
    let addr_str = addr.to_string();
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        println!("[MOCK SERVER] Listening on {addr_str}");
        if let Ok((mut stream, _)) = listener.accept() {
            println!("[MOCK SERVER] Accepted connection!");
            let mut buf = [0; 4096];
            let n = std::io::Read::read(&mut stream, &mut buf).unwrap_or(0);
            println!(
                "[MOCK SERVER] Request content:\n{}",
                String::from_utf8_lossy(&buf[..n])
            );

            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, headers.as_bytes());

            let resp = "data: {\"choices\":[{\"delta\":{\"content\":\"Task complete.\"}}]}\n\ndata: [DONE]\n\n";
            let _ = std::io::Write::write_all(&mut stream, resp.as_bytes());
            let _ = std::io::Write::flush(&mut stream);
            let _ = stream.shutdown(std::net::Shutdown::Write);
            println!("[MOCK SERVER] Finished sending mock stream.");
        }
    })
}

fn init_git_repo(path: &Path) {
    let run = |args: &[&str]| {
        Command::new("git")
            .current_dir(path)
            .args(args)
            .output()
            .unwrap();
    };
    run(&["init"]);
    run(&["config", "user.name", "Autopilot Test"]);
    run(&["config", "user.email", "autopilot@example.com"]);
    fs::write(path.join("initial.txt"), "stable state").unwrap();
    run(&["add", "."]);
    run(&["commit", "-m", "initial commit"]);
}

fn setup_autopilot_orchestrator(
    workspace: &Path,
    snapshots: &Path,
    verify_cmd: Option<String>,
    port: u16,
) -> Arc<Orchestrator> {
    let mut config = rad::config::Config::default();
    config.core = rad::config::CoreConfig {
        workspace: workspace.to_string_lossy().to_string(),
        snapshot: snapshots.to_string_lossy().to_string(),
        log: workspace.join("logs").to_string_lossy().to_string(),
        hitl_enabled: false,
        verification_command: verify_cmd,
    };

    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            ..Default::default()
        }),
        network: Some(rad::config::NetworkConfig {
            allow_network: true,
            allow_domains: vec!["127.0.0.1".to_string()],
        }),
        ..Default::default()
    };

    let conn_perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        network: Some(rad::config::NetworkConfig {
            allow_network: true,
            allow_domains: vec!["127.0.0.1".to_string()],
        }),
        ..Default::default()
    };

    config.extensions = vec![
        rad::config::ExtensionConfig {
            name: "openai-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/openai_orchestrator.wasm".to_string(),
            permissions: Some(perms),
            config: HashMap::new(),
        },
        rad::config::ExtensionConfig {
            name: "openai-connector".to_string(),
            enabled: true,
            role: "llm-connector".to_string(),
            source: "target/wasm32-wasip2/debug/openai_connector.wasm".to_string(),
            permissions: Some(conn_perms),
            config: HashMap::new(),
        },
    ];

    let dag = Arc::new(Mutex::new(Dag::new()));
    // Create initial node to start task from
    {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Initial").unwrap();
    }

    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
        std::env::set_var("RAD_YOLO", "true");
    }

    Arc::new(Orchestrator::new(
        config,
        "test_autopilot_session".to_string(),
        dag,
        None,
    ))
}

#[test]
fn test_autopilot_rollback_on_verification_failure() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    init_git_repo(&workspace);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let _server = run_mock_http_server(&format!("127.0.0.1:{port}"));

    // Setup orchestrator with a failing verification command
    let orchestrator =
        setup_autopilot_orchestrator(&workspace, &snapshots, Some("exit 1".to_string()), port);

    // Make some dirty modifications to the workspace before task starts
    fs::write(workspace.join("initial.txt"), "broken changes").unwrap();
    fs::write(workspace.join("broken.txt"), "dirty code").unwrap();

    // Trigger run_task
    let res = orchestrator.run_task("run".to_string());
    assert!(res.is_ok());

    // Wait for task execution thread to complete
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(5) {
        if !orchestrator.is_running() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Verify workspace is rolled back to "stable state" and dirty files are deleted
    let initial_content = fs::read_to_string(workspace.join("initial.txt")).unwrap();
    assert_eq!(initial_content, "stable state");
    assert!(!workspace.join("broken.txt").exists());
}

#[test]
fn test_autopilot_commit_on_verification_success() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    init_git_repo(&workspace);

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let _server = run_mock_http_server(&format!("127.0.0.1:{port}"));

    // Setup orchestrator with a successful verification command (e.g. exit 0)
    let orchestrator =
        setup_autopilot_orchestrator(&workspace, &snapshots, Some("exit 0".to_string()), port);

    // Make some dirty modifications to the workspace (like what the agent would do)
    fs::write(workspace.join("initial.txt"), "new improved state").unwrap();
    fs::write(workspace.join("new_feature.txt"), "working code").unwrap();

    // Trigger run_task
    let res = orchestrator.run_task("run".to_string());
    assert!(res.is_ok());

    // Wait for task execution thread to complete
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < std::time::Duration::from_secs(5) {
        if !orchestrator.is_running() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Verify workspace changes are preserved (since verification succeeded)
    let initial_content = fs::read_to_string(workspace.join("initial.txt")).unwrap();
    assert_eq!(initial_content, "new improved state");
    assert!(workspace.join("new_feature.txt").exists());

    // Verify git history contains the new autopilot commit
    let output = Command::new("git")
        .current_dir(&workspace)
        .args(["log", "-1", "--pretty=%s"])
        .output()
        .unwrap();
    let commit_msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(commit_msg, "rad-autopilot: checkpoint verification_passed");
}
