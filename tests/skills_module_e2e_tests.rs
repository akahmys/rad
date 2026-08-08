//! The DoD for AWU 960: a skill reaches the model and runs **with
//! `skill-tool-provider` absent from the configuration**.
//!
//! `skills_module_tests.rs` calls the kernel directly, which proves the module
//! works but not that the host's tool path finds it. Stage 3 taught why that
//! distinction matters: the module was returning nothing and the still-loaded
//! extension quietly served every call, so the port looked finished when
//! nothing had moved. The only way to see that is to take the extension away —
//! which is what this does.
//!
//! Structure follows `skill_tool_provider_e2e_tests.rs` (mock LLM over a local
//! socket, real Orchestrator, real Wasm) so the two are directly comparable.
use rad::config::{ExecutionConfig, ExtensionConfig, ModuleConfig, PermissionConfig};
use rad::dag::Dag;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Writes a skill into the run's own workspace.
///
/// This used to have to go in the crate root: the kernel preopened the process
/// working directory, so a module could not see the test's temporary workspace
/// at all. AWU 965 rooted both the preopen and `proc-spawn` at the configured
/// workspace — matching what the extension host always did — so the skill now
/// lives where the rest of the run's files do, and nothing is written into the
/// repository.
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
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn a_skill_runs_end_to_end_with_no_tool_provider_extension() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    write_skill(
        &workspace,
        "rad_e2e_greeter",
        "---\ndescription: Greets someone by name.\n---\n\nSay hello to $ARGUMENTS warmly.",
    );
    fs::create_dir_all(&snapshots).unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    // Turn 1 calls the skill; turn 2 closes the task. Popped from the back.
    // Calls `skill`, not `rad_e2e_greeter`: one tool selects by argument (§4.5 ③).
    let turn1 = "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_g\",\"type\":\"function\",\"function\":{\"name\":\"skill\",\"arguments\":\"{\\\"name\\\":\\\"rad_e2e_greeter\\\",\\\"args\\\":\\\"Alice\\\"}\"}}]}}]}\n\ndata: [DONE]\n\n".to_string();
    let turn2 =
        "data: {\"choices\":[{\"delta\":{\"content\":\"Done.\"}}]}\n\ndata: [DONE]\n\n".to_string();
    let responses = Arc::new(Mutex::new(vec![turn2, turn1]));
    let _server_handle = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    // SAFETY: process-global, serialised by `TEST_MUTEX`.
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

    // No `skill-tool-provider`, and no tool-provider extension of any kind:
    // whatever tools the model is offered must come from the module.
    config.extensions = vec![ExtensionConfig {
        name: "rad-orchestrator".to_string(),
        enabled: true,
        role: "orchestrator".to_string(),
        source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
        permissions: Some(perms.clone()),
        config: HashMap::new(),
    }];
    config.modules = vec![
        ModuleConfig {
            name: "skills".to_string(),
            source: "target/wasm32-wasip2/debug/skills_module.wasm".to_string(),
            enabled: true,
            config: serde_json::Value::Null,
        },
        ModuleConfig {
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
        "test_skills_module_session".to_string(),
        dag.clone(),
        None,
    ));
    // `Orchestrator::new` boots whatever `modules` declares. Asserted rather
    // than assumed: with the module missing this test would pass vacuously
    // against an empty tool list.
    let loaded = orchestrator
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.modules())
        .unwrap_or_default();
    // Membership, not equality: the transport is a module too as of AWU 969,
    // and this test is about `skills` being there — not about being alone.
    assert!(
        loaded.iter().any(|m| m == "skills"),
        "the skills module must load, got {loaded:?}"
    );

    orchestrator
        .run_task("start".to_string())
        .expect("task spawning failed");

    let start_time = Instant::now();
    while start_time.elapsed() < Duration::from_secs(30) {
        if !orchestrator.is_running() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(
        !orchestrator.is_running(),
        "task did not finish within the wait budget; the assertions below would \
         report a half-completed run as a logic failure"
    );

    let dag_guard = dag.lock();
    let found = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Say hello to Alice warmly."));
    assert!(
        found,
        "the skill body never reached the conversation: {:?}",
        dag_guard
            .nodes
            .values()
            .map(|n| &n.text)
            .collect::<Vec<_>>()
    );
}
