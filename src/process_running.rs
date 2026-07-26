// `RunningProcess`: a handle to a spawned child plus its stdio pipes and
// timeout/liveness bookkeeping, split out of `process.rs` to stay under the
// 300-line file limit.
use crate::sys::Pid;
use parking_lot::Mutex;
use portable_pty::{Child, ExitStatus};
use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

pub struct RunningProcess {
    pub child: Box<dyn Child + Send + Sync>,
    pub(crate) pgid: Pid,
    pub stdout_rx: Option<Receiver<Vec<u8>>>,
    pub stderr_rx: Option<Receiver<Vec<u8>>>,
    pub stdin_tx: Option<Mutex<Box<dyn std::io::Write + Send>>>,
    pub last_activity: Instant,
    pub(crate) active_pgids: Arc<Mutex<Vec<Pid>>>,
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
