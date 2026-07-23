use super::*;
use crate::config::{ExecutionConfig, PermissionConfig};
use crate::dag::Dag;
use crate::fs::FsSandbox;
use crate::process::ProcessManager;

use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

struct TestContext {
    _temp_dir: tempfile::TempDir,
    _sandbox: Arc<FsSandbox>,
    _process_manager: Arc<ProcessManager>,
    _dag: Arc<Mutex<Dag>>,
    _active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
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

    let debug_path = std::path::Path::new("target/wasm32-wasip2/debug/security_guard.wasm");
    let release_path = std::path::Path::new("target/wasm32-wasip2/release/security_guard.wasm");
    let wasm_path = if debug_path.exists() {
        debug_path
    } else {
        release_path
    };

    let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl { dag: dag.clone() });
    let network_subsystem = Arc::new(crate::http::HttpManager);
    let (event_tx, _event_rx) = std::sync::mpsc::channel();

    let runtime = WasmRuntime::new(
        "test-extension".to_string(),
        wasm_path,
        "security".to_string(),
        perms,
        sandbox.clone() as Arc<dyn FsSubsystem>,
        process_manager.clone() as Arc<dyn ProcessSubsystem>,
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
        _sandbox: sandbox,
        _process_manager: process_manager,
        _dag: dag,
        _active_processes: active_processes,
        runtime,
    }
}

#[test]
fn test_verify_rpc_blocked_file() {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        ..Default::default()
    };
    let mut ctx = setup_test_context(perms);

    let req = crate::ipc::RasRpcRequest {
        id: Some("wasm_call".to_string()),
        command: rad_models::RasRpcCommand::FileWrite {
            path: std::path::PathBuf::from("blocked.txt"),
            data: b"dangerous".to_vec(),
        },
    };
    let req_bytes = serde_json::to_vec(&req).unwrap();
    let res = ctx.runtime.verify_rpc(&req_bytes);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err(), "Operation rejected by security extension");
}

#[test]
fn test_verify_rpc_blocked_command() {
    let perms = PermissionConfig {
        fs_read_allow: vec![],
        fs_write_allow: vec![],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec![],
        }),
        ..Default::default()
    };
    let mut ctx = setup_test_context(perms);

    let req = crate::ipc::RasRpcRequest {
        id: Some("wasm_call".to_string()),
        command: rad_models::RasRpcCommand::SpawnBashProcess {
            command: "blocked_command and parameters".to_string(),
        },
    };
    let req_bytes = serde_json::to_vec(&req).unwrap();
    let res = ctx.runtime.verify_rpc(&req_bytes);
    assert!(res.is_err());
}

#[test]
fn test_verify_rpc_allowed() {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        ..Default::default()
    };
    let mut ctx = setup_test_context(perms);

    let req = crate::ipc::RasRpcRequest {
        id: Some("wasm_call".to_string()),
        command: rad_models::RasRpcCommand::FileWrite {
            path: std::path::PathBuf::from("allowed.txt"),
            data: b"safe data".to_vec(),
        },
    };
    let req_bytes = serde_json::to_vec(&req).unwrap();
    let res = ctx.runtime.verify_rpc(&req_bytes);
    assert!(res.is_ok());
}

#[test]
fn test_resolve_and_verify_path_helper() {
    let temp_dir = tempfile::tempdir().unwrap();
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    let workspace = workspace.canonicalize().unwrap();

    let safe_file = "safe.txt";
    let res = super::imports_rpc::resolve_and_verify_path(&workspace, safe_file);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), workspace.join("safe.txt"));

    let unsafe_traversal = "../unsafe.txt";
    let res = super::imports_rpc::resolve_and_verify_path(&workspace, unsafe_traversal);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Access denied"));
}
