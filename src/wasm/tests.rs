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

const TEST_WAT: &str = r#"
(module
  (import "env" "rad_host_rpc" (func $host_rpc (param i32 i32) (result i64)))
  (memory (export "memory") 1)
  (global $alloc_ptr (mut i32) (i32.const 1024))
  (global $test_case (mut i32) (i32.const 0))

  (func (export "alloc") (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $alloc_ptr))
    (global.set $alloc_ptr (i32.add (local.get $ptr) (local.get $size)))
    (local.get $ptr)
  )

  (func (export "dealloc") (param $ptr i32) (param $size i32)
    ;; no-op
  )

  (func (export "set_test_case") (param $val i32)
    (global.set $test_case (local.get $val))
  )

  (func (export "rad_on_event") (param $event_ptr i32) (param $event_len i32) (result i64)
    (local $case i32)
    (local.set $case (global.get $test_case))

    ;; Case 0: FileRead (test.txt) (len: 58)
    (if (i32.eq (local.get $case) (i32.const 0))
      (then
        (call $host_rpc (i32.const 0) (i32.const 58))
        (drop)
      )
    )

    ;; Case 1: FileWrite (test_write.txt) (len: 95)
    (if (i32.eq (local.get $case) (i32.const 1))
      (then
        (call $host_rpc (i32.const 100) (i32.const 95))
        (drop)
      )
    )

    ;; Case 2: SpawnBashProcess (echo hello) (len: 72)
    (if (i32.eq (local.get $case) (i32.const 2))
      (then
        (call $host_rpc (i32.const 200) (i32.const 72))
        (drop)
      )
    )

    ;; Case 3: SpawnBashProcess Blocked (curl ...) (len: 85)
    (if (i32.eq (local.get $case) (i32.const 3))
      (then
        (call $host_rpc (i32.const 300) (i32.const 85))
        (drop)
      )
    )

    ;; Case 4: Create DAG Node (len: 77)
    (if (i32.eq (local.get $case) (i32.const 4))
      (then
        (call $host_rpc (i32.const 400) (i32.const 77))
        (drop)
      )
    )

    (i64.const 0)
  )

  ;; JSON RPC Data Sections
  ;; Case 0: FileRead test.txt (len: 58)
  (data (i32.const 0) "{\"id\":\"1\",\"method\":\"FileRead\",\"params\":{\"path\":\"test.txt\"}}")

  ;; Case 1: FileWrite test_write.txt (len: 94)
  (data (i32.const 100) "{\"id\":\"2\",\"method\":\"FileWrite\",\"params\":{\"path\":\"test_write.txt\",\"data\":[104,101,108,108,111]}}")

  ;; Case 2: SpawnBashProcess echo hello (len: 72)
  (data (i32.const 200) "{\"id\":\"3\",\"method\":\"SpawnBashProcess\",\"params\":{\"command\":\"echo hello\"}}")

  ;; Case 3: SpawnBashProcess Blocked (curl) (len: 85)
  (data (i32.const 300) "{\"id\":\"4\",\"method\":\"SpawnBashProcess\",\"params\":{\"command\":\"curl http://example.com\"}}")

  ;; Case 4: CreateNode (len: 76)
  (data (i32.const 400) "{\"id\":\"5\",\"method\":\"CreateNode\",\"params\":{\"parent_id\":\"\",\"node_type\":\"task\"}}")
)
"#;

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
        &module,
        perms,
        sandbox.clone() as Arc<dyn FsSubsystem>,
        process_manager.clone() as Arc<dyn ProcessSubsystem>,
        dag_subsystem,
        network_subsystem,
        active_processes.clone(),
        event_tx,
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
