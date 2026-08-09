// `open_process` implementation, split out of `imports_rpc.rs` to stay
// under the 300-line file limit.
//
// The `verify_rpc_exclude` call that stood before the HITL check is gone
// (AWU 970). No guest has imported `open-process` since `mcp-tool-provider`
// was deleted in AWU 965; the import survives only because `wit/rad.wit`
// keeps its existing functions' types unchanged during the migration. The
// removal was made after instrumenting this function to panic on entry and
// running the full suite without it firing — and after confirming the same
// probe on `execute_tool_text` does fail a test, so an unfired probe means
// something.
use crate::wasm::{HostExecution, WasmState, permissions};
use parking_lot::Mutex;

pub(crate) struct MutexWriteWrapper(pub(crate) parking_lot::Mutex<Box<dyn std::io::Write + Send>>);
impl std::io::Write for MutexWriteWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().flush()
    }
}

pub(crate) fn open_process(
    state: &mut WasmState,
    command: String,
) -> Result<wasmtime::component::Resource<HostExecution>, String> {
    use wasmtime_wasi::WasiView;

    let expanded_command = if command.starts_with('~') {
        let mut parts = command.split_whitespace();
        if let Some(first) = parts.next() {
            let expanded_first = crate::config::expand_tilde(first)
                .to_string_lossy()
                .to_string();
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                expanded_first
            } else {
                format!("{expanded_first} {}", rest.join(" "))
            }
        } else {
            command.clone()
        }
    } else {
        command.clone()
    };

    let cmd = rad_models::RasRpcCommand::SpawnBashProcess {
        command: expanded_command.clone(),
    };

    if let Err(ref e) =
        permissions::check_permissions(&cmd, &state.permissions, state.sandbox.workspace_dir())
    {
        crate::log_host!(
            "[HOST] RPC OpenProcess Perm Failed in extension '{}': {}",
            state.name,
            e
        );
        return Err(format!(
            "Permission denied in extension '{}': {e}",
            state.name
        ));
    }

    // HITL check
    if state.hitl_enabled {
        let approved = crate::wasm::rpc_process::ask_human_approval_internal(&format!(
            "Spawn bash process: {expanded_command}"
        ))?;
        if !approved {
            return Err("User rejected execution of tool spawn_bash_process".to_string());
        }
    }

    // Spawn process
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let call_id = format!("wasm_proc_{ts}");
    let mut running = state.process_manager.spawn_bash_process(
        &expanded_command,
        Some(state.sandbox.workspace_dir()),
        call_id,
        "spawn_bash_process".to_string(),
        format!("{{\"command\":\"{expanded_command}\"}}"),
    )?;

    // Extract stdin, stdout, stderr channels
    let stdout_rx = running.stdout_rx.take();
    let stderr_rx = running.stderr_rx.take();
    let stdin_writer = if let Some(stdin_tx) = running.stdin_tx.take() {
        let wrapper = MutexWriteWrapper(stdin_tx);
        Some(Box::new(wrapper) as Box<dyn std::io::Write + Send>)
    } else {
        None
    };

    // Insert into Host ResourceTable
    let host_exec = HostExecution {
        running: Mutex::new(running),
        stdout: Mutex::new(stdout_rx),
        stderr: Mutex::new(stderr_rx),
        stdin: Mutex::new(stdin_writer),
    };
    let res = state.table().push(host_exec).map_err(|e| e.to_string())?;
    Ok(res)
}
