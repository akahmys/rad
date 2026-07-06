use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::{RasRpcCommand, RasRpcRequest, RasRpcResponse};
use rad::process::{ProcessManager, RunningProcess};
use rad::wasm::WasmRuntime;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use wasmtime::{Engine, Module};

const E2E_WAT: &str = r#"
(module
  (import "env" "rad_host_rpc" (func $host_rpc (param i32 i32) (result i64)))
  (memory (export "memory") 1)
  (global $alloc_ptr (mut i32) (i32.const 1024))

  (func (export "alloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $alloc_ptr))
    (global.set $alloc_ptr (i32.add (local.get $ptr) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "dealloc") (param $ptr i32) (param $size i32)
    ;; no-op
  )

  (func (export "run_step") (param $ptr i32) (param $len i32) (result i64)
    (call $host_rpc (local.get $ptr) (local.get $len))
  )
)
"#;

struct TestContext {
    _temp_dir: tempfile::TempDir,
    sandbox: Arc<FsSandbox>,
    process_manager: Arc<ProcessManager>,
    active_processes: Arc<Mutex<HashMap<i32, RunningProcess>>>,
    runtime: WasmRuntime,
}

fn setup_test_context(perms: PermissionConfig) -> TestContext {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    let snapshots = temp_dir.path().join("snapshots");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(&snapshots).unwrap();

    let sandbox = Arc::new(FsSandbox::new(
        workspace.clone(),
        snapshots,
        perms.fs_read_allow.clone(),
        perms.fs_write_allow.clone(),
    ));
    let process_manager = Arc::new(ProcessManager::new());
    let dag = Arc::new(Mutex::new(Dag::new()));
    let active_processes = Arc::new(Mutex::new(HashMap::new()));

    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, E2E_WAT).unwrap();

    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let network_subsystem = Arc::new(rad::http::HttpManager);
    let (event_tx, _event_rx) = std::sync::mpsc::channel();
    let runtime = WasmRuntime::new_with_module(
        "test-extension".to_string(),
        &module,
        perms,
        sandbox.clone() as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager.clone() as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network_subsystem,
        active_processes.clone(),
        event_tx,
        None,
    )
    .unwrap();

    TestContext {
        _temp_dir: temp_dir,
        sandbox,
        process_manager,
        active_processes,
        runtime,
    }
}

fn call_rpc(ctx: &mut TestContext, command: RasRpcCommand) -> Result<serde_json::Value, String> {
    let request = RasRpcRequest {
        id: Some("test_id".to_string()),
        command,
    };
    let req_bytes = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
    let len = i32::try_from(req_bytes.len()).map_err(|e| e.to_string())?;

    let alloc_fn = ctx
        .runtime
        .instance
        .get_typed_func::<i32, i32>(&mut ctx.runtime.store, "alloc")
        .map_err(|e| e.to_string())?;

    let ptr = alloc_fn
        .call(&mut ctx.runtime.store, len)
        .map_err(|e| e.to_string())?;

    let memory = ctx
        .runtime
        .instance
        .get_export(&mut ctx.runtime.store, "memory")
        .and_then(wasmtime::Extern::into_memory)
        .ok_or_else(|| "Failed to get memory".to_string())?;

    memory
        .write(&mut ctx.runtime.store, ptr as usize, &req_bytes)
        .map_err(|e| e.to_string())?;

    let run_step_fn = ctx
        .runtime
        .instance
        .get_typed_func::<(i32, i32), u64>(&mut ctx.runtime.store, "run_step")
        .map_err(|e| e.to_string())?;

    let ret = run_step_fn
        .call(&mut ctx.runtime.store, (ptr, len))
        .map_err(|e| e.to_string())?;

    let resp_ptr = (ret >> 32) as usize;
    let resp_len = (ret & 0xFFFF_FFFF) as usize;

    if resp_ptr == 0 || resp_len == 0 {
        return Err("Empty response from host RPC".to_string());
    }

    let mut resp_buf = vec![0; resp_len];
    memory
        .read(&ctx.runtime.store, resp_ptr, &mut resp_buf)
        .map_err(|e| e.to_string())?;

    let response: RasRpcResponse = serde_json::from_slice(&resp_buf).map_err(|e| e.to_string())?;

    response.result
}

#[test]
fn test_e2e_full_flow() {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec![],
        }),
        network: None,
    };

    let mut ctx = setup_test_context(perms);

    verify_pty_flow(&mut ctx);
    verify_edit_flow(&mut ctx);
    let node_id = verify_snapshot_flow(&mut ctx);
    verify_rollback_flow(&mut ctx, &node_id);
    verify_cleanup_flow(ctx);
}

fn verify_pty_flow(ctx: &mut TestContext) {
    let pty_res = call_rpc(
        ctx,
        RasRpcCommand::SpawnBashProcess {
            command: "echo 'hello' > pty_out.txt".to_string(),
        },
    )
    .unwrap();
    let pgid = pty_res.as_i64().unwrap() as i32;
    assert!(pgid > 0);

    let start = Instant::now();
    loop {
        let is_running = {
            let active = ctx.active_processes.lock().unwrap();
            active.contains_key(&pgid)
        };
        if !is_running {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            panic!("Spawned process timed out");
        }
        thread::sleep(Duration::from_millis(50));
    }

    let path = Path::new("pty_out.txt");
    let content = ctx.sandbox.file_read(path).unwrap();
    assert_eq!(content, b"hello\n");
}

fn verify_edit_flow(ctx: &mut TestContext) {
    let diff = "--- a\n+++ b\n@@ -1 +1 @@\n-hello\n+hello world\n";
    call_rpc(
        ctx,
        RasRpcCommand::FileEditPatch {
            path: PathBuf::from("pty_out.txt"),
            diff: diff.to_string(),
        },
    )
    .unwrap();

    let path = Path::new("pty_out.txt");
    let content_after_patch = ctx.sandbox.file_read(path).unwrap();
    assert_eq!(content_after_patch, b"hello world\n");
}

fn verify_snapshot_flow(ctx: &mut TestContext) -> String {
    let node_id_val = call_rpc(
        ctx,
        RasRpcCommand::CreateNode {
            parent_id: "".to_string(),
            node_type: "task".to_string(),
        },
    )
    .unwrap();
    let node_id = node_id_val.as_str().unwrap().to_string();

    call_rpc(
        ctx,
        RasRpcCommand::TakeSnapshot {
            node_id: node_id.clone(),
            target_paths: vec![PathBuf::from("pty_out.txt")],
        },
    )
    .unwrap();

    node_id
}

fn verify_rollback_flow(ctx: &mut TestContext, node_id: &str) {
    call_rpc(
        ctx,
        RasRpcCommand::FileWrite {
            path: PathBuf::from("pty_out.txt"),
            data: b"modified".to_vec(),
        },
    )
    .unwrap();

    let path = Path::new("pty_out.txt");
    let content_after_write = ctx.sandbox.file_read(path).unwrap();
    assert_eq!(content_after_write, b"modified");

    call_rpc(
        ctx,
        RasRpcCommand::CheckoutSnapshot {
            node_id: node_id.to_string(),
        },
    )
    .unwrap();

    let content_after_rollback = ctx.sandbox.file_read(path).unwrap();
    assert_eq!(content_after_rollback, b"hello world\n");
}

fn verify_cleanup_flow(mut ctx: TestContext) {
    let sleep_res = call_rpc(
        &mut ctx,
        RasRpcCommand::SpawnBashProcess {
            command: "sleep 100".to_string(),
        },
    )
    .unwrap();
    let sleep_pgid = sleep_res.as_i64().unwrap() as i32;

    {
        let active = ctx.active_processes.lock().unwrap();
        assert!(active.contains_key(&sleep_pgid));
    }

    let pm = ctx.process_manager.clone();
    drop(ctx);
    drop(pm);

    thread::sleep(Duration::from_millis(150));
    let kill_res = nix::sys::signal::kill(nix::unistd::Pid::from_raw(-sleep_pgid), None);
    assert!(
        kill_res.is_err(),
        "Process group {sleep_pgid} was not killed"
    );
}
