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
                    let Some(mut runtime) = runtime_arc.try_lock() else {
                        continue;
                    };
                    if runtime.tool_provider.is_some() {
                        match runtime.get_tools() {
                            Ok(json_str) => {
                                if let Ok(serde_json::Value::Array(arr)) =
                                    serde_json::from_str::<serde_json::Value>(&json_str)
                                    && let Some(arr_ref) = all_tools.as_array_mut()
                                {
                                    arr_ref.extend(arr);
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
                    let Some(mut runtime) = runtime_arc.try_lock() else {
                        continue;
                    };
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
                execute_core_tool_fallback(name, arguments, ctx)
            } else {
                Err("Orchestrator unavailable".to_string())
            }
        }
        _ => unreachable!(),
    }
}

fn execute_core_tool_fallback(
    name: &str,
    arguments: &str,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    println!("[HOST] Core Tool Fallback: executing '{}' with args '{}'", name, arguments);
    let res = match name {
        "read" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: std::path::PathBuf,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse read args: {e}"))?;
            let val = super::rpc_fs::handle_fs(&RasRpcCommand::FileRead { path: args.path }, ctx)?;
            
            let result_str = if let Some(bytes_val) = val.as_array() {
                let bytes: Vec<u8> = bytes_val
                    .iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u8))
                    .collect();
                String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in file: {e}"))?
            } else if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                val.to_string()
            };
            Ok(serde_json::Value::String(result_str))
        }
        "write" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                content: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse write args: {e}"))?;
            let _ = super::rpc_fs::handle_fs(&RasRpcCommand::FileWrite {
                path: args.path,
                data: args.content.into_bytes(),
            }, ctx)?;
            Ok(serde_json::Value::String("File written successfully.".to_string()))
        }
        "edit" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                diff: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse edit args: {e}"))?;
            let _ = super::rpc_fs::handle_fs(&RasRpcCommand::FileEditPatch {
                path: args.path,
                diff: args.diff,
            }, ctx)?;
            Ok(serde_json::Value::String("Patch applied successfully.".to_string()))
        }
        "bash" => {
            #[derive(serde::Deserialize)]
            struct Args {
                command: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse bash args: {e}"))?;
            
            let val = super::rpc_process::handle_process(&RasRpcCommand::SpawnBashProcess {
                command: args.command,
            }, ctx)?;
            
            let pgid = val.as_i64().ok_or("Expected pgid")?.to_string();
            
            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(10));
                let mut procs = ctx.active_processes.lock();
                if let Some(proc) = procs.get_mut(&pgid) {
                    if proc.child.try_wait().ok().flatten().is_some() {
                        let (stdout, _stderr) = proc.read_available();
                        proc.unregister_pgid();
                        procs.remove(&pgid);
                        let out_str = String::from_utf8_lossy(&stdout).to_string();
                        return Ok(serde_json::Value::String(out_str));
                    }
                } else {
                    return Ok(serde_json::Value::String(String::new()));
                }
                if start.elapsed() > std::time::Duration::from_secs(30) {
                    return Err("Bash fallback execution timed out".to_string());
                }
            }
        }
        other => Err(format!("Unknown tool: {other}")),
    };
    println!("[HOST] Core Tool Fallback Result for '{}': {:?}", name, res);
    res
}
