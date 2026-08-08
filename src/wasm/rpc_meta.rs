use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;

#[cfg(test)]
mod tests;

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
            crate::log_host!(
                "\x1b[36m[TRACE {trace_id}]\x1b[0m \x1b[33m[{module}]\x1b[0m {message}"
            );
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
                // Modules first, so a ported provider's tools take the slot its
                // extension used to hold if both happen to be live.
                let kernel_tools = orch
                    .kernel
                    .lock()
                    .clone()
                    .map(|k| crate::kernel::tools::list(&k))
                    .unwrap_or_default();
                let mut all_tools = serde_json::Value::Array(kernel_tools);
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

        // `RasRpcCommand::ExecuteTool` is never dispatched via `host_rpc` in
        // practice — real tool execution goes through the `execute-tool`
        // WIT import directly (`src/wasm/imports_tool.rs`), which is what
        // `rad-orchestrator` (and every extension) actually calls. This
        // variant only still exists because `imports_tool.rs` constructs
        // one to describe the call to `verify_rpc_exclude` for security
        // checks, and `security-guard`'s policy matches on it. Kept here
        // only to keep this match exhaustive.
        RasRpcCommand::ExecuteTool { .. } => Err(
            "ExecuteTool is not dispatched via host_rpc; call the execute-tool WIT import directly"
                .to_string(),
        ),
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
                // Kernel modules answer first. This is the bridge that carries
                // every stage of the migration: the caller — itself still a Wasm
                // extension — keeps issuing the same `CallExtension`, and which
                // surface serves it becomes a deployment question rather than a
                // code change. A module is consulted only if one actually
                // provides the method, so removing it from `modules` falls
                // straight back to the extension.
                //
                // The legacy call names a role and a bare method
                // ("context-tools", "optimize"); module methods are namespaced
                // ("context-tools.optimize"). The mapping is that
                // concatenation and nothing cleverer, so a module wanting the
                // legacy traffic simply provides `<role>.<method>`.
                let kernel = orch.kernel.lock().clone();
                if let Some(kernel) = kernel {
                    let namespaced = format!("{extension_id}.{method}");
                    if kernel.resolve(extension_id, &namespaced).is_some() {
                        let reply = kernel.call("host", extension_id, &namespaced, arguments)?;
                        return Ok(serde_json::Value::String(reply));
                    }
                }

                // `extension_id` selects by declared *role*
                // (`ExtensionConfig.role`), not literal extension name —
                // resolved via `find_extension_arc_by_role`, which reads
                // the role from static config rather than locking every
                // candidate runtime (the nested-lock pattern AWU 900 fixed
                // away from). Callers ask for "whichever extension serves
                // this role" (e.g. "context-tools") rather than a specific
                // deployment's chosen name, so a user can swap in their
                // own compatible implementation under a different name
                // without breaking anything that calls it this way.
                let Some(runtime_arc) = orch.find_extension_arc_by_role(extension_id) else {
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
        RasRpcCommand::GetActiveLlmProfile => {
            let Some(orch) = ctx.orchestrator else {
                return Ok(serde_json::Value::Null);
            };
            Ok(active_llm_profile_json(orch))
        }
        RasRpcCommand::GetExtensionConfig => {
            let Some(orch) = ctx.orchestrator else {
                return Ok(serde_json::Value::Object(serde_json::Map::new()));
            };
            Ok(extension_config_json(orch, ctx.caller_name))
        }
        _ => unreachable!(),
    }
}

/// The active `/llm` profile's resolved fields — including `base_url`/
/// `api_key`, which `active_llm_profile_json` deliberately omits from the
/// generic `GetActiveLlmProfile` RPC response (that's reachable by any
/// extension; a credential doesn't belong in a broadly-readable fact
/// query). Used host-internally by `rpc_meta_llm_connector`, which passes
/// `base_url`/`api_key` as explicit call arguments to the one extension
/// that actually needs them, instead of the old approach of setting
/// process environment variables for a Wasm guest to read — a `WasiCtxBuilder`
/// environment is snapshotted once at instance creation, so that never
/// reliably reached an already-running (e.g. eagerly-loaded-at-startup)
/// instance in the first place.
pub(crate) struct ActiveLlmProfile {
    pub(crate) model: Option<String>,
    pub(crate) context_length: Option<u32>,
    pub(crate) base_url: Option<String>,
    pub(crate) api_key: Option<String>,
    pub(crate) dialect: Option<String>,
}

pub(crate) fn resolve_active_llm_profile(
    orch: &crate::orchestrator::Orchestrator,
) -> ActiveLlmProfile {
    let cfg = orch.config.lock();
    let profile = cfg
        .llm
        .active
        .as_deref()
        .and_then(|name| cfg.llm.endpoints.get(name));
    ActiveLlmProfile {
        model: profile.and_then(|p| p.model.clone()),
        context_length: profile.and_then(|p| p.context_length),
        base_url: profile.map(|p| p.base_url.clone()),
        api_key: profile.and_then(crate::config::LlmEndpointProfile::resolved_api_key),
        dialect: profile.and_then(|p| p.dialect.clone()),
    }
}

/// Reports facts about the active LLM profile — model name and detected
/// context window — with no opinion on how a caller uses them. Extensions
/// that need to reason about context-window budgets (e.g.
/// `rad-orchestrator` sizing a `context-tools.optimize` request) ask this
/// generic question instead of the host special-casing any one extension's
/// request shape. Deliberately excludes `base_url`/`api_key` — see
/// `ActiveLlmProfile`'s docs.
fn active_llm_profile_json(orch: &crate::orchestrator::Orchestrator) -> serde_json::Value {
    let p = resolve_active_llm_profile(orch);
    serde_json::json!({
        "model": p.model,
        "context_length": p.context_length,
    })
}

/// Returns the calling extension's own `config` blob (from
/// `ExtensionConfig.config` in `~/.rad/config.json`) as a JSON object, so an
/// extension can be configured (e.g. `security-guard`'s blocklist patterns)
/// without the host needing to know anything about the shape of that
/// configuration. Empty object if the extension isn't registered or has no
/// configured `config`.
fn extension_config_json(
    orch: &crate::orchestrator::Orchestrator,
    caller_name: &str,
) -> serde_json::Value {
    let cfg = orch.config.lock();
    cfg.extensions
        .iter()
        .find(|e| e.name == caller_name)
        .map(|e| serde_json::Value::Object(e.config.clone().into_iter().collect()))
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()))
}
