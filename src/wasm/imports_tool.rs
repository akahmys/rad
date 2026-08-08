/// `execute_tool` implementation, split out of `imports_rpc.rs` to stay
/// under the 300-line file limit.
use crate::wasm::format_wasm_error;
use crate::wasm::{HostExecution, WasmState};

/// Puts the call past `security-guard` before anything runs.
///
/// Extracted so the module path (`execute_tool_text`) is guarded by exactly the
/// same policy as the extension path, and guarded *once* — running it on both
/// would double any policy that has a side effect, such as prompting a human.
fn verify_tool_call(state: &WasmState, name: &str, arguments: &str) -> Result<(), String> {
    let Some(orchestrator) = state.orchestrator.as_ref().and_then(|w| w.upgrade()) else {
        return Ok(());
    };
    let req = crate::ipc::RasRpcRequest {
        id: Some("test".to_string()),
        command: crate::ipc::RasRpcCommand::ExecuteTool {
            call_id: "test".to_string(),
            name: name.to_string(),
            arguments: arguments.to_string(),
        },
    };
    let req_bytes = serde_json::to_vec(&req).unwrap_or_default();
    orchestrator
        .verify_rpc_exclude(&state.name, &req, &req_bytes)
        .map_err(|e| format!("Operation rejected by security extension: {e}"))
}

/// `execute-tool-text`: the tool's output as a string.
///
/// A module returns its answer directly; an extension still returns a process,
/// so the handle is drained here instead of at every call site. Modules are
/// consulted first — during the migration a ported provider and the extension
/// it replaces can both be loaded, and the module is the one that should win.
pub(crate) fn execute_tool_text(
    state: &mut WasmState,
    name: String,
    arguments: String,
) -> Result<String, String> {
    verify_tool_call(state, &name, &arguments)?;

    let kernel = state
        .orchestrator
        .as_ref()
        .and_then(|w| w.upgrade())
        .and_then(|orch| orch.kernel.lock().clone());
    if let Some(kernel) = kernel
        && let Some(result) = crate::kernel::tools::execute(&kernel, &name, &arguments)
    {
        return result;
    }

    let handle = execute_tool_unverified(state, name, arguments)?;
    drain_to_string(state, handle)
}

/// Reads a finished extension tool's stdout to end-of-stream.
///
/// The receiver closes when the child's stdout does, so this ends on its own;
/// `wait` afterwards is what reaps the child rather than leaving a zombie.
fn drain_to_string(
    state: &mut WasmState,
    handle: wasmtime::component::Resource<HostExecution>,
) -> Result<String, String> {
    use wasmtime_wasi::WasiView;

    let exec = state
        .table()
        .delete(handle)
        .map_err(|e| format!("Failed to take execution handle: {e}"))?;
    let receiver = exec.stdout.lock().take();
    let mut out = Vec::new();
    if let Some(rx) = receiver {
        while let Ok(chunk) = rx.recv() {
            out.extend_from_slice(&chunk);
        }
    }
    let _ = exec
        .running
        .lock()
        .wait_with_timeout(std::time::Duration::from_secs(1));
    Ok(String::from_utf8_lossy(&out).into_owned())
}

pub(crate) fn execute_tool(
    state: &mut WasmState,
    name: String,
    arguments: String,
) -> Result<wasmtime::component::Resource<HostExecution>, String> {
    verify_tool_call(state, &name, &arguments)?;
    execute_tool_unverified(state, name, arguments)
}

/// The extension path proper. Callers must have run [`verify_tool_call`].
fn execute_tool_unverified(
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

    verify_tool_call(state, &name, &arguments)?;

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
