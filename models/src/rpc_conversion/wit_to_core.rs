// `impl_rpc_command_wit_to_core!`, split out of `rpc_conversion.rs` to stay
// under the 300-line file limit. `#[macro_export]` always exports at the
// crate root regardless of which module a macro is textually defined in,
// so this split has no effect on how callers invoke it
// (`rad_models::impl_rpc_command_wit_to_core!(wit)`).

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
                    $wit::RasRpcCommand::FileRead(path) => $crate::RasRpcCommand::FileRead {
                        path: std::path::PathBuf::from(path),
                    },
                    $wit::RasRpcCommand::ListDir(path) => $crate::RasRpcCommand::ListDir {
                        path: std::path::PathBuf::from(path),
                    },
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
