// `common_rpc_command_core_to_wit!`, split out of `rpc_conversion.rs` to
// stay under the 300-line file limit. `#[macro_export]` always exports at
// the crate root regardless of which module a macro is textually defined
// in, so this split has no effect on how callers invoke it
// (`rad_models::common_rpc_command_core_to_wit!(wit, cmd)`).

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
            $crate::RasRpcCommand::FileRead { path } => Some($wit::RasRpcCommand::FileRead(
                path.to_string_lossy().into_owned(),
            )),
            $crate::RasRpcCommand::ListDir { path } => Some($wit::RasRpcCommand::ListDir(
                path.to_string_lossy().into_owned(),
            )),
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
            $crate::RasRpcCommand::CreateNode {
                parent_id,
                node_type,
            } => Some($wit::RasRpcCommand::CreateNode($wit::CreateNodePayload {
                parent_id,
                node_type,
            })),
            $crate::RasRpcCommand::SetNodeText { node_id, text } => {
                Some($wit::RasRpcCommand::SetNodeText($wit::SetNodeTextPayload {
                    node_id,
                    text,
                }))
            }
            $crate::RasRpcCommand::MergeNodes {
                node_ids,
                summary_text,
            } => Some($wit::RasRpcCommand::MergeNodes($wit::MergeNodesPayload {
                node_ids,
                summary_text,
            })),
            $crate::RasRpcCommand::DeleteNode { node_id } => {
                Some($wit::RasRpcCommand::DeleteNode(node_id))
            }
            $crate::RasRpcCommand::TakeSnapshot {
                node_id,
                target_paths,
            } => Some($wit::RasRpcCommand::TakeSnapshot(
                $wit::TakeSnapshotPayload {
                    node_id,
                    target_paths: target_paths
                        .into_iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect(),
                },
            )),
            $crate::RasRpcCommand::CheckoutSnapshot { node_id } => {
                Some($wit::RasRpcCommand::CheckoutSnapshot(node_id))
            }
            $crate::RasRpcCommand::OpenHttpStream { url, headers, body } => Some(
                $wit::RasRpcCommand::OpenHttpStream($wit::OpenHttpStreamPayload {
                    url,
                    headers: headers.into_iter().collect(),
                    body,
                }),
            ),
            $crate::RasRpcCommand::SetStreamTimeoutPolicy { target, policy } => Some(
                $wit::RasRpcCommand::SetStreamTimeoutPolicy($wit::SetStreamTimeoutPolicyPayload {
                    target: $wit::Target::from(target),
                    policy: $wit::TimeoutPolicy::from(policy),
                }),
            ),
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
            $crate::RasRpcCommand::ReportTokenUsage {
                prompt_tokens,
                completion_tokens,
            } => Some($wit::RasRpcCommand::ReportTokenUsage(
                $wit::ReportTokenUsagePayload {
                    prompt_tokens,
                    completion_tokens,
                },
            )),
            $crate::RasRpcCommand::GetRepoMap => Some($wit::RasRpcCommand::GetRepoMap),
            $crate::RasRpcCommand::GetTools => Some($wit::RasRpcCommand::GetTools),
            $crate::RasRpcCommand::ExecuteTool {
                call_id,
                name,
                arguments,
            } => Some($wit::RasRpcCommand::ExecuteTool($wit::ExecuteToolPayload {
                call_id,
                name,
                arguments,
            })),
            $crate::RasRpcCommand::GenerateLlmStream {
                model,
                messages_json,
                tools_json,
            } => Some($wit::RasRpcCommand::GenerateLlmStream(
                $wit::GenerateLlmStreamPayload {
                    model,
                    messages_json,
                    tools_json,
                },
            )),
            $crate::RasRpcCommand::CallExtension {
                extension_id,
                method,
                arguments,
            } => Some($wit::RasRpcCommand::CallExtension(
                $wit::CallExtensionPayload {
                    extension_id,
                    method,
                    arguments,
                },
            )),
            $crate::RasRpcCommand::LogTracedEvent {
                trace_id,
                module,
                message,
            } => Some($wit::RasRpcCommand::LogTracedEvent(
                $wit::LogTracedEventPayload {
                    trace_id,
                    module,
                    message,
                },
            )),
            $crate::RasRpcCommand::OpenFile { .. } | $crate::RasRpcCommand::OpenProcess { .. } => {
                None
            }
        }
    };
}
