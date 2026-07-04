use nix::sys::signal::{killpg, Signal};
use nix::unistd::Pid;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
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

    /// Spawns a bash process within its own process group.
    ///
    /// # Errors
    ///
    /// Returns an error if the process fails to spawn or if lock is poisoned.
    pub fn spawn_bash_process(&self, command: &str) -> Result<RunningProcess, String> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(command)
            .process_group(0) // Safe way to call setpgid(0, 0) in child
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn bash: {e}"))?;

        let pid_raw = i32::try_from(child.id()).map_err(|e| format!("Invalid PID: {e}"))?;
        let pid = Pid::from_raw(pid_raw);

        {
            let mut pgids = self.active_pgids.lock().map_err(|e| format!("Lock error: {e}"))?;
            pgids.push(pid);
        }

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        let (stdout_tx, stdout_rx) = mpsc::channel();
        let (stderr_tx, stderr_rx) = mpsc::channel();

        spawn_reader_thread(stdout, stdout_tx);
        spawn_reader_thread(stderr, stderr_tx);

        Ok(RunningProcess {
            child,
            pgid: pid,
            stdout_rx,
            stderr_rx,
            last_activity: Instant::now(),
            active_pgids: self.active_pgids.clone(),
        })
    }
}

impl Drop for ProcessManager {
    fn drop(&mut self) {
        if let Ok(pgids) = self.active_pgids.lock() {
            for pgid in pgids.iter() {
                let _ = killpg(*pgid, Signal::SIGKILL);
            }
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
    child: Child,
    pgid: Pid,
    stdout_rx: Receiver<Vec<u8>>,
    stderr_rx: Receiver<Vec<u8>>,
    last_activity: Instant,
    active_pgids: Arc<Mutex<Vec<Pid>>>,
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
        while let Ok(mut chunk) = self.stdout_rx.try_recv() {
            stdout.append(&mut chunk);
        }

        let mut stderr = Vec::new();
        while let Ok(mut chunk) = self.stderr_rx.try_recv() {
            stderr.append(&mut chunk);
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
    pub fn wait_with_timeout(&mut self, timeout: Duration) -> Result<std::process::ExitStatus, String> {
        let start = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait().map_err(|e| format!("try_wait error: {e}"))? {
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
        let _ = killpg(self.pgid, Signal::SIGKILL);
        self.unregister_pgid();
    }

    fn unregister_pgid(&mut self) {
        if let Ok(mut pgids) = self.active_pgids.lock() {
            pgids.retain(|&x| x != self.pgid);
        }
    }
}
