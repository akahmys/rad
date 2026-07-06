use super::*;
use crate::config::{ExecutionConfig, PermissionConfig};
use crate::dag::Dag;
use crate::fs::FsSandbox;
use crate::ipc::RasCoreEvent;
use crate::process::ProcessManager;

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
use wasmtime::{Engine, Module};

const TEST_WAT: &str = include_str!("test_module.wat");

struct TestContext {
    _temp_dir: tempfile::TempDir,
    sandbox: Arc<FsSandbox>,
    _process_manager: Arc<ProcessManager>,
    dag: Arc<Mutex<Dag>>,
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

    // Compile module from WAT
    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, TEST_WAT).unwrap();

    let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl { dag: dag.clone() });
    let network_subsystem = Arc::new(crate::http::HttpManager);
    let (event_tx, _event_rx) = std::sync::mpsc::channel();
    let runtime = WasmRuntime::new_with_module(
        "test-extension".to_string(),
        &module,
        perms,
        sandbox.clone() as Arc<dyn FsSubsystem>,
        process_manager.clone() as Arc<dyn ProcessSubsystem>,
        dag_subsystem,
        network_subsystem,
        active_processes.clone(),
        event_tx,
        None,
    ).unwrap();

    TestContext {
        _temp_dir: temp_dir,
        sandbox,
        _process_manager: process_manager,
        dag,
        active_processes,
        runtime,
    }
}

#[test]
fn test_file_write_via_wasm() {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: None,
        network: None,
    };

    let mut ctx = setup_test_context(perms);

    // Set test case to 1 (FileWrite test_write.txt)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 1).unwrap();

    // Trigger event
    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    ctx.runtime.on_event(&event).unwrap();

    // Check if file was written
    let content = ctx.sandbox.file_read(std::path::Path::new("test_write.txt")).unwrap();
    assert_eq!(content, b"hello");
}

#[test]
fn test_file_read_via_wasm() {
    let perms = PermissionConfig {
        fs_read_allow: vec!["*".to_string()],
        fs_write_allow: vec!["*".to_string()],
        execution: None,
        network: None,
    };

    let mut ctx = setup_test_context(perms);

    // Create a file to read
    ctx.sandbox.file_write(std::path::Path::new("test.txt"), b"file content").unwrap();

    // Set test case to 0 (FileRead test.txt)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 0).unwrap();

    // Trigger event
    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    ctx.runtime.on_event(&event).unwrap();
}

#[test]
fn test_spawn_process_via_wasm() {
    let perms = PermissionConfig {
        fs_read_allow: vec![],
        fs_write_allow: vec![],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec![],
        }),
        network: None,
    };

    let mut ctx = setup_test_context(perms);

    // Set test case to 2 (SpawnBashProcess echo hello)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 2).unwrap();

    // Trigger event
    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    ctx.runtime.on_event(&event).unwrap();

    // Verify process is registered in active_processes
    let processes = ctx.active_processes.lock().unwrap();
    assert_eq!(processes.len(), 1);
}

#[test]
fn test_spawn_process_blocked_via_wasm() {
    let perms = PermissionConfig {
        fs_read_allow: vec![],
        fs_write_allow: vec![],
        execution: Some(ExecutionConfig {
            allow_bash: true,
            allow_commands: vec![],
            block_commands: vec!["curl".to_string()],
        }),
        network: None,
    };

    let mut ctx = setup_test_context(perms);

    // Set test case to 3 (SpawnBashProcess Blocked: curl)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 3).unwrap();

    // Trigger event
    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    
    // We expect the operation to return Ok because host RPC returns an error response
    // rather than failing the execution of the guest itself.
    ctx.runtime.on_event(&event).unwrap();

    // Verify no processes were spawned
    let processes = ctx.active_processes.lock().unwrap();
    assert_eq!(processes.len(), 0);
}

#[test]
fn test_dag_operations_via_wasm() {
    let perms = PermissionConfig::default();
    let mut ctx = setup_test_context(perms);

    // Set test case to 4 (Create DAG Node)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 4).unwrap();

    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    ctx.runtime.on_event(&event).unwrap();

    // Check DAG
    let dag = ctx.dag.lock().unwrap();
    assert_eq!(dag.nodes.len(), 1);
    assert!(dag.nodes.contains_key("node_0"));
    assert_eq!(dag.nodes.get("node_0").unwrap().node_type, "task");
}

#[test]
fn test_get_dag_via_wasm() {
    let perms = PermissionConfig::default();
    let mut ctx = setup_test_context(perms);

    // Create a node to populate the DAG
    {
        let mut dag = ctx.dag.lock().unwrap();
        dag.create_node("", "root_task").unwrap();
    }

    // Set test case to 5 (GetDag)
    let set_test_case_fn = ctx.runtime.instance
        .get_typed_func::<i32, ()>(&mut ctx.runtime.store, "set_test_case")
        .unwrap();
    set_test_case_fn.call(&mut ctx.runtime.store, 5).unwrap();

    let event = RasCoreEvent::HumanInputReceived {
        text: "hello".to_string(),
    };
    
    // We expect GetDag RPC to be triggered when we invoke on_event
    ctx.runtime.on_event(&event).unwrap();
}
