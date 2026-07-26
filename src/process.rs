use crate::sys::Pid;
use parking_lot::Mutex;
use portable_pty::{Child, ExitStatus};
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

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
        let has_shell_features = command.contains(';') || command.contains('|') || command.contains('>') || command.contains('<') || command.contains('&');
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

        let stdin_writer: Option<Mutex<Box<dyn std::io::Write + Send>>> =
            child.stdin.take().map(|sin| Mutex::new(Box::new(sin) as Box<dyn std::io::Write + Send>));

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

use portable_pty::ChildKiller;

#[derive(Debug)]
struct StdioChild {
    child: std::process::Child,
}

#[derive(Debug, Clone)]
struct StdioChildKiller {
    pid: u32,
}

impl ChildKiller for StdioChildKiller {
    fn kill(&mut self) -> std::io::Result<()> {
        let pid = i32::try_from(self.pid).map_err(std::io::Error::other)?;
        let _ = crate::sys::kill_process_group(Pid::from_raw(pid));
        Ok(())
    }

    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(self.clone())
    }
}

impl ChildKiller for StdioChild {
    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    fn clone_killer(&self) -> Box<dyn ChildKiller + Send + Sync> {
        Box::new(StdioChildKiller {
            pid: self.child.id(),
        })
    }
}

impl Child for StdioChild {
    fn try_wait(&mut self) -> std::io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        Ok(status.map(|s| ExitStatus::with_exit_code(u32::try_from(s.code().unwrap_or(0)).unwrap_or(0))))
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        let status = self.child.wait()?;
        Ok(ExitStatus::with_exit_code(u32::try_from(status.code().unwrap_or(0)).unwrap_or(0)))
    }

    fn process_id(&self) -> Option<u32> {
        Some(self.child.id())
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

fn spawn_reader_thread<R: Read + Send + 'static>(mut reader: R, tx: mpsc::Sender<Vec<u8>>) {
    thread::spawn(move || {
        let mut buf = [0; 1024];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 {
                break;
            }
            if tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });
}

pub struct RunningProcess {
    pub child: Box<dyn Child + Send + Sync>,
    pgid: Pid,
    pub stdout_rx: Option<Receiver<Vec<u8>>>,
    pub stderr_rx: Option<Receiver<Vec<u8>>>,
    pub stdin_tx: Option<Mutex<Box<dyn std::io::Write + Send>>>,
    pub last_activity: Instant,
    active_pgids: Arc<Mutex<Vec<Pid>>>,
    pub timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
}

impl RunningProcess {
    #[must_use]
    pub fn pgid(&self) -> Pid {
        self.pgid
    }

    /// Read any available stdout/stderr without blocking.
    /// Returns (stdout, stderr).
    pub fn read_available(&mut self) -> (Vec<u8>, Vec<u8>) {
        let mut stdout = Vec::new();
        if let Some(ref rx) = self.stdout_rx {
            while let Ok(mut chunk) = rx.try_recv() {
                stdout.append(&mut chunk);
            }
        }

        let mut stderr = Vec::new();
        if let Some(ref rx) = self.stderr_rx {
            while let Ok(mut chunk) = rx.try_recv() {
                stderr.append(&mut chunk);
            }
        }

        if !stdout.is_empty() || !stderr.is_empty() {
            self.last_activity = Instant::now();
        }

        (stdout, stderr)
    }

    /// Wait for process completion or timeout.
    /// If timeout is exceeded, kills the process group and returns an error.
    ///
    /// # Errors
    ///
    /// Returns error if `try_wait` fails or if execution times out.
    pub fn wait_with_timeout(&mut self, timeout: Duration) -> Result<ExitStatus, String> {
        let start = Instant::now();
        loop {
            if let Some(status) = self
                .child
                .try_wait()
                .map_err(|e| format!("try_wait error: {e}"))?
            {
                self.unregister_pgid();
                return Ok(status);
            }

            if self.last_activity.elapsed() > timeout {
                self.kill_group();
                return Err("Process execution timed out due to inactivity".to_string());
            }

            // Fallback upper limit (e.g. 2x timeout) to avoid infinite loops if thread gets stuck
            if start.elapsed() > timeout * 2 {
                self.kill_group();
                return Err("Process execution exceeded maximum timeout limit".to_string());
            }

            thread::sleep(Duration::from_millis(50));
        }
    }

    pub fn kill_group(&mut self) {
        let _ = crate::sys::kill_process_group(self.pgid);
        self.unregister_pgid();
    }

    pub fn unregister_pgid(&mut self) {
        let mut pgids = self.active_pgids.lock();
        pgids.retain(|&x| x != self.pgid);
    }
}

impl Drop for RunningProcess {
    fn drop(&mut self) {
        self.kill_group();
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
