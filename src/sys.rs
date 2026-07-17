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
    match killpg(pid, Signal::SIGKILL) {
        Ok(_) => Ok(()),
        Err(nix::Error::ESRCH) => {
            // Process group already exited, ignore
            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(not(unix))]
pub fn kill_process_group(_pgid: Pid) -> Result<(), String> {
    // Platform fallback stub
    Ok(())
}
