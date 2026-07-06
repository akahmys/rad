use std::collections::{HashMap, hash_map::RandomState};
use std::sync::{Arc, Mutex};
use std::io::Write;
use crate::ipc::RasCoreEvent;
use crate::process::RunningProcess;
use crate::subsystems::{FsSubsystem, ProcessSubsystem};

pub(crate) fn ask_human_approval_internal(prompt: &str) -> Result<bool, String> {
    println!("{prompt}");
    if let Ok(val) = std::env::var("RAD_TEST_APPROVE") {
        let approved = val == "y" || val == "yes";
        return Ok(approved);
    }
    print!("Approve? (y/N): ");
    std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {e}"))?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).map_err(|e| format!("Failed to read stdin: {e}"))?;
    let trimmed = input.trim().to_lowercase();
    let approved = trimmed == "y" || trimmed == "yes";
    Ok(approved)
}

pub(crate) fn spawn_bash_process_rpc(
    command: &str,
    sandbox: &dyn FsSubsystem,
    process_manager: &dyn ProcessSubsystem,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
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

    let mut processes = active_processes.lock().map_err(|e| format!("Process lock error: {e}"))?;
    processes.insert(pgid, running);

    let event_tx_clone = event_tx.clone();
    let active_processes_clone = active_processes.clone();

    std::thread::spawn(move || {
        let _ = event_tx_clone.send(RasCoreEvent::ProcessSpawned { pgid, pid: pgid });
        loop {
            let done = monitor_single_process_tick(pgid, &active_processes_clone, &event_tx_clone);
            if done {
                if let Ok(mut procs) = active_processes_clone.lock() {
                    procs.remove(&pgid);
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    serde_json::to_value(pgid).map_err(|e| format!("Serialization error: {e}"))
}

fn monitor_single_process_tick(
    pgid: i32,
    active_processes: &Arc<Mutex<HashMap<i32, RunningProcess, RandomState>>>,
    event_tx: &std::sync::mpsc::Sender<RasCoreEvent>,
) -> bool {
    let Ok(mut procs) = active_processes.lock() else { return true };
    let Some(proc) = procs.get_mut(&pgid) else { return true };

    let (stdout, stderr) = proc.read_available();
    if !stdout.is_empty() {
        let _ = event_tx.send(RasCoreEvent::ProcessStdout { pgid, data: stdout });
    }
    if !stderr.is_empty() {
        let _ = event_tx.send(RasCoreEvent::ProcessStderr { pgid, data: stderr });
    }

    match proc.child.try_wait() {
        Ok(Some(status)) => {
            let code = i32::try_from(status.exit_code()).ok();
            let _ = event_tx.send(RasCoreEvent::ProcessExited { pgid, exit_code: code });
            proc.unregister_pgid();
            true
        }
        Ok(None) => {
            let policy = proc.timeout_policy.lock().map(|g| g.clone()).unwrap_or(crate::ipc::TimeoutPolicy::Infinite);
            let is_timeout = match policy {
                crate::ipc::TimeoutPolicy::Dynamic { heartbeat_timeout_ms, .. } => {
                    proc.last_activity.elapsed() > std::time::Duration::from_millis(heartbeat_timeout_ms)
                }
                crate::ipc::TimeoutPolicy::Infinite => false,
            };
            if is_timeout {
                proc.kill_group();
                let _ = event_tx.send(RasCoreEvent::StreamTimeout {
                    target: format!("process_{pgid}"),
                    duration_ms: match policy {
                        crate::ipc::TimeoutPolicy::Dynamic { heartbeat_timeout_ms, .. } => heartbeat_timeout_ms,
                        crate::ipc::TimeoutPolicy::Infinite => 0,
                    },
                });
                let _ = event_tx.send(RasCoreEvent::ProcessExited { pgid, exit_code: None });
                true
            } else {
                false
            }
        }
        Err(_) => {
            proc.kill_group();
            let _ = event_tx.send(RasCoreEvent::ProcessExited { pgid, exit_code: None });
            true
        }
    }
}
