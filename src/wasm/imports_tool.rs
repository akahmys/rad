/// `execute_tool` implementation, split out of `imports_rpc.rs` to stay
/// under the 300-line file limit.
use crate::wasm::format_wasm_error;
use crate::wasm::{HostExecution, WasmState};

pub(crate) fn execute_tool(
    state: &mut WasmState,
    name: String,
    arguments: String,
) -> Result<wasmtime::component::Resource<HostExecution>, String> {
    use wasmtime_wasi::WasiView;

    crate::log_host!(
        "[HOST] RadExtensionImports::execute_tool called: name = '{}', args = '{}'",
        name,
        arguments
    );
    let mut provider_opt = None;

    if let Some(orchestrator) = state.orchestrator.as_ref().and_then(|w| w.upgrade()) {
        // Find the tool provider runtime
        let runtimes = {
            let guard = orchestrator.wasm_runtime.lock();
            guard.values().cloned().collect::<Vec<_>>()
        };
        for runtime_arc in runtimes {
            let Some(mut runtime) = runtime_arc.try_lock() else {
                continue;
            };
            if (runtime.role == "tool-provider" || runtime.tool_provider.is_some())
                && let Ok(json_str) = runtime.get_tools()
                && let Ok(serde_json::Value::Array(arr)) =
                    serde_json::from_str::<serde_json::Value>(&json_str)
            {
                let has_tool = arr.iter().any(|t| {
                    t.get("function")
                        .and_then(|f| f.get("name"))
                        .and_then(|n| n.as_str())
                        == Some(name.as_str())
                });
                if has_tool {
                    provider_opt = Some(runtime_arc.clone());
                    break;
                }
            }
        }
    }

    if let Some(orchestrator) = state.orchestrator.as_ref().and_then(|w| w.upgrade()) {
        let exec_cmd = crate::ipc::RasRpcCommand::ExecuteTool {
            call_id: "test".to_string(),
            name: name.clone(),
            arguments: arguments.clone(),
        };
        let dummy_req = crate::ipc::RasRpcRequest {
            id: Some("test".to_string()),
            command: exec_cmd,
        };
        let req_bytes = serde_json::to_vec(&dummy_req).unwrap_or_default();
        if let Err(e) = orchestrator.verify_rpc_exclude(&state.name, &dummy_req, &req_bytes) {
            return Err(format!("Operation rejected by security extension: {e}"));
        }
    }

    let provider_arc = match provider_opt {
        Some(arc) => arc,
        None => {
            return Err(format!(
                "Tool '{name}' is not a valid built-in tool and no registered tool provider handled it"
            ));
        }
    };
    let mut provider = provider_arc.lock();

    let provider_ref = &mut *provider;
    let ext_name = provider_ref.store.data().name.clone();
    let provider_res = {
        let prov = provider_ref
            .tool_provider
            .as_ref()
            .ok_or_else(|| "Tool provider bindings missing".to_string())?;
        prov.call_execute_tool(&mut provider_ref.store, &name, &arguments)
            .map_err(|e| format_wasm_error(&ext_name, "execute_tool", &e))??
    };

    // Extract the HostExecution from the provider's table
    let provider_state = provider.store.data_mut();
    let host_exec = provider_state
        .table()
        .delete(provider_res)
        .map_err(|e| format!("Failed to extract resource from provider table: {e}"))?;

    // Push it into our (the caller/orchestrator) table
    let our_res = state
        .table()
        .push(host_exec)
        .map_err(|e| format!("Failed to insert resource into caller table: {e}"))?;

    Ok(our_res)
}
