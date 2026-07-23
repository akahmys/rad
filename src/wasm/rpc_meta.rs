use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;

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
            println!("\x1b[36m[TRACE {trace_id}]\x1b[0m \x1b[33m[{module}]\x1b[0m {message}");
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
                                return Err(format!("Failed to get tools: {e}"));
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
            #[derive(serde::Deserialize)]
            struct RemoteToolCallFunction {
                name: String,
                arguments: String,
            }

            #[derive(serde::Deserialize)]
            struct RemoteToolCall {
                id: String,
                #[serde(rename = "type")]
                tool_type: String,
                function: RemoteToolCallFunction,
            }

            #[derive(serde::Deserialize)]
            struct RemoteMessage {
                role: String,
                content: Option<String>,
                name: Option<String>,
                tool_call_id: Option<String>,
                tool_calls: Option<Vec<RemoteToolCall>>,
            }

            #[derive(serde::Deserialize)]
            struct RemoteFunctionDefinition {
                name: String,
                description: Option<String>,
                parameters: serde_json::Value,
            }

            #[derive(serde::Deserialize)]
            struct RemoteTool {
                #[serde(rename = "type")]
                tool_type: String,
                function: RemoteFunctionDefinition,
            }

            if let Some(orch) = ctx.orchestrator {
                eprintln!("[DEBUG] Host GenerateLlmStream starting...");
                let connector_arc = {
                    let runtimes = orch.wasm_runtime.lock();
                    let mut connector_runtime_opt = None;
                    for runtime_arc in runtimes.values() {
                        let Some(runtime) = runtime_arc.try_lock() else {
                            continue;
                        };
                        if runtime.llm_connector.is_some() {
                            connector_runtime_opt = Some(runtime_arc.clone());
                            break;
                        }
                    }
                    connector_runtime_opt.ok_or_else(|| {
                        "LLM Connector extension not found or not loaded".to_string()
                    })?
                };
                eprintln!("[DEBUG] Host found connector_arc.");

                let mut connector = connector_arc.lock();
                let connector_ref = &mut *connector;

                let conn_bindings = connector_ref
                    .llm_connector
                    .as_ref()
                    .ok_or_else(|| "LLM Connector bindings missing".to_string())?;

                let remote_messages: Vec<RemoteMessage> = serde_json::from_str(messages_json)
                    .map_err(|e| format!("Failed to parse messages JSON: {e}"))?;

                let remote_tools: Vec<RemoteTool> = serde_json::from_str(tools_json)
                    .map_err(|e| format!("Failed to parse tools JSON: {e}"))?;

                use crate::wasm::bindings::rad_llm_connector::radcomp::connector::types as conn_types;

                let wit_messages: Vec<conn_types::Message> = remote_messages
                    .into_iter()
                    .map(|m| conn_types::Message {
                        role: m.role,
                        content: m.content,
                        name: m.name,
                        tool_call_id: m.tool_call_id,
                        tool_calls: m.tool_calls.map(|calls| {
                            calls
                                .into_iter()
                                .map(|c| conn_types::ToolCall {
                                    id: c.id,
                                    tool_type: c.tool_type,
                                    function: conn_types::ToolCallFunction {
                                        name: c.function.name,
                                        arguments: c.function.arguments,
                                    },
                                })
                                .collect()
                        }),
                    })
                    .collect();

                let wit_tools: Vec<conn_types::Tool> = remote_tools
                    .into_iter()
                    .map(|t| conn_types::Tool {
                        tool_type: t.tool_type,
                        function: conn_types::FunctionDefinition {
                            name: t.function.name,
                            description: t.function.description,
                            parameters: t.function.parameters.to_string(),
                        },
                    })
                    .collect();

                let stream_res = match conn_bindings
                    .radcomp_connector_producer()
                    .call_generate_stream(
                        &mut connector_ref.store,
                        model,
                        &wit_messages,
                        &wit_tools,
                    ) {
                    Ok(Ok(stream)) => stream,
                    Ok(Err(e)) => {
                        eprintln!("\x1b[31m[LLM Connector Error] {e}\x1b[0m");
                        return Err(format!("LLM Stream Generation Error: {e}"));
                    }
                    Err(e) => {
                        eprintln!("\x1b[31m[LLM Connector Call Error] {e}\x1b[0m");
                        return Err(format!("LLM Connector Call Error: {e}"));
                    }
                };
                let resource_any = stream_res;

                // Spawn a thread to poll the event stream.
                let connector_arc_clone = connector_arc.clone();
                let event_tx_clone = ctx.event_tx.clone();
                std::thread::spawn(move || {
                    loop {
                        let read_res = {
                            let mut connector = connector_arc_clone.lock();
                            let connector_ref = &mut *connector;
                            let store = &mut connector_ref.store;
                            let conn_bindings = connector_ref.llm_connector.as_ref().unwrap();

                            conn_bindings
                                .radcomp_connector_producer()
                                .event_stream()
                                .call_read(store, resource_any)
                        };

                        match read_res {
                            Ok(Ok(Some(event))) => {
                                if let Ok(event_json) = serde_json::to_string(&event) {
                                    let _ = event_tx_clone.send(
                                        crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                            event: event_json,
                                        },
                                    );
                                }
                            }
                            Ok(Ok(None)) => {
                                let _ = event_tx_clone.send(
                                    crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                        event: serde_json::json!({ "type": "done" }).to_string(),
                                    },
                                );
                                break;
                            }
                            Ok(Err(e)) => {
                                let _ = event_tx_clone.send(
                                    crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                        event: serde_json::json!({ "type": "error", "payload": e })
                                            .to_string(),
                                    },
                                );
                                break;
                            }
                            Err(e) => {
                                let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                    event: serde_json::json!({ "type": "error", "payload": e.to_string() }).to_string(),
                                });
                                break;
                            }
                        }
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    }
                });

                Ok(serde_json::Value::Null)
            } else {
                let (tx, rx) = std::sync::mpsc::channel();
                let _ = ctx.network.open_http_stream(
                    "http://127.0.0.1/v1/chat/completions",
                    std::collections::HashMap::new(),
                    "",
                    tx,
                    ctx.llm_timeout_policy.clone(),
                )?;

                let event_tx_clone = ctx.event_tx.clone();
                std::thread::spawn(move || {
                    let mut buffer = String::new();
                    while let Ok(event) = rx.recv() {
                        if let crate::ipc::RasCoreEvent::HttpChunkReceived { chunk } = event {
                            buffer.push_str(&chunk);
                            while let Some(pos) = buffer.find('\n') {
                                let line = buffer[..pos].trim().to_string();
                                buffer = buffer[pos + 1..].to_string();
                                if line.is_empty() {
                                    continue;
                                }
                                if let Some(stripped) = line.strip_prefix("data:") {
                                    let data_str = stripped.trim();
                                    if data_str == "[DONE]" {
                                        let _ = event_tx_clone.send(
                                            crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                                event: serde_json::json!({ "type": "done" })
                                                    .to_string(),
                                            },
                                        );
                                        break;
                                    }
                                    if let Ok(val) =
                                        serde_json::from_str::<serde_json::Value>(data_str)
                                    {
                                        if let Some(reasoning) = val
                                            .pointer("/choices/0/delta/reasoning_content")
                                            .and_then(serde_json::Value::as_str)
                                        {
                                            let ev =
                                                serde_json::json!({ "ReasoningChunk": reasoning });
                                            let _ = event_tx_clone.send(
                                                crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                                    event: ev.to_string(),
                                                },
                                            );
                                        } else if let Some(content) = val
                                            .pointer("/choices/0/delta/content")
                                            .and_then(serde_json::Value::as_str)
                                        {
                                            let ev = serde_json::json!({ "ContentChunk": content });
                                            let _ = event_tx_clone.send(
                                                crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                                    event: ev.to_string(),
                                                },
                                            );
                                        }

                                        if let Some(tool_calls) = val
                                            .pointer("/choices/0/delta/tool_calls")
                                            .and_then(serde_json::Value::as_array)
                                        {
                                            for tc in tool_calls {
                                                let index = tc
                                                    .get("index")
                                                    .and_then(serde_json::Value::as_u64)
                                                    .unwrap_or(0);
                                                let id = tc
                                                    .get("id")
                                                    .and_then(serde_json::Value::as_str);
                                                let name = tc
                                                    .pointer("/function/name")
                                                    .and_then(serde_json::Value::as_str);
                                                let arguments_chunk = tc
                                                    .pointer("/function/arguments")
                                                    .and_then(serde_json::Value::as_str)
                                                    .unwrap_or("");
                                                let ev = serde_json::json!({
                                                    "ToolCallChunk": {
                                                        "index": index,
                                                        "id": id,
                                                        "name": name,
                                                        "arguments-chunk": arguments_chunk,
                                                    }
                                                });
                                                let _ = event_tx_clone.send(
                                                    crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                                        event: ev.to_string(),
                                                    },
                                                );
                                            }
                                        }

                                        if let Some(usage) = val.get("usage") {
                                            let prompt_tokens = usage
                                                .get("prompt_tokens")
                                                .and_then(serde_json::Value::as_u64)
                                                .unwrap_or(0);
                                            let completion_tokens = usage
                                                .get("completion_tokens")
                                                .and_then(serde_json::Value::as_u64)
                                                .unwrap_or(0);
                                            if prompt_tokens > 0 || completion_tokens > 0 {
                                                let ev = serde_json::json!({
                                                    "CompletionComplete": {
                                                        "prompt-tokens": prompt_tokens,
                                                        "completion-tokens": completion_tokens,
                                                    }
                                                });
                                                let _ = event_tx_clone.send(
                                                    crate::ipc::RasCoreEvent::LlmConnectorEvent {
                                                        event: ev.to_string(),
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
                Ok(serde_json::Value::Null)
            }
        }
        RasRpcCommand::CallExtension {
            extension_id,
            method,
            arguments,
        } => {
            if let Some(orch) = ctx.orchestrator {
                let runtimes = orch.wasm_runtime.lock();
                if let Some(runtime_arc) = runtimes.get(extension_id) {
                    let mut runtime = runtime_arc.lock();
                    let res_str = runtime.call_extension_method(method, arguments)?;
                    Ok(serde_json::Value::String(res_str))
                } else {
                    Ok(serde_json::Value::Null)
                }
            } else {
                Ok(serde_json::Value::Null)
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
    crate::log_host!(
        "[HOST] Core Tool Fallback: executing '{}' with args '{}'",
        name,
        arguments
    );
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
            let _ = super::rpc_fs::handle_fs(
                &RasRpcCommand::FileWrite {
                    path: args.path,
                    data: args.content.into_bytes(),
                },
                ctx,
            )?;
            Ok(serde_json::Value::String(
                "File written successfully.".to_string(),
            ))
        }
        "edit" => {
            #[derive(serde::Deserialize)]
            struct Args {
                path: std::path::PathBuf,
                diff: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse edit args: {e}"))?;
            let _ = super::rpc_fs::handle_fs(
                &RasRpcCommand::FileEditPatch {
                    path: args.path,
                    diff: args.diff,
                },
                ctx,
            )?;
            Ok(serde_json::Value::String(
                "Patch applied successfully.".to_string(),
            ))
        }
        "bash" | "spawn_bash_process" | "execute_command" | "terminal" | "sh" => {
            #[derive(serde::Deserialize)]
            struct Args {
                #[serde(alias = "cmd")]
                command: String,
            }
            let args: Args = serde_json::from_str(arguments)
                .map_err(|e| format!("Failed to parse command args: {e}"))?;

            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0);
            let call_id = format!("wasm_proc_{ts}");

            let mut running = ctx.process_manager.spawn_bash_process(
                &args.command,
                Some(ctx.sandbox.workspace_dir()),
                call_id,
                "spawn_bash_process".to_string(),
                format!("{{\"command\":\"{}\"}}", args.command),
            )?;

            let start = std::time::Instant::now();
            let mut accumulated = Vec::new();
            loop {
                let (stdout, _stderr) = running.read_available();
                accumulated.extend(stdout);
                if running.child.try_wait().ok().flatten().is_some() {
                    let (final_out, _) = running.read_available();
                    accumulated.extend(final_out);
                    let out_str = String::from_utf8_lossy(&accumulated).to_string();
                    return Ok(serde_json::Value::String(out_str));
                }
                if start.elapsed() > std::time::Duration::from_secs(30) {
                    let _ = running.child.kill();
                    let out_str = String::from_utf8_lossy(&accumulated).to_string();
                    return Ok(serde_json::Value::String(format!(
                        "{out_str}\n[Execution Timed Out]"
                    )));
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
        other => Err(format!("Unknown tool: {other}")),
    };
    crate::log_host!("[HOST] Core Tool Fallback Result for '{}': {:?}", name, res);
    res
}
