use crate::process_child::{StdioChild, spawn_reader_thread};
use crate::sys::Pid;
use parking_lot::Mutex;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Instant;

// `StdioChild`/`ChildKiller` glue and the reader-thread helper live in
// `process_child.rs`; `RunningProcess` lives in `process_running.rs` (both
// re-exported below) to keep this file under the 300-line limit.
pub use crate::process_running::RunningProcess;

#[cfg(test)]
mod tests;

pub struct ProcessManager {
    active_pgids: Arc<Mutex<Vec<Pid>>>,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            active_pgids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawns a bash process within its own process group using clean Stdio Pipe I/O.
    ///
    /// # Errors
    /// Returns an error if the process fails to spawn.
    pub fn spawn_bash_process(
        &self,
        command: &str,
        cwd: Option<&Path>,
        call_id: String,
        name: String,
        arguments: String,
    ) -> Result<RunningProcess, crate::error::UnifiedError> {
        self.spawn_std_pipe_process(command, cwd, call_id, name, arguments)
    }

    fn spawn_std_pipe_process(
        &self,
        command: &str,
        cwd: Option<&Path>,
        call_id: String,
        name: String,
        arguments: String,
    ) -> Result<RunningProcess, crate::error::UnifiedError> {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};

        let expanded = crate::config::expand_tilde(command);
        let parts = expanded
            .to_string_lossy()
            .split_whitespace()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        let first_bin = parts.first().cloned().unwrap_or_default();
        let bin_path = std::path::PathBuf::from(&first_bin);
        let is_in_path = std::env::var_os("PATH").is_some_and(|paths| {
            std::env::split_paths(&paths).any(|p| p.join(&first_bin).is_file())
        });
        // The direct-exec fast path splits arguments on plain whitespace,
        // which silently mangles any quoted argument (`echo -n 'a b'`
        // would run as three literal args `'a`, `b`, `b'`, emitting the
        // quotes verbatim). Quotes therefore need a real shell just as
        // much as redirection does — this matters because tool providers
        // return their results through `open_process("echo -n '<result>'")`,
        // so treating quotes as inert corrupted every tool result with
        // spurious surrounding quotes.
        let has_shell_features = command.contains(';')
            || command.contains('|')
            || command.contains('>')
            || command.contains('<')
            || command.contains('&')
            || command.contains('\'')
            || command.contains('"');
        let is_direct_executable = (bin_path.is_file() || is_in_path) && !has_shell_features;

        let mut cmd = if is_direct_executable {
            let mut c = Command::new(&first_bin);
            if parts.len() > 1 {
                c.args(&parts[1..]);
            }
            c
        } else {
            let mut c = Command::new("bash");
            c.arg("-c").arg(command);
            c
        };

        if let Some(p) = cwd {
            cmd.current_dir(p);
        }
        cmd.process_group(0);
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            crate::error::UnifiedError::l1(
                format!("Failed to spawn command via stdio pipe: {e}"),
                "Process",
            )
        })?;

        let pid_raw = i32::try_from(child.id()).unwrap_or(0);
        let pid = Pid::from_raw(pid_raw);
        {
            let mut pgids = self.active_pgids.lock();
            pgids.push(pid);
        }

        let (stdout_tx, stdout_rx) = mpsc::channel();
        let (stderr_tx, stderr_rx) = mpsc::channel();

        if let Some(stdout) = child.stdout.take() {
            spawn_reader_thread(stdout, stdout_tx);
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_reader_thread(stderr, stderr_tx);
        }

        let stdin_writer: Option<Mutex<Box<dyn std::io::Write + Send>>> = child
            .stdin
            .take()
            .map(|sin| Mutex::new(Box::new(sin) as Box<dyn std::io::Write + Send>));

        Ok(RunningProcess {
            child: Box::new(StdioChild { child }),
            pgid: pid,
            stdout_rx: Some(stdout_rx),
            stderr_rx: Some(stderr_rx),
            stdin_tx: stdin_writer,
            last_activity: Instant::now(),
            active_pgids: self.active_pgids.clone(),
            timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Infinite)),
            call_id,
            name,
            arguments,
        })
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        let pgids = self.active_pgids.lock();
        for pgid in pgids.iter() {
            let _ = crate::sys::kill_process_group(*pgid);
        }
    }
}

impl crate::subsystems::ProcessSubsystem for ProcessManager {
    fn spawn_bash_process(
        &self,
        command: &str,
        cwd: Option<&Path>,
        call_id: String,
        name: String,
        arguments: String,
    ) -> Result<crate::process::RunningProcess, crate::error::UnifiedError> {
        self.spawn_bash_process(command, cwd, call_id, name, arguments)
    }
}
