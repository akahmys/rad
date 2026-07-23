use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::RasRpcCommand;
use rad::process::{ProcessManager, RunningProcess};
use rad::wasm::WasmRuntime;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

struct TestContext {
    _temp_dir: tempfile::TempDir,
    sandbox: Arc<FsSandbox>,
    process_manager: Arc<ProcessManager>,
    active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
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

    let wasm_path = "target/wasm32-wasip2/debug/rad_orchestrator.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag: dag.clone() });
    let network_subsystem = Arc::new(rad::http::HttpManager);
    let (event_tx, _event_rx) = std::sync::mpsc::channel();
    let runtime = WasmRuntime::new(
        "test-extension".to_string(),
        std::path::Path::new(wasm_path),
        "orchestrator".to_string(),
        perms,
        sandbox.clone() as Arc<dyn rad::subsystems::FsSubsystem>,
        process_manager.clone() as Arc<dyn rad::subsystems::ProcessSubsystem>,
        dag_subsystem,
        network_subsystem,
        active_processes.clone(),
        event_tx,
        None,
        false,
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
    use rad::wasm::bindings::RadExtensionImports;
    let wit_cmd = rad::wasm::bindings::wit::RasRpcCommand::from(command);
    let res = ctx.runtime.store.data_mut().host_rpc(wit_cmd);
    match res {
        Ok(json_str) => {
            if json_str.is_empty() || json_str == "null" {
                Ok(serde_json::Value::Null)
            } else {
                serde_json::from_str(&json_str).map_err(|e| e.to_string())
            }
        }
        Err(err_msg) => Err(err_msg),
    }
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
        ..Default::default()
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
            let active = ctx.active_processes.lock();
            active.contains_key(&pgid.to_string())
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
        let active = ctx.active_processes.lock();
        assert!(active.contains_key(&sleep_pgid.to_string()));
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
