use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::{RasRpcCommand, RasRpcRequest, RasRpcResponse};
use rad::process::ProcessManager;
use rad::wasm::WasmRuntime;

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};
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
    _sandbox: Arc<FsSandbox>,
    _process_manager: Arc<ProcessManager>,
    _active_processes: Arc<Mutex<HashMap<i32, rad::process::RunningProcess>>>,
    runtime: WasmRuntime,
}

fn setup_test_context(perms: PermissionConfig, dag: Arc<Mutex<Dag>>) -> TestContext {
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
    let active_processes = Arc::new(Mutex::new(HashMap::new()));

    let mut config = wasmtime::Config::new();
    config.wasm_multi_memory(true);
    let engine = Engine::new(&config).unwrap();
    let module = Module::new(&engine, E2E_WAT).unwrap();

    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag });
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
        _sandbox: sandbox,
        _process_manager: process_manager,
        _active_processes: active_processes,
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
fn test_context_restoration() {
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

    // Shared DAG to simulate database/filesystem persistence across restarts
    let dag = Arc::new(Mutex::new(Dag::new()));

    // 1. First runtime session
    {
        let mut ctx = setup_test_context(perms.clone(), dag.clone());

        // Create some nodes in the DAG
        let node_id_val = call_rpc(
            &mut ctx,
            RasRpcCommand::CreateNode {
                parent_id: "".to_string(),
                node_type: "user".to_string(),
            },
        )
        .unwrap();
        let node_id = node_id_val.as_str().unwrap().to_string();

        call_rpc(
            &mut ctx,
            RasRpcCommand::SetNodeText {
                node_id: node_id.clone(),
                text: "Hello from previous session".to_string(),
            },
        )
        .unwrap();
    }

    // 2. Second runtime session (restored from the same DAG instance)
    {
        let mut ctx = setup_test_context(perms, dag);

        // Get DAG via RPC in the new session and verify the node and its text still exist
        let dag_val = call_rpc(&mut ctx, RasRpcCommand::GetDag).unwrap();
        let restored_dag: Dag = serde_json::from_value(dag_val).unwrap();

        assert!(restored_dag.current_node_id.is_some());
        let current_node = restored_dag.nodes.get(restored_dag.current_node_id.as_ref().unwrap()).unwrap();
        assert_eq!(current_node.node_type, "user");
        assert_eq!(current_node.text, "Hello from previous session");
    }
}
