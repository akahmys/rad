//! `modules/mcp` against a real MCP server over real pipes (AWU 964).
//!
//! The server is a small script spawned through `proc-spawn`, so this exercises
//! the whole path at once: the syscall, the `process`/`byte-stream` resources,
//! the JSON-RPC handshake, and `tools/list` / `tools/call`. Mocking the
//! transport would have left every one of those untested — the extension this
//! replaces had no test that ever spoke to a server.
use parking_lot::Mutex;
use rad::kernel::{KernelShared, ModuleRuntime};
use std::path::PathBuf;
use std::sync::Arc;

fn mcp_wasm() -> PathBuf {
    ["debug", "release"]
        .iter()
        .map(|p| PathBuf::from(format!("target/wasm32-wasip2/{p}/mcp_module.wasm")))
        .find(|p| p.exists())
        .expect("mcp_module.wasm not built for wasm32-wasip2")
}

/// A minimal MCP server: line-delimited JSON-RPC on stdio, one tool.
///
/// `flush` on every write is not optional — Python block-buffers a pipe, so
/// without it the handshake reply never arrives and the module times out.
const FAKE_SERVER: &str = r#"
import json, sys

TOOL = {
    "name": "fake_echo",
    "description": "Echoes what it is given.",
    "inputSchema": {"type": "object", "properties": {"text": {"type": "string"}}},
}

def reply(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        req = json.loads(line)
    except ValueError:
        continue
    method, rid = req.get("method"), req.get("id")
    if method == "initialize":
        reply({"jsonrpc": "2.0", "id": rid, "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "fake", "version": "0.1.0"},
        }})
    elif method == "tools/list":
        reply({"jsonrpc": "2.0", "id": rid, "result": {"tools": [TOOL]}})
    elif method == "tools/call":
        args = req.get("params", {}).get("arguments") or {}
        if args.get("fail"):
            reply({"jsonrpc": "2.0", "id": rid, "result": {
                "content": [{"type": "text", "text": "it went wrong"}], "isError": True}})
        else:
            reply({"jsonrpc": "2.0", "id": rid, "result": {
                "content": [{"type": "text", "text": "echo: " + str(args.get("text", ""))}]}})
    # notifications have no id and need no reply
"#;

struct Fixture {
    _dir: tempfile::TempDir,
    kernel: Arc<KernelShared>,
}

fn fixture() -> Fixture {
    // Asserted rather than skipped: a test that quietly does nothing when its
    // dependency is missing reports as a pass.
    let python = std::process::Command::new("python3")
        .arg("--version")
        .output();
    assert!(
        python.is_ok_and(|o| o.status.success()),
        "python3 is required for the fake MCP server"
    );

    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("fake_mcp_server.py");
    std::fs::write(&script, FAKE_SERVER).unwrap();

    let shared = KernelShared::new();
    shared.set_module_config(
        "mcp",
        serde_json::json!({
            "mcp_servers": {
                "fake": {
                    "command": "python3",
                    "args": ["-u", script.to_string_lossy()]
                }
            }
        }),
    );
    let rt = ModuleRuntime::load("mcp", &mcp_wasm(), &shared.engine, Arc::downgrade(&shared))
        .expect("mcp module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("mcp".to_string(), Arc::new(Mutex::new(rt)));
    Fixture {
        _dir: dir,
        kernel: shared,
    }
}

fn call(k: &KernelShared, method: &str, payload: &str) -> Result<serde_json::Value, String> {
    k.call("test", "mcp", method, payload)
        .map(|reply| serde_json::from_str(&reply).unwrap())
}

#[test]
fn a_server_is_spawned_handshaken_and_its_tools_listed() {
    let f = fixture();
    let res = call(&f.kernel, "mcp.tools.list", "{}").expect("listing should succeed");
    let tools = res["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1, "{tools:?}");
    assert_eq!(tools[0]["function"]["name"], "fake_echo");
    assert_eq!(
        tools[0]["function"]["description"],
        "Echoes what it is given."
    );
    // The schema is passed through from the server, not rewritten.
    assert_eq!(
        tools[0]["function"]["parameters"]["properties"]["text"]["type"],
        "string"
    );
}

#[test]
fn a_tool_call_reaches_the_server_and_returns_its_text() {
    let f = fixture();
    let res = call(
        &f.kernel,
        "mcp.tools.call",
        r#"{"name":"fake_echo","arguments":"{\"text\":\"hello\"}"}"#,
    )
    .expect("call should succeed");
    assert_eq!(res["content"], "echo: hello");
}

/// A tool that fails reports it as `isError` rather than a JSON-RPC error, and
/// the module normalises that to a leading `Error:` — which is what
/// rad-orchestrator's consecutive-failure circuit breaker counts.
#[test]
fn a_tool_level_failure_comes_back_prefixed() {
    let f = fixture();
    let res = call(
        &f.kernel,
        "mcp.tools.call",
        r#"{"name":"fake_echo","arguments":"{\"fail\":true}"}"#,
    )
    .unwrap();
    assert_eq!(res["content"], "Error: it went wrong");
}

#[test]
fn an_unknown_tool_is_refused_by_name() {
    let f = fixture();
    let err = call(
        &f.kernel,
        "mcp.tools.call",
        r#"{"name":"no_such_tool","arguments":"{}"}"#,
    )
    .unwrap_err();
    assert!(err.contains("no_such_tool"), "{err}");
}

/// Server tool lists do not change mid-session, so a second `list` must reuse
/// the cache rather than re-issuing `tools/list` — this runs on every LLM turn.
#[test]
fn a_second_listing_reuses_the_running_server() {
    let f = fixture();
    let first = call(&f.kernel, "mcp.tools.list", "{}").unwrap();
    let second = call(&f.kernel, "mcp.tools.list", "{}").unwrap();
    assert_eq!(first, second);
}

/// With nothing configured the module fails with a message saying so, rather
/// than reporting an empty tool list as success.
#[test]
fn a_module_with_no_servers_configured_says_so() {
    let shared = KernelShared::new();
    shared.set_module_config("mcp", serde_json::json!({}));
    let rt = ModuleRuntime::load("mcp", &mcp_wasm(), &shared.engine, Arc::downgrade(&shared))
        .expect("mcp module should load");
    shared
        .registry
        .lock()
        .register(rt.manifest.clone())
        .unwrap();
    shared
        .modules
        .lock()
        .insert("mcp".to_string(), Arc::new(Mutex::new(rt)));

    let err = call(&shared, "mcp.tools.list", "{}").unwrap_err();
    assert!(err.contains("no mcp_servers configured"), "{err}");
}
