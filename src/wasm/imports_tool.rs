/// `execute_tool` implementation and its built-in-tool fallback path, split
/// out of `imports_rpc.rs` to stay under the 300-line file limit.
use crate::ipc::RasRpcRequest;
use crate::wasm::format_wasm_error;
use crate::wasm::imports_rpc::resolve_and_verify_path;
use crate::wasm::{HostExecution, WasmState, permissions};

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
            return execute_core_tool_fallback(state, &name, &arguments);
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

fn execute_core_tool_fallback(
    state: &mut WasmState,
    name: &str,
    arguments: &str,
) -> Result<wasmtime::component::Resource<HostExecution>, String> {
    crate::log_host!(
        "[HOST] WIT Core Tool Fallback: executing '{}' with args '{}'",
        name,
        arguments
    );

    // Reconstruct RasRpcCommand to perform security check
    crate::log_host!("[HOST] Fallback: parsing arguments and resolving path");
    let rpc_cmd = match name {
        "read" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
            }
            let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
            let resolved = resolve_and_verify_path(state.sandbox.workspace_dir(), &args.path)?;
            rad_models::RasRpcCommand::FileRead { path: resolved }
        }
        "write" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
                content: String,
            }
            let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
            let resolved = resolve_and_verify_path(state.sandbox.workspace_dir(), &args.path)?;
            rad_models::RasRpcCommand::FileWrite {
                path: resolved,
                data: args.content.into_bytes(),
            }
        }
        "edit" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
                diff: String,
            }
            let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
            let resolved = resolve_and_verify_path(state.sandbox.workspace_dir(), &args.path)?;
            rad_models::RasRpcCommand::FileEditPatch {
                path: resolved,
                diff: args.diff,
            }
        }
        "bash" => {
            #[derive(serde::Deserialize)]
            struct Args {
                command: String,
            }
            let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
            rad_models::RasRpcCommand::SpawnBashProcess {
                command: args.command,
            }
        }
        other => return Err(format!("Unknown core tool: {other}")),
    };

    permissions::check_permissions(&rpc_cmd, &state.permissions, state.sandbox.workspace_dir())
        .map_err(|e| format!("Permission denied in extension '{}': {e}", state.name))?;

    crate::log_host!("[HOST] Fallback: parsed command, fetching orchestrator");
    let orchestrator = state.orchestrator.as_ref().and_then(|w| w.upgrade());
    if let Some(ref orch) = orchestrator {
        let req = RasRpcRequest {
            id: Some("wasm_call".to_string()),
            command: rpc_cmd.clone(),
        };
        let buf = serde_json::to_vec(&req)
            .map_err(|e| format!("Failed to serialize request: {e}"))?;
        crate::log_host!("[HOST] Fallback: calling verify_rpc_exclude");
        if let Err(e) = orch.verify_rpc_exclude(&state.name, &req, &buf) {
            crate::log_host!("[HOST] Fallback: verify_rpc_exclude rejected request");
            return Err(format!("Security verification failed: {e}"));
        }
        crate::log_host!("[HOST] Fallback: verify_rpc_exclude accepted request");
    }

    let command_to_run = match name {
        "read" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse read args: {e}"))?;
            format!("cat '{}'", args.path)
        }
        "write" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
                content: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse write args: {e}"))?;

            let resolved = resolve_and_verify_path(state.sandbox.workspace_dir(), &args.path)?;
            state
                .sandbox
                .file_write(&resolved, args.content.as_bytes())?;
            "echo 'File written successfully.'".to_string()
        }
        "edit" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: String,
                diff: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse edit args: {e}"))?;

            let resolved = resolve_and_verify_path(state.sandbox.workspace_dir(), &args.path)?;
            state.sandbox.file_edit_patch(&resolved, &args.diff)?;
            "echo 'Patch applied successfully.'".to_string()
        }
        "bash" | "spawn_bash_process" => {
            #[derive(serde::Deserialize)]
            struct Args {
                #[serde(alias = "cmd")]
                command: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse bash args: {e}"))?;
            args.command
        }
        other => {
            return Err(format!("Tool '{other}' is not a valid built-in tool and no registered MCP tool provider handled it"));
        }
    };

    crate::wasm::bindings::RadExtensionImports::open_process(state, command_to_run)
}
