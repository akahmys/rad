use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;

/// Handles meta/orchestration commands that are not specific to one subsystem.
pub fn handle_meta(cmd: &RasRpcCommand, ctx: &RpcContext<'_>) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::CompleteTask => {
            let _ = ctx.event_tx.send(crate::ipc::RasCoreEvent::TaskCompleted);
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
                let runtimes = orch.wasm_runtime.lock();
                for (id, runtime_arc) in runtimes.iter() {
                    let mut runtime = runtime_arc.lock();
                    if runtime.tool_provider.is_some() {
                        match runtime.get_tools() {
                            Ok(json_str) => {
                                if let Ok(serde_json::Value::Array(arr)) =
                                    serde_json::from_str::<serde_json::Value>(&json_str)
                                {
                                    all_tools.as_array_mut().unwrap().extend(arr);
                                }
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Failed to get tools from runtime '{}': {}",
                                    id, e
                                ));
                            }
                        }
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
                let runtimes = orch.wasm_runtime.lock();
                for (id, runtime_arc) in runtimes.iter() {
                    let mut runtime = runtime_arc.lock();
                    if runtime.tool_provider.is_some() {
                        let args_val: serde_json::Value = serde_json::from_str(arguments)
                            .unwrap_or(serde_json::Value::Null);
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
                            .map_err(|e| {
                                format!("Tool execution failed in runtime '{}': {}", id, e)
                            });
                    }
                }
                Err(format!("No Tool Provider found to execute tool '{}'", name))
            } else {
                Err("Orchestrator unavailable".to_string())
            }
        }
        _ => unreachable!(),
    }
}
