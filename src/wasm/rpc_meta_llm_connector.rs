// `GenerateLlmStream` handling via a loaded `llm-connector` Wasm extension,
// split out of `rpc_meta.rs` to stay under the 300-line file limit. Used
// when an orchestrator (and therefore a Wasm runtime registry) is present.
use crate::wasm::rpc::RpcContext;

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

pub(crate) fn generate(
    model: &str,
    messages_json: &str,
    tools_json: &str,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    let Some(orch) = ctx.orchestrator else {
        return Err("Orchestrator unavailable".to_string());
    };

    crate::log_host!("[DEBUG] Host GenerateLlmStream starting...");
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
        connector_runtime_opt
            .ok_or_else(|| "LLM Connector extension not found or not loaded".to_string())?
    };
    crate::log_host!("[DEBUG] Host found connector_arc.");

    let mut connector = connector_arc.lock();
    let connector_ref = &mut *connector;

    let conn_bindings = connector_ref
        .llm_connector
        .as_ref()
        .ok_or_else(|| "LLM Connector bindings missing".to_string())?;

    let remote_messages: Vec<RemoteMessage> = serde_json::from_str(messages_json)
        .map_err(|e| format!("Failed to parse messages JSON: {e}"))?;

    let remote_tools: Vec<RemoteTool> =
        serde_json::from_str(tools_json).map_err(|e| format!("Failed to parse tools JSON: {e}"))?;

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

    let active_profile = super::rpc_meta::resolve_active_llm_profile(orch);
    let stream_res = match conn_bindings
        .radcomp_connector_producer()
        .call_generate_stream(
            &mut connector_ref.store,
            model,
            active_profile.base_url.as_deref(),
            active_profile.api_key.as_deref(),
            active_profile.dialect.as_deref(),
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
                let Some(conn_bindings) = connector_ref.llm_connector.as_ref() else {
                    let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                        event: serde_json::json!({
                            "type": "error",
                            "payload": "LLM Connector bindings missing while polling stream"
                        })
                        .to_string(),
                    });
                    break;
                };

                conn_bindings
                    .radcomp_connector_producer()
                    .event_stream()
                    .call_read(store, resource_any)
            };

            match read_res {
                Ok(Ok(Some(event))) => {
                    if let Ok(event_json) = serde_json::to_string(&event) {
                        let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                            event: event_json,
                        });
                    }
                }
                Ok(Ok(None)) => {
                    let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                        event: serde_json::json!({ "type": "done" }).to_string(),
                    });
                    break;
                }
                Ok(Err(e)) => {
                    let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                        event: serde_json::json!({ "type": "error", "payload": e }).to_string(),
                    });
                    break;
                }
                Err(e) => {
                    let _ = event_tx_clone.send(crate::ipc::RasCoreEvent::LlmConnectorEvent {
                        event: serde_json::json!({ "type": "error", "payload": e.to_string() })
                            .to_string(),
                    });
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    });

    Ok(serde_json::Value::Null)
}
