//! Does `modules/agent-loop` build the same message list the extension does?
//!
//! AWU 980 moved the DAG walk, the orphan filter and the system prompt into the
//! module by copying them. Its own tests prove the copy behaves as the copy was
//! written to; none of them prove it matches `ext/rad-orchestrator`, which is
//! still the code that actually runs. Until something compares the two on the
//! same DAG, "ported" is a claim rather than a finding.
//!
//! The comparison point is the wire. The extension's message list is not
//! reachable from here — it is private, inside a component, and built from host
//! RPCs — but every turn sends it to the backend, so a mock server that keeps
//! the request body has the real answer in its hands.
use parking_lot::Mutex;
use rad::config::{ExecutionConfig, ExtensionConfig, ModuleConfig, PermissionConfig};
use rad::dag::Dag;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Serialises `RAD_TEST_PORT`, which is process-global.
static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Keeps the whole request body, unlike `llm_module_e2e_tests`' server, which
/// reads a fixed buffer and discards it. A message list with a tool call in it
/// runs past 4KB easily, and a truncated body would silently compare the wrong
/// thing.
fn capturing_server(
    addr: &str,
    body: Arc<Mutex<Option<String>>>,
    dag: Arc<Mutex<Dag>>,
    dag_at_request: Arc<Mutex<Option<serde_json::Value>>>,
) -> std::thread::JoinHandle<()> {
    let listener = std::net::TcpListener::bind(addr).unwrap();
    std::thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let mut raw = Vec::new();
            let mut buf = [0u8; 4096];
            // Read until the body is as long as Content-Length says, rather
            // than until the peer closes — it will not close, it is waiting for
            // a response.
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        raw.extend_from_slice(&buf[..n]);
                        if let Some(complete) = complete_body(&raw) {
                            body.lock().replace(complete);
                            // The DAG *as the extension saw it*. Taken here
                            // rather than after the turn because the assistant's
                            // reply becomes a node once the stream finishes, and
                            // comparing a post-turn DAG against a pre-turn
                            // request is how the first version of this test
                            // reported a difference that was its own doing.
                            dag_at_request
                                .lock()
                                .replace(serde_json::to_value(&*dag.lock()).unwrap());
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            let headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\
                           Cache-Control: no-cache\r\nConnection: close\r\n\r\n";
            let _ = stream.write_all(headers.as_bytes());
            let _ = stream.write_all(
                b"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\ndata: [DONE]\n\n",
            );
            let _ = stream.flush();
            let _ = stream.shutdown(std::net::Shutdown::Write);
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}

/// `Some(body)` once every byte `Content-Length` promised has arrived.
fn complete_body(raw: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = text.split_once("\r\n\r\n")?;
    let len: usize = head.lines().find_map(|l| {
        l.to_ascii_lowercase()
            .strip_prefix("content-length:")?
            .trim()
            .parse()
            .ok()
    })?;
    (body.len() >= len).then(|| body.to_string())
}

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

fn module(name: &str, artefact: &str) -> ModuleConfig {
    ModuleConfig {
        name: name.to_string(),
        source: format!("target/wasm32-wasip2/debug/{artefact}.wasm"),
        enabled: true,
        config: serde_json::Value::Null,
    }
}

fn config_for(dir: &std::path::Path, with_agent_loop: bool) -> rad::config::Config {
    let workspace = dir.join("workspace");
    let snapshots = dir.join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    rad::config::Config {
        core: rad::config::CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshots.to_string_lossy().to_string(),
            log: dir.join("logs").to_string_lossy().to_string(),
            hitl_enabled: false,
            verification_command: None,
            ..Default::default()
        },
        extensions: vec![ExtensionConfig {
            name: "rad-orchestrator".to_string(),
            enabled: true,
            role: "orchestrator".to_string(),
            source: "target/wasm32-wasip2/debug/rad_orchestrator.wasm".to_string(),
            permissions: Some(permissions()),
            config: HashMap::new(),
        }],
        modules: {
            let mut m = vec![module("llm-openai", "llm_openai_module")];
            if with_agent_loop {
                m.push(module("agent-loop", "agent_loop_module"));
            }
            m
        },
        ..Default::default()
    }
}

/// Runs one turn against a capturing backend and returns the message list that
/// reached the wire.
fn messages_on_the_wire(with_agent_loop: bool) -> Vec<serde_json::Value> {
    let temp = tempfile::tempdir().unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let captured = Arc::new(Mutex::new(None));
    let dag_at_request = Arc::new(Mutex::new(None));

    let config = config_for(temp.path(), with_agent_loop);
    let dag = seeded_dag(&temp.path().join("snapshots"));
    let _server = capturing_server(
        &format!("127.0.0.1:{port}"),
        Arc::clone(&captured),
        Arc::clone(&dag),
        Arc::clone(&dag_at_request),
    );
    // SAFETY: process-global, serialised by `TEST_MUTEX` in the caller.
    unsafe { std::env::set_var("RAD_TEST_PORT", port.to_string()) };

    let orchestrator = Arc::new(rad::orchestrator::Orchestrator::new(
        config,
        "parity".to_string(),
        dag,
        None,
    ));
    orchestrator
        .run_task("go".to_string())
        .expect("task spawning failed");
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(30) && orchestrator.is_running() {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!orchestrator.is_running(), "the turn never finished");

    let body = captured
        .lock()
        .clone()
        .expect("the backend never received a request");
    let sent: serde_json::Value = serde_json::from_str(&body).unwrap();
    sent["messages"]
        .as_array()
        .unwrap_or_else(|| panic!("no messages in the request: {sent}"))
        .iter()
        .map(essence)
        .collect()
}

/// The module now *serves* the extension's message assembly (AWU 984), so the
/// two implementations can be compared by the only thing that distinguishes
/// them: whether `agent-loop` is loaded at all.
///
/// This replaces the earlier shape, which compared the extension's request
/// against a `agent.messages` call made from the test. That stopped meaning
/// anything the moment the extension started asking the module — it would have
/// been comparing the module against itself.
#[test]
fn the_request_is_the_same_with_and_without_the_module() {
    let _lock = TEST_MUTEX
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let with_module = messages_on_the_wire(true);
    let without_module = messages_on_the_wire(false);

    assert_eq!(
        with_module, without_module,
        "routing message assembly through agent-loop changed the request.\n\
         with: {with_module:#?}\nwithout: {without_module:#?}"
    );

    // Asserted on the shared result so agreement cannot come from both paths
    // producing nothing.
    let roles: Vec<_> = with_module.iter().map(|m| m["role"].clone()).collect();
    assert_eq!(
        roles,
        vec!["system", "user", "assistant", "tool", "user"],
        "the fixture did not exercise what it was built to exercise"
    );
    assert!(
        !with_module
            .iter()
            .any(|m| m["tool_call_id"] == "call_never_made"),
        "the orphan survived, so neither path is filtering"
    );
    assert!(
        with_module[0]["content"]
            .as_str()
            .unwrap_or_default()
            .contains("notes.md"),
        "the digest never fired, so this compares nothing about it"
    );
}

/// A conversation with everything the walk and the filter have opinions about:
/// a plain user turn, an assistant turn carrying a tool call, the matching tool
/// reply, and an **orphan** reply whose call was never made — the one the
/// filter must drop.
fn seeded_dag(snapshots: &std::path::Path) -> Arc<Mutex<Dag>> {
    let dag = Arc::new(Mutex::new(Dag::new()));
    let assistant = serde_json::json!({
        "role": "assistant",
        "tool_calls": [{
            "id": "call_kept",
            "type": "function",
            // A real `path`, so `digest::build_digest_addendum` has something
            // to report. With `{}` here the digest is empty and the comparison
            // holds without ever exercising it — which is how AWU 980 shipped
            // without the digest and AWU 981 failed to notice.
            "function": { "name": "write", "arguments": "{\"path\":\"notes.md\"}" }
        }]
    })
    .to_string();
    let kept = serde_json::json!({
        "role": "tool", "tool_call_id": "call_kept", "content": "wrote it"
    })
    .to_string();
    let orphan = serde_json::json!({
        "role": "tool", "tool_call_id": "call_never_made", "content": "should vanish"
    })
    .to_string();

    let mut guard = dag.lock();
    let mut parent = String::new();
    for (text, kind) in [
        ("Please write a file.", "user"),
        (assistant.as_str(), "assistant"),
        (kept.as_str(), "tool"),
        (orphan.as_str(), "tool"),
    ] {
        let id = guard.create_node(&parent, kind).unwrap();
        guard.set_node_text(&id, text).unwrap();
        fs::create_dir_all(snapshots.join(&id)).unwrap();
        parent = id;
    }
    drop(guard);
    dag
}

/// Reduces a message to what a backend actually distinguishes, so the
/// comparison is about content rather than about key order or absent-vs-null.
fn essence(msg: &serde_json::Value) -> serde_json::Value {
    let calls: Vec<serde_json::Value> = msg
        .get("tool_calls")
        .and_then(|c| c.as_array())
        .map(|calls| {
            calls
                .iter()
                .map(|c| serde_json::json!({ "id": c["id"], "name": c["function"]["name"] }))
                .collect()
        })
        .unwrap_or_default();
    serde_json::json!({
        "role": msg.get("role").cloned().unwrap_or(serde_json::Value::Null),
        "content": msg.get("content").cloned().unwrap_or(serde_json::Value::Null),
        "tool_call_id": msg.get("tool_call_id").cloned().unwrap_or(serde_json::Value::Null),
        "tool_calls": calls,
    })
}
