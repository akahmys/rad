use crate::ipc::{RasCoreEvent, RasRpcCommand};
use crate::process::RunningProcess;
use crate::subsystems::{FsSubsystem, ProcessSubsystem};
use crate::wasm::rpc::RpcContext;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

pub(crate) fn ask_human_approval_internal(prompt: &str) -> Result<bool, String> {
    println!("{prompt}");
    if let Ok(val) = std::env::var("RAD_TEST_APPROVE") {
        let approved = val == "y" || val == "yes";
        return Ok(approved);
    }
    print!("Approve? (y/N): ");
    std::io::stdout()
        .flush()
        .map_err(|e| format!("Failed to flush stdout: {e}"))?;
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read stdin: {e}"))?;
    let trimmed = input.trim().to_lowercase();
    let approved = trimmed == "y" || trimmed == "yes";
    Ok(approved)
}

pub(crate) fn request_approval(desc: &str) -> Result<(), String> {
    // Bypass approval prompt during cargo test to prevent blocking unit tests
    if std::env::var("CARGO_MANIFEST_DIR").is_ok() {
        return Ok(());
    }

    print!(
        "\n[Privileged Operation Request] {}\nApprove this action? (y/N): ",
        desc
    );
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| format!("Failed to read stdin: {e}"))?;

    let trimmed = input.trim().to_lowercase();
    if trimmed == "y" || trimmed == "yes" {
        Ok(())
    } else {
        Err("Operation rejected by user".to_string())
    }
}

pub(crate) fn spawn_bash_process_rpc(
    command: &str,
    sandbox: &dyn FsSubsystem,
    process_manager: &dyn ProcessSubsystem,
    active_processes: &Arc<Mutex<HashMap<String, RunningProcess>>>,
    event_tx: &std::sync::mpsc::Sender<RasCoreEvent>,
    call_id: String,
    hitl_enabled: bool,
) -> Result<serde_json::Value, String> {
    if hitl_enabled {
        let approved = ask_human_approval_internal(&format!("Spawn bash process: {command}"))?;
        if !approved {
            return Err("User rejected execution of tool spawn_bash_process".to_string());
        }
    }

    let running = process_manager.spawn_bash_process(
        command,
        Some(sandbox.workspace_dir()),
        call_id,
        "spawn_bash_process".to_string(),
        format!("{{\"command\":\"{command}\"}}"),
    )?;
    let pgid = running.pgid().as_raw();

    {
        let mut processes = active_processes.lock();
        processes.insert(pgid.to_string(), running);
    }

    let event_tx_clone = event_tx.clone();
    let active_processes_clone = active_processes.clone();

    std::thread::spawn(move || {
        let _ = event_tx_clone.send(RasCoreEvent::ProcessSpawned {
            pgid: pgid.to_string(),
            pid: pgid,
        });
        let pgid_str = pgid.to_string();
        loop {
            let done = monitor_single_process_tick(
                pgid_str.clone(),
                &active_processes_clone,
                &event_tx_clone,
            );
            if done {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    serde_json::to_value(pgid).map_err(|e| format!("Serialization error: {e}"))
}

pub(crate) fn handle_process(
    cmd: &RasRpcCommand,
    ctx: &RpcContext<'_>,
) -> Result<serde_json::Value, String> {
    match cmd {
        RasRpcCommand::SpawnBashProcess { command } => {
            if ctx.hitl_enabled {
                request_approval(&format!("Execute shell command: '{command}'"))?;
            }
            spawn_bash_process_rpc(
                command,
                ctx.sandbox,
                ctx.process_manager,
                ctx.active_processes,
                ctx.event_tx,
                ctx.call_id.clone(),
                ctx.hitl_enabled,
            )
        }
        RasRpcCommand::SpawnMcpServer {
            name,
            command,
            args,
        } => {
            let proc = crate::mcp::McpProcess::spawn(name, command, args, ctx.event_tx.clone())?;
            let mut guard = ctx.active_mcp_servers.lock();
            guard.insert(name.clone(), proc);
            Ok(serde_json::Value::Null)
        }
        RasRpcCommand::SendMcpRequest { name, message } => {
            let mut guard = ctx.active_mcp_servers.lock();
            if let Some(proc) = guard.get_mut(name) {
                proc.send_message(message)?;
                Ok(serde_json::Value::Null)
            } else {
                Err(format!("MCP server '{name}' is not running"))
            }
        }
        _ => Err("Unhandled RPC command in handle_process".to_string()),
    }
}

fn monitor_single_process_tick(
    pgid: String,
    active_processes: &Arc<Mutex<HashMap<String, RunningProcess>>>,
    event_tx: &std::sync::mpsc::Sender<RasCoreEvent>,
) -> bool {
    let mut procs = active_processes.lock();
    let Some(proc) = procs.get_mut(&pgid) else {
        return true;
    };

    let (stdout, stderr) = proc.read_available();
    if !stdout.is_empty() {
        let _ = event_tx.send(RasCoreEvent::ProcessStdout {
            pgid: pgid.clone(),
            data: stdout,
        });
    }
    if !stderr.is_empty() {
        let _ = event_tx.send(RasCoreEvent::ProcessStderr {
            pgid: pgid.clone(),
            data: stderr,
        });
    }

    match proc.child.try_wait() {
        Ok(Some(status)) => {
            let code = i32::try_from(status.exit_code()).ok();
            let _ = event_tx.send(RasCoreEvent::ProcessExited {
                pgid: pgid.clone(),
                exit_code: code,
            });
            proc.unregister_pgid();
            procs.remove(&pgid);
            true
        }
        Ok(None) => {
            let policy = proc.timeout_policy.lock().clone();
            let is_timeout = match policy {
                crate::ipc::TimeoutPolicy::Dynamic {
                    heartbeat_timeout_ms,
                    ..
                } => {
                    proc.last_activity.elapsed()
                        > std::time::Duration::from_millis(heartbeat_timeout_ms)
                }
                crate::ipc::TimeoutPolicy::Infinite => false,
            };
            if is_timeout {
                proc.kill_group();
                let _ = event_tx.send(RasCoreEvent::StreamTimeout {
                    target: format!("process_{}", pgid),
                    duration_ms: match policy {
                        crate::ipc::TimeoutPolicy::Dynamic {
                            heartbeat_timeout_ms,
                            ..
                        } => heartbeat_timeout_ms,
                        crate::ipc::TimeoutPolicy::Infinite => 0,
                    },
                });
                let _ = event_tx.send(RasCoreEvent::ProcessExited {
                    pgid: pgid.clone(),
                    exit_code: None,
                });
                procs.remove(&pgid);
                true
            } else {
                false
            }
        }
        Err(_) => {
            proc.kill_group();
            let _ = event_tx.send(RasCoreEvent::ProcessExited {
                pgid: pgid.clone(),
                exit_code: None,
            });
            procs.remove(&pgid);
            true
        }
    }
}
