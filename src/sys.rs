pub type RawPid = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pid(pub RawPid);

impl Pid {
    #[must_use]
    pub fn from_raw(raw: RawPid) -> Self {
        Pid(raw)
    }
    #[must_use]
    pub fn as_raw(self) -> RawPid {
        self.0
    }
}

#[cfg(unix)]
pub fn kill_process_group(pgid: Pid) -> Result<(), String> {
    use nix::sys::signal::{Signal, killpg};
    use nix::unistd::Pid as NixPid;

    let pid = NixPid::from_raw(pgid.as_raw());
    let _ = killpg(pid, Signal::SIGKILL);

    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_millis(100) {
        if killpg(pid, Signal::SIGTERM) == Err(nix::Error::ESRCH) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(())
}

#[cfg(not(unix))]
pub fn kill_process_group(_pgid: Pid) -> Result<(), String> {
    // Platform fallback stub
    Ok(())
}
