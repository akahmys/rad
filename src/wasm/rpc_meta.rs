use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;
use crate::wasm::rpc_meta_fallback::execute_core_tool_fallback;

/// Handles meta/orchestration commands that are not specific to one subsystem.
pub fn handle_meta(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::CompleteTask => {
            let _ = ctx.event_tx.send(crate::ipc::RasCoreEvent::TaskCompleted);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::LogTracedEvent {
            trace_id,
            module,
            message,
        } => {
            crate::log_host!("\x1b[36m[TRACE {trace_id}]\x1b[0m \x1b[33m[{module}]\x1b[0m {message}");
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::AskHumanApproval { prompt } => {
            if !ctx.hitl_enabled {
                Ok(serde_json::Value::Bool(true))
            } else {
                let approved = crate::wasm::rpc_process::ask_human_approval_internal(prompt)?;
                Ok(serde_json::Value::Bool(approved))
            }
        }
        RasRpcCommand::ReportTokenUsage {
            prompt_tokens,
            completion_tokens,
        } => {
            if let Some(orch) = ctx.orchestrator {
                let mut usage = orch.token_usage.lock();
                usage.prompt_tokens += prompt_tokens;
                usage.completion_tokens += completion_tokens;
            }
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::GetTools => {
            if let Some(orch) = ctx.orchestrator {
                let mut all_tools = serde_json::Value::Array(Vec::new());
                let runtimes = {
                    let guard = orch.wasm_runtime.lock();
                    guard.values().cloned().collect::<Vec<_>>()
                };

                for runtime_arc in runtimes {
                    let Some(mut runtime) = runtime_arc.try_lock() else {
                        continue;
                    };
                    if runtime.tool_provider.is_some()
                        && let Ok(json_str) = runtime.get_tools()
                        && let Ok(serde_json::Value::Array(arr)) =
                            serde_json::from_str::<serde_json::Value>(&json_str)
                        && let Some(arr_ref) = all_tools.as_array_mut()
                    {
                        arr_ref.extend(arr);
                    }
                }
                Ok(all_tools)
            } else {
                Err("Orchestrator unavailable".to_string())
            }
        }

        RasRpcCommand::ExecuteTool {
            call_id,
            name,
            arguments,
        } => {
            if let Some(orch) = ctx.orchestrator {
                let provider_arc = {
                    let runtimes = {
                        let guard = orch.wasm_runtime.lock();
                        guard.values().cloned().collect::<Vec<_>>()
                    };
                    let mut provider = None;
                    for runtime_arc in runtimes {
                        let Some(mut runtime) = runtime_arc.try_lock() else {
                            continue;
                        };
                        if runtime.tool_provider.is_some()
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
                                provider = Some(runtime_arc.clone());
                                break;
                            }
                        }
                    }
                    provider
                };

                if let Some(provider_arc) = provider_arc {
                    let mut runtime = provider_arc.lock();
                    let args_val: serde_json::Value =
                        serde_json::from_str(arguments).unwrap_or(serde_json::Value::Null);
                    let _ = ctx
                        .event_tx
                        .send(crate::ipc::RasCoreEvent::ToolCallRequested {
                            call_id: call_id.clone(),
                            name: name.clone(),
                            args: args_val,
                        });

                    return runtime
                        .execute_tool(name, arguments)
                        .map(serde_json::Value::String)
                        .map_err(|e| format!("Tool execution failed: {e}"));
                }
                execute_core_tool_fallback(name, arguments, ctx)
            } else {
                Err("Orchestrator unavailable".to_string())
            }
        }
        RasRpcCommand::GenerateLlmStream {
            model,
            messages_json,
            tools_json,
        } => {
            if ctx.orchestrator.is_some() {
                crate::wasm::rpc_meta_llm_connector::generate(model, messages_json, tools_json, ctx)
            } else {
                crate::wasm::rpc_meta_llm_fallback::generate(ctx)
            }
        }
        RasRpcCommand::CallExtension {
            extension_id,
            method,
            arguments,
        } => {
            if let Some(orch) = ctx.orchestrator {
                // Clone the Arc out of the outer map lock and drop the guard
                // immediately, then use try_lock() on the target runtime.
                // Matches the GetTools/ExecuteTool pattern above; holding
                // wasm_runtime's lock for the duration of a nested extension
                // call is exactly the class of bug fixed in AWU 900.
                let runtime_arc = {
                    let guard = orch.wasm_runtime.lock();
                    guard.get(extension_id).cloned()
                };
                let Some(runtime_arc) = runtime_arc else {
                    return Ok(serde_json::Value::Null);
                };
                let Some(mut runtime) = runtime_arc.try_lock() else {
                    return Err(format!(
                        "Extension '{extension_id}' is busy handling another call"
                    ));
                };
                let res_str = runtime.call_extension_method(method, arguments)?;
                Ok(serde_json::Value::String(res_str))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        _ => unreachable!(),
    }
}
