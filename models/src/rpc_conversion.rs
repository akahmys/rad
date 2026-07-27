//! Shared `macro_rules!` definitions that generate the WIT ↔ `RasRpcCommand`
//! (and `Target`/`TimeoutPolicy`) conversion boilerplate, so each of the 4
//! crates that previously hand-duplicated this match (`rad` host bindings,
//! `security-guard`, `mcp-tool-provider`, `rad-orchestrator`) invokes one
//! shared definition instead of hand-copying ~25 match arms per direction.
//!
//! Each crate's `wit_bindgen::generate!` produces its own local `RasRpcCommand`
//! type (structurally identical, since all worlds import the same `rad.wit`
//! `types` interface, but nominally distinct per crate) — hence these are
//! declarative macros taking the invoking crate's `wit` module path as a
//! parameter, rather than a single `impl` written once here.
//!
//! `core-to-wit` is split from `wit-to-core`: `RasRpcCommand::OpenFile`/
//! `OpenProcess` exist only on the core side (normalized into `FileRead`/
//! `FileWrite`/`SpawnBashProcess` before ever reaching a guest), so no
//! guest-side `wit` variant exists for them. The host converts them for
//! real; guests never construct them and are expected to handle that
//! residual themselves (typically `unreachable!()`/`panic!()`), so the
//! core-to-wit macro returns `Option` and leaves that residual to the caller.

/// Generates `From<$wit::Target> for $crate::Target`.
#[macro_export]
macro_rules! impl_rpc_target_wit_to_core {
    ($wit:ident) => {
        impl From<$wit::Target> for $crate::Target {
            fn from(t: $wit::Target) -> Self {
                match t {
                    $wit::Target::Llm => $crate::Target::Llm,
                    $wit::Target::Process(p) => $crate::Target::Process(p.to_string()),
                }
            }
        }
    };
}

/// Generates `From<$crate::Target> for $wit::Target`.
#[macro_export]
macro_rules! impl_rpc_target_core_to_wit {
    ($wit:ident) => {
        impl From<$crate::Target> for $wit::Target {
            fn from(t: $crate::Target) -> Self {
                match t {
                    $crate::Target::Llm => $wit::Target::Llm,
                    $crate::Target::Process(p) => $wit::Target::Process(p.parse().unwrap_or(0)),
                }
            }
        }
    };
}

/// Generates `From<$wit::TimeoutPolicy> for $crate::TimeoutPolicy`.
#[macro_export]
macro_rules! impl_rpc_timeout_policy_wit_to_core {
    ($wit:ident) => {
        impl From<$wit::TimeoutPolicy> for $crate::TimeoutPolicy {
            fn from(tp: $wit::TimeoutPolicy) -> Self {
                match tp {
                    $wit::TimeoutPolicy::Dynamic(p) => $crate::TimeoutPolicy::Dynamic {
                        heartbeat_timeout_ms: p.heartbeat_timeout_ms,
                        max_silent_wait_ms: p.max_silent_wait_ms,
                    },
                    $wit::TimeoutPolicy::Infinite => $crate::TimeoutPolicy::Infinite,
                }
            }
        }
    };
}

/// Generates `From<$crate::TimeoutPolicy> for $wit::TimeoutPolicy`.
#[macro_export]
macro_rules! impl_rpc_timeout_policy_core_to_wit {
    ($wit:ident) => {
        impl From<$crate::TimeoutPolicy> for $wit::TimeoutPolicy {
            fn from(tp: $crate::TimeoutPolicy) -> Self {
                match tp {
                    $crate::TimeoutPolicy::Dynamic {
                        heartbeat_timeout_ms,
                        max_silent_wait_ms,
                    } => $wit::TimeoutPolicy::Dynamic($wit::DynamicPolicy {
                        heartbeat_timeout_ms,
                        max_silent_wait_ms,
                    }),
                    $crate::TimeoutPolicy::Infinite => $wit::TimeoutPolicy::Infinite,
                }
            }
        }
    };
}

/// Generates the full, exhaustive `From<$wit::RasRpcCommand> for
/// $crate::RasRpcCommand`. Uniform across all 4 sites — no per-crate
/// residual needed in this direction. Requires `impl_rpc_target_wit_to_core!`
/// and `impl_rpc_timeout_policy_wit_to_core!` to also be invoked somewhere in
/// the same crate (item order doesn't matter, just presence).
#[macro_export]
macro_rules! impl_rpc_command_wit_to_core {
    ($wit:ident) => {
        impl From<$wit::RasRpcCommand> for $crate::RasRpcCommand {
            fn from(cmd: $wit::RasRpcCommand) -> Self {
                match cmd {
                    $wit::RasRpcCommand::FileRead(path) => {
                        $crate::RasRpcCommand::FileRead { path: std::path::PathBuf::from(path) }
                    }
                    $wit::RasRpcCommand::FileWrite(payload) => $crate::RasRpcCommand::FileWrite {
                        path: std::path::PathBuf::from(payload.path),
                        data: payload.data,
                    },
                    $wit::RasRpcCommand::FileEditPatch(payload) => {
                        $crate::RasRpcCommand::FileEditPatch {
                            path: std::path::PathBuf::from(payload.path),
                            diff: payload.diff,
                        }
                    }
                    $wit::RasRpcCommand::SpawnBashProcess(cmd_str) => {
                        $crate::RasRpcCommand::SpawnBashProcess { command: cmd_str }
                    }
                    $wit::RasRpcCommand::CreateNode(payload) => $crate::RasRpcCommand::CreateNode {
                        parent_id: payload.parent_id,
                        node_type: payload.node_type,
                    },
                    $wit::RasRpcCommand::SetNodeText(payload) => {
                        $crate::RasRpcCommand::SetNodeText {
                            node_id: payload.node_id,
                            text: payload.text,
                        }
                    }
                    $wit::RasRpcCommand::MergeNodes(payload) => $crate::RasRpcCommand::MergeNodes {
                        node_ids: payload.node_ids,
                        summary_text: payload.summary_text,
                    },
                    $wit::RasRpcCommand::DeleteNode(node_id) => {
                        $crate::RasRpcCommand::DeleteNode { node_id }
                    }
                    $wit::RasRpcCommand::TakeSnapshot(payload) => {
                        $crate::RasRpcCommand::TakeSnapshot {
                            node_id: payload.node_id,
                            target_paths: payload
                                .target_paths
                                .into_iter()
                                .map(std::path::PathBuf::from)
                                .collect(),
                        }
                    }
                    $wit::RasRpcCommand::CheckoutSnapshot(node_id) => {
                        $crate::RasRpcCommand::CheckoutSnapshot { node_id }
                    }
                    $wit::RasRpcCommand::OpenHttpStream(payload) => {
                        $crate::RasRpcCommand::OpenHttpStream {
                            url: payload.url,
                            headers: payload.headers.into_iter().collect(),
                            body: payload.body,
                        }
                    }
                    $wit::RasRpcCommand::SetStreamTimeoutPolicy(payload) => {
                        $crate::RasRpcCommand::SetStreamTimeoutPolicy {
                            target: $crate::Target::from(payload.target),
                            policy: $crate::TimeoutPolicy::from(payload.policy),
                        }
                    }
                    $wit::RasRpcCommand::WriteStdout(text) => {
                        $crate::RasRpcCommand::WriteStdout { text }
                    }
                    $wit::RasRpcCommand::CompleteTask => $crate::RasRpcCommand::CompleteTask,
                    $wit::RasRpcCommand::GetDag => $crate::RasRpcCommand::GetDag,
                    $wit::RasRpcCommand::GetActiveLlmProfile => {
                        $crate::RasRpcCommand::GetActiveLlmProfile
                    }
                    $wit::RasRpcCommand::GetExtensionConfig => {
                        $crate::RasRpcCommand::GetExtensionConfig
                    }
                    $wit::RasRpcCommand::AskHumanApproval(prompt) => {
                        $crate::RasRpcCommand::AskHumanApproval { prompt }
                    }
                    $wit::RasRpcCommand::ReportTokenUsage(payload) => {
                        $crate::RasRpcCommand::ReportTokenUsage {
                            prompt_tokens: payload.prompt_tokens,
                            completion_tokens: payload.completion_tokens,
                        }
                    }
                    $wit::RasRpcCommand::SpawnMcpServer(payload) => {
                        $crate::RasRpcCommand::SpawnMcpServer {
                            name: payload.name,
                            command: payload.command,
                            args: payload.args,
                        }
                    }
                    $wit::RasRpcCommand::SendMcpRequest(payload) => {
                        $crate::RasRpcCommand::SendMcpRequest {
                            name: payload.name,
                            message: payload.message,
                        }
                    }
                    $wit::RasRpcCommand::GetRepoMap => $crate::RasRpcCommand::GetRepoMap,
                    $wit::RasRpcCommand::GetTools => $crate::RasRpcCommand::GetTools,
                    $wit::RasRpcCommand::ExecuteTool(payload) => {
                        $crate::RasRpcCommand::ExecuteTool {
                            call_id: payload.call_id,
                            name: payload.name,
                            arguments: payload.arguments,
                        }
                    }
                    $wit::RasRpcCommand::GenerateLlmStream(payload) => {
                        $crate::RasRpcCommand::GenerateLlmStream {
                            model: payload.model,
                            messages_json: payload.messages_json,
                            tools_json: payload.tools_json,
                        }
                    }
                    $wit::RasRpcCommand::CallExtension(payload) => {
                        $crate::RasRpcCommand::CallExtension {
                            extension_id: payload.extension_id,
                            method: payload.method,
                            arguments: payload.arguments,
                        }
                    }
                    $wit::RasRpcCommand::LogTracedEvent(payload) => {
                        $crate::RasRpcCommand::LogTracedEvent {
                            trace_id: payload.trace_id,
                            module: payload.module,
                            message: payload.message,
                        }
                    }
                }
            }
        }
    };
}

/// Expression-position macro converting everything EXCEPT `OpenFile`/
/// `OpenProcess` (which have no `wit` variant — see module docs). Returns
/// `Some(wit::RasRpcCommand)` for every other `$crate::RasRpcCommand`
/// variant, `None` for `OpenFile`/`OpenProcess` so the caller supplies its
/// own residual handling (the host converts them for real; guests never
/// construct them and typically `unreachable!()`/`panic!()`).
#[macro_export]
macro_rules! common_rpc_command_core_to_wit {
    ($wit:ident, $cmd:expr) => {
        match $cmd {
            $crate::RasRpcCommand::FileRead { path } => {
                Some($wit::RasRpcCommand::FileRead(path.to_string_lossy().into_owned()))
            }
            $crate::RasRpcCommand::FileWrite { path, data } => {
                Some($wit::RasRpcCommand::FileWrite($wit::FileWritePayload {
                    path: path.to_string_lossy().into_owned(),
                    data,
                }))
            }
            $crate::RasRpcCommand::FileEditPatch { path, diff } => {
                Some($wit::RasRpcCommand::FileEditPatch($wit::FilePatchPayload {
                    path: path.to_string_lossy().into_owned(),
                    diff,
                }))
            }
            $crate::RasRpcCommand::SpawnBashProcess { command } => {
                Some($wit::RasRpcCommand::SpawnBashProcess(command))
            }
            $crate::RasRpcCommand::CreateNode { parent_id, node_type } => {
                Some($wit::RasRpcCommand::CreateNode($wit::CreateNodePayload {
                    parent_id,
                    node_type,
                }))
            }
            $crate::RasRpcCommand::SetNodeText { node_id, text } => {
                Some($wit::RasRpcCommand::SetNodeText($wit::SetNodeTextPayload { node_id, text }))
            }
            $crate::RasRpcCommand::MergeNodes { node_ids, summary_text } => {
                Some($wit::RasRpcCommand::MergeNodes($wit::MergeNodesPayload {
                    node_ids,
                    summary_text,
                }))
            }
            $crate::RasRpcCommand::DeleteNode { node_id } => {
                Some($wit::RasRpcCommand::DeleteNode(node_id))
            }
            $crate::RasRpcCommand::TakeSnapshot { node_id, target_paths } => {
                Some($wit::RasRpcCommand::TakeSnapshot($wit::TakeSnapshotPayload {
                    node_id,
                    target_paths: target_paths
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect(),
                }))
            }
            $crate::RasRpcCommand::CheckoutSnapshot { node_id } => {
                Some($wit::RasRpcCommand::CheckoutSnapshot(node_id))
            }
            $crate::RasRpcCommand::OpenHttpStream { url, headers, body } => {
                Some($wit::RasRpcCommand::OpenHttpStream($wit::OpenHttpStreamPayload {
                    url,
                    headers: headers.into_iter().collect(),
                    body,
                }))
            }
            $crate::RasRpcCommand::SetStreamTimeoutPolicy { target, policy } => {
                Some($wit::RasRpcCommand::SetStreamTimeoutPolicy(
                    $wit::SetStreamTimeoutPolicyPayload {
                        target: $wit::Target::from(target),
                        policy: $wit::TimeoutPolicy::from(policy),
                    },
                ))
            }
            $crate::RasRpcCommand::WriteStdout { text } => {
                Some($wit::RasRpcCommand::WriteStdout(text))
            }
            $crate::RasRpcCommand::CompleteTask => Some($wit::RasRpcCommand::CompleteTask),
            $crate::RasRpcCommand::GetDag => Some($wit::RasRpcCommand::GetDag),
            $crate::RasRpcCommand::GetActiveLlmProfile => {
                Some($wit::RasRpcCommand::GetActiveLlmProfile)
            }
            $crate::RasRpcCommand::GetExtensionConfig => {
                Some($wit::RasRpcCommand::GetExtensionConfig)
            }
            $crate::RasRpcCommand::AskHumanApproval { prompt } => {
                Some($wit::RasRpcCommand::AskHumanApproval(prompt))
            }
            $crate::RasRpcCommand::ReportTokenUsage { prompt_tokens, completion_tokens } => {
                Some($wit::RasRpcCommand::ReportTokenUsage($wit::ReportTokenUsagePayload {
                    prompt_tokens,
                    completion_tokens,
                }))
            }
            $crate::RasRpcCommand::SpawnMcpServer { name, command, args } => {
                Some($wit::RasRpcCommand::SpawnMcpServer($wit::SpawnMcpServerPayload {
                    name,
                    command,
                    args,
                }))
            }
            $crate::RasRpcCommand::SendMcpRequest { name, message } => {
                Some($wit::RasRpcCommand::SendMcpRequest($wit::SendMcpRequestPayload {
                    name,
                    message,
                }))
            }
            $crate::RasRpcCommand::GetRepoMap => Some($wit::RasRpcCommand::GetRepoMap),
            $crate::RasRpcCommand::GetTools => Some($wit::RasRpcCommand::GetTools),
            $crate::RasRpcCommand::ExecuteTool { call_id, name, arguments } => {
                Some($wit::RasRpcCommand::ExecuteTool($wit::ExecuteToolPayload {
                    call_id,
                    name,
                    arguments,
                }))
            }
            $crate::RasRpcCommand::GenerateLlmStream { model, messages_json, tools_json } => {
                Some($wit::RasRpcCommand::GenerateLlmStream($wit::GenerateLlmStreamPayload {
                    model,
                    messages_json,
                    tools_json,
                }))
            }
            $crate::RasRpcCommand::CallExtension { extension_id, method, arguments } => {
                Some($wit::RasRpcCommand::CallExtension($wit::CallExtensionPayload {
                    extension_id,
                    method,
                    arguments,
                }))
            }
            $crate::RasRpcCommand::LogTracedEvent { trace_id, module, message } => {
                Some($wit::RasRpcCommand::LogTracedEvent($wit::LogTracedEventPayload {
                    trace_id,
                    module,
                    message,
                }))
            }
            $crate::RasRpcCommand::OpenFile { .. } | $crate::RasRpcCommand::OpenProcess { .. } => {
                None
            }
        }
    };
}
