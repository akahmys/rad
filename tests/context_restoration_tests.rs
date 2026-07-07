use rad::config::{ExecutionConfig, PermissionConfig};
use rad::dag::Dag;
use rad::fs::FsSandbox;
use rad::ipc::RasRpcCommand;
use rad::process::ProcessManager;
use rad::wasm::WasmRuntime;

use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

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

    let wasm_path = "target/wasm32-wasip2/debug/openai_orchestrator.wasm";
    let dag_subsystem = Arc::new(rad::dag::DagSubsystemImpl { dag });
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
        _sandbox: sandbox,
        _process_manager: process_manager,
        _active_processes: active_processes,
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
fn test_context_restoration() {
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
