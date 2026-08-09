//! A task runs to completion through the transport module.
//!
//! Was AWU 968's DoD — completing **with `llm-connector` absent** — back when
//! the extension still existed to be absent from. AWU 969 deleted it, so what
//! this now holds is that the module serves a real turn end to end, and that
//! nothing else quietly does when it is removed.
//!
//! `llm_module_tests.rs` calls the kernel directly, which proves the module
//! works but not that the host's LLM path finds it. Stage 3 taught why that
//! distinction matters: the module was returning nothing and the still-loaded
//! extension quietly served every call, so the port looked finished when
//! nothing had moved. The only way to see it is to take the extension away.
//!
//! Structure follows `skills_module_e2e_tests.rs` so the two are comparable.
use rad::config::{ExecutionConfig, ExtensionConfig, ModuleConfig, PermissionConfig};
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
            let mut buf = [0; 4096];
            let _ = std::io::Read::read(&mut stream, &mut buf);
            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n";
            let _ = std::io::Write::write_all(&mut stream, headers.as_bytes());
            let resp = {
                let mut guard = responses.lock();
                guard.pop()
            };
            if let Some(chunks) = resp {
                let _ = std::io::Write::write_all(&mut stream, chunks.as_bytes());
            }
            let _ = std::io::Write::flush(&mut stream);
            let _ = stream.shutdown(std::net::Shutdown::Write);
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}

static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn permissions() -> PermissionConfig {
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

/// Builds a config with `rad-orchestrator` and nothing else on the extension
/// side. `with_module` decides whether the transport exists at all.
fn config_for(dir: &std::path::Path, with_module: bool) -> rad::config::Config {
    let workspace = dir.join("workspace");
    let snapshots = dir.join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let mut config = rad::config::Config {
        core: rad::config::CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: dir.join("logs").to_string_lossy().to_string(),
            hitl_enabled: false,
            verification_command: None,
            ..Default::default()
        },
        ..Default::default()
    };
    // The transport module is the only thing that can answer.
    config.extensions = vec![ExtensionConfig {
        name: "rad-orchestrator".to_string(),
        enabled: true,
        role: "orchestrator".to_string(),
        source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
        permissions: Some(permissions()),
        config: HashMap::new(),
    }];
    if with_module {
        config.modules = vec![
            ModuleConfig {
                name: "llm-openai".to_string(),
                source: "target/wasm32-wasip2/debug/llm_openai_module.wasm".to_string(),
                enabled: true,
                config: serde_json::Value::Null,
            },
            // Loaded alongside from AWU 979. It only accumulates — the
            // extension still runs the turn — so its presence changes nothing
            // about the assertions below, and its absence would leave the
            // transport's new `post` path untested against a real stream.
            ModuleConfig {
                name: "agent-loop".to_string(),
                source: "target/wasm32-wasip2/debug/agent_loop_module.wasm".to_string(),
                enabled: true,
                config: serde_json::Value::Null,
            },
        ];
    }
    config
}

fn seeded_dag(snapshots: &std::path::Path) -> Arc<Mutex<Dag>> {
    let dag = Arc::new(Mutex::new(Dag::new()));
    {
        let mut guard = dag.lock();
        let n0 = guard.create_node("", "user").unwrap();
        guard.set_node_text(&n0, "Initial").unwrap();
        fs::create_dir_all(snapshots.join(&n0)).unwrap();
    }
    dag
}

fn run_to_completion(orchestrator: &Arc<rad::orchestrator::Orchestrator>) {
    orchestrator
        .run_task("start".to_string())
        .expect("task spawning failed");
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(30) {
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
}

#[test]
fn a_turn_completes_through_the_transport_module() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp_dir = tempfile::tempdir().unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let responses = Arc::new(Mutex::new(vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Answered by the module.\"}}]}\n\ndata: [DONE]\n\n".to_string(),
    ]));
    let _server = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    // SAFETY: process-global, serialised by `TEST_MUTEX`.
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
    }

    let config = config_for(temp_dir.path(), true);
    let dag = seeded_dag(&temp_dir.path().join("snapshots"));
    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "test_llm_module_session".to_string(),
        dag.clone(),
        None,
    ));

    // Asserted rather than assumed: with the module missing this would fall
    // back to the extension path and the test would prove nothing.
    let loaded = orchestrator
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.modules())
        .unwrap_or_default();
    assert_eq!(
        loaded,
        vec!["agent-loop".to_string(), "llm-openai".to_string()],
        "the transport module must load"
    );

    run_to_completion(&orchestrator);

    let dag_guard = dag.lock();
    let answered = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Answered by the module."));
    assert!(
        answered,
        "the model's answer never reached the conversation: {:?}",
        dag_guard
            .nodes
            .values()
            .map(|n| &n.text)
            .collect::<Vec<_>>()
    );
    drop(dag_guard);

    // The same stream, seen by the module. `tests/agent_loop_tests.rs` drives
    // the intake with hand-built posts; this is the only place the transport
    // actually produces them, through the relay thread and the event loop's
    // drain. Removing `post_to_agent` fails here and nowhere else.
    let turn = orchestrator
        .kernel
        .lock()
        .as_ref()
        .map(|k| k.call("test", "agent-loop", "agent.turn", "{}"))
        .expect("the kernel is loaded")
        .expect("agent.turn must answer");
    let turn: serde_json::Value = serde_json::from_str(&turn).unwrap();
    assert_eq!(
        turn["text"], "Answered by the module.",
        "the transport's events never reached agent-loop: {turn}"
    );
    assert_eq!(turn["done"], true, "the turn never saw its terminator");
}

/// The negative control stage 3 lacked: the same run with the module removed
/// and *everything else identical* — same server, same response waiting on it.
/// The answer must not appear, which is what makes the test above evidence that
/// the module served it rather than something else having.
///
/// The failure itself surfaces on the terminal and through the recovery loop
/// ("No LLM transport is loaded...", twice, then "Wasm execution failed after
/// maximum recovery attempts"), not as a DAG node — the
/// orchestrator extension reports it as an L1 internal error. Asserted on what
/// is observable from here rather than on where it would be tidier to find it.
#[test]
fn without_the_module_the_same_run_produces_no_answer() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let temp_dir = tempfile::tempdir().unwrap();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let responses = Arc::new(Mutex::new(vec![
        "data: {\"choices\":[{\"delta\":{\"content\":\"Answered by the module.\"}}]}\n\ndata: [DONE]\n\n".to_string(),
    ]));
    let _server = run_mock_http_server(&format!("127.0.0.1:{port}"), responses);
    // SAFETY: process-global, serialised by `TEST_MUTEX`.
    unsafe {
        std::env::set_var("RAD_TEST_PORT", port.to_string());
    }

    let config = config_for(temp_dir.path(), false);
    let dag = seeded_dag(&temp_dir.path().join("snapshots"));
    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "test_llm_missing_session".to_string(),
        dag.clone(),
        None,
    ));
    assert!(
        orchestrator
            .kernel
            .lock()
            .as_ref()
            .is_none_or(|k| k.modules().is_empty()),
        "no module should be loaded in this case"
    );

    run_to_completion(&orchestrator);

    let dag_guard = dag.lock();
    let answered = dag_guard
        .nodes
        .values()
        .any(|n| n.text.contains("Answered by the module."));
    assert!(
        !answered,
        "with no transport configured the answer must not appear: {:?}",
        dag_guard
            .nodes
            .values()
            .map(|n| &n.text)
            .collect::<Vec<_>>()
    );
}
