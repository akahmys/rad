// Built-in tool fallback (`read`/`write`/`edit`/`bash`) for `ExecuteTool`
// requests that no registered MCP tool provider handled, split out of
// `rpc_meta.rs` to stay under the 300-line file limit.
use crate::ipc::RasRpcCommand;
use crate::wasm::rpc::RpcContext;

pub(crate) fn execute_core_tool_fallback(
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
        "bash" | "spawn_bash_process" => {
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
        other => Err(format!(
            "Tool '{other}' is not a valid built-in tool and no registered MCP tool provider handled it"
        )),
    };
    crate::log_host!("[HOST] Core Tool Fallback Result for '{}': {:?}", name, res);
    res
}
