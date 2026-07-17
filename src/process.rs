use crate::sys::Pid;
use parking_lot::Mutex;
use portable_pty::{Child, CommandBuilder, ExitStatus, PtySize, native_pty_system};
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

    /// Spawns a bash process within its own process group using PTY.
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
        let pty_system = native_pty_system();
        let pty_pair = pty_system
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| {
                crate::error::UnifiedError::l1(format!("Failed to open PTY: {e}"), "Process")
            })?;

        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-c");
        cmd.arg(command);
        if let Some(p) = cwd {
            cmd.cwd(p);
        }

        let child = pty_pair.slave.spawn_command(cmd).map_err(|e| {
            crate::error::UnifiedError::l1(
                format!("Failed to spawn command in PTY: {e}"),
                "Process",
            )
        })?;

        let pgid_raw = pty_pair.master.process_group_leader().ok_or_else(|| {
            crate::error::UnifiedError::l1("Failed to get process group leader", "Process")
        })?;

        let pid = Pid::from_raw(pgid_raw);

        {
            let mut pgids = self.active_pgids.lock();
            pgids.push(pid);
        }

        let master_reader = pty_pair.master.try_clone_reader().map_err(|e| {
            crate::error::UnifiedError::l1(format!("Failed to clone master reader: {e}"), "Process")
        })?;

        let master_writer = pty_pair.master.take_writer().map_err(|e| {
            crate::error::UnifiedError::l1(format!("Failed to take master writer: {e}"), "Process")
        })?;

        let (stdout_tx, stdout_rx) = mpsc::channel();
        let (_stderr_tx, stderr_rx) = mpsc::channel();

        spawn_reader_thread(master_reader, stdout_tx);

        Ok(RunningProcess {
            child,
            pgid: pid,
            stdout_rx: Some(stdout_rx),
            stderr_rx: Some(stderr_rx),
            stdin_tx: Some(Mutex::new(master_writer)),
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
