// `portable_pty::Child`/`ChildKiller` glue around a plain `std::process::Child`
// spawned via stdio pipes, plus the stdout/stderr reader-thread helper,
// split out of `process.rs` to stay under the 300-line file limit.
use crate::sys::Pid;
use portable_pty::{Child, ChildKiller, ExitStatus};
use std::io::Read;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub(crate) struct StdioChild {
    pub(crate) child: std::process::Child,
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
        Ok(status
            .map(|s| ExitStatus::with_exit_code(u32::try_from(s.code().unwrap_or(0)).unwrap_or(0))))
    }

    fn wait(&mut self) -> std::io::Result<ExitStatus> {
        let status = self.child.wait()?;
        Ok(ExitStatus::with_exit_code(
            u32::try_from(status.code().unwrap_or(0)).unwrap_or(0),
        ))
    }

    fn process_id(&self) -> Option<u32> {
        Some(self.child.id())
    }
}

pub(crate) fn spawn_reader_thread<R: Read + Send + 'static>(
    mut reader: R,
    tx: mpsc::Sender<Vec<u8>>,
) {
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
