//! `proc-spawn` and the `process` / `byte-stream` resources behind it.
//!
//! The first real syscall. §3.1.2's rule puts it here rather than in WASI:
//! Rust's `std` has no way to reach a child process from inside a component, so
//! the kernel has to provide one. Filesystem and clock access stay on WASI,
//! where `std` already insulates a module from the difference.
//!
//! Split from `host.rs` to keep that file under the 300-line limit.

use super::host::KernelState;
use crate::process::RunningProcess;
use crate::wasm::bindings::rad_kernel::rad::kernel::types;
use std::sync::mpsc::Receiver;
use std::time::Duration;
use wasmtime::component::Resource;
use wasmtime_wasi::WasiView;

/// How long a `read` waits before reporting "nothing yet".
///
/// It must be bounded. Epoch interruption only preempts *guest* code, so a host
/// call blocked in `recv()` cannot be interrupted at all — a module reading from
/// a live but silent child would hang the kernel with the deadline machinery
/// looking on. Returning empty hands control back to the guest, which is where
/// the deadline can fire. The extension host does block here, which is a hazard
/// this does not inherit.
const READ_POLL: Duration = Duration::from_millis(100);

/// How long a single `wait` blocks before reporting [`WAIT_PENDING`].
///
/// Shorter than `HANDLE_DEADLINE_TICKS` (30s) on purpose: a `wait` that
/// outlasts the deadline would be killed as a runaway rather than answered.
const WAIT_SLICE: Duration = Duration::from_secs(25);

/// `wait` returned without the process having exited. Retryable — it says
/// "still running", not "failed".
pub const WAIT_PENDING: u32 = 504;

pub struct KernelProcess {
    pub running: RunningProcess,
}

pub enum KernelStream {
    Reader(Receiver<Vec<u8>>),
    Writer(Box<dyn std::io::Write + Send>),
    /// Handed back when a stream was already taken. A second `stdout()` reads
    /// nothing rather than trapping: the guest asked for something reasonable
    /// and got an honest empty answer.
    Closed,
}

fn err(code: u32, message: impl Into<String>) -> types::Error {
    types::Error {
        code,
        message: message.into(),
    }
}

/// Takes one of a process's streams out and wraps it as a resource.
fn take_stream<F>(
    state: &mut KernelState,
    process: &Resource<types::Process>,
    take: F,
) -> Resource<types::ByteStream>
where
    F: FnOnce(&mut KernelProcess) -> Option<KernelStream>,
{
    let taken = match state.table().get_mut(process) {
        Ok(p) => take(p),
        Err(e) => {
            crate::log_host!("[kernel] process stream lookup failed: {e}");
            None
        }
    };
    let stream = taken.unwrap_or(KernelStream::Closed);
    match state.table().push(stream) {
        Ok(r) => r,
        Err(e) => {
            // Pushing `Closed` cannot be what failed — the table is full or
            // gone — so there is nothing to fall back to but a trap-free lie.
            crate::log_host!("[kernel] could not push stream resource: {e}");
            state
                .table()
                .push(KernelStream::Closed)
                .unwrap_or_else(|_| Resource::new_own(u32::MAX))
        }
    }
}

pub fn spawn(
    state: &mut KernelState,
    argv: Vec<String>,
) -> Result<Resource<types::Process>, types::Error> {
    let Some(shared) = state.shared.upgrade() else {
        return Err(err(503, "kernel is shutting down"));
    };
    if argv.is_empty() {
        return Err(err(400, "proc-spawn requires at least a program name"));
    }
    // The same gate the extension host applies before `open_process`. Without
    // it, moving a tool provider from an extension to a module would quietly
    // remove human approval from every process it starts.
    if shared
        .hitl_enabled
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        let prompt = format!(
            "Spawn process from module '{}': {}",
            state.module_name,
            argv.join(" ")
        );
        match crate::wasm::rpc_process::ask_human_approval(&prompt) {
            Ok(true) => {}
            Ok(false) => {
                // Worded exactly as the extension host words it. The rejection
                // reaches the model and the DAG as text, and both the suite and
                // anyone reading a transcript match on that phrase.
                return Err(err(
                    403,
                    "User rejected execution of tool spawn_bash_process".to_string(),
                ));
            }
            Err(e) => return Err(err(500, format!("approval failed: {e}"))),
        }
    }
    crate::log_host!(
        "[kernel] module '{}' spawning {:?}",
        state.module_name,
        argv
    );
    let running = shared
        .processes
        // Rooted at the workspace, matching what the extension host does. A
        // module's child resolving a relative path has to land where the
        // extension's did.
        .spawn_argv(&argv, Some(&shared.workspace))
        .map_err(|e| err(500, format!("proc-spawn failed: {e}")))?;
    state
        .table()
        .push(KernelProcess { running })
        .map_err(|e| err(500, format!("could not register the process: {e}")))
}

impl types::HostProcess for KernelState {
    fn stdout(&mut self, self_: Resource<types::Process>) -> Resource<types::ByteStream> {
        take_stream(self, &self_, |p| {
            p.running.stdout_rx.take().map(KernelStream::Reader)
        })
    }

    fn stderr(&mut self, self_: Resource<types::Process>) -> Resource<types::ByteStream> {
        take_stream(self, &self_, |p| {
            p.running.stderr_rx.take().map(KernelStream::Reader)
        })
    }

    fn stdin(&mut self, self_: Resource<types::Process>) -> Resource<types::ByteStream> {
        take_stream(self, &self_, |p| {
            p.running
                .stdin_tx
                .take()
                .map(|w| KernelStream::Writer(w.into_inner()))
        })
    }

    /// Blocks for at most [`WAIT_SLICE`]; [`WAIT_PENDING`] means "call again".
    fn wait(&mut self, self_: Resource<types::Process>) -> Result<i32, types::Error> {
        let process = self
            .table()
            .get_mut(&self_)
            .map_err(|e| err(404, format!("no such process: {e}")))?;
        match process.running.wait_with_timeout(WAIT_SLICE) {
            Ok(status) => Ok(i32::try_from(status.exit_code()).unwrap_or(-1)),
            Err(e) => Err(err(WAIT_PENDING, e)),
        }
    }

    fn kill(&mut self, self_: Resource<types::Process>) {
        match self.table().get_mut(&self_) {
            Ok(p) => p.running.kill_group(),
            Err(e) => crate::log_host!("[kernel] kill on a missing process: {e}"),
        }
    }

    /// Killing on drop is the point. A module that returns while its child is
    /// still running would otherwise leave it behind with nothing holding a
    /// handle to it — `ProcessManager::drop` only catches what survives to
    /// shutdown.
    fn drop(&mut self, rep: Resource<types::Process>) -> wasmtime::Result<()> {
        if let Ok(mut process) = self.table().delete(rep) {
            process.running.kill_group();
        }
        Ok(())
    }
}

impl types::HostByteStream for KernelState {
    fn read(
        &mut self,
        self_: Resource<types::ByteStream>,
        max: u32,
    ) -> Result<Vec<u8>, types::Error> {
        let stream = self
            .table()
            .get_mut(&self_)
            .map_err(|e| err(404, format!("no such stream: {e}")))?;
        match stream {
            KernelStream::Reader(rx) => match rx.recv_timeout(READ_POLL) {
                Ok(mut data) => {
                    // The reader thread decides chunk sizes, so `max` is a
                    // ceiling the caller asked for, not one the pipe enforces.
                    data.truncate(max as usize);
                    Ok(data)
                }
                // Both arms are empty rather than an error: a timeout means
                // "nothing yet" and a disconnect means "the child's stdout
                // closed". Neither is a failure the guest can act on
                // differently, and `wait` is how it learns the process ended.
                Err(_) => Ok(Vec::new()),
            },
            KernelStream::Writer(_) => Err(err(400, "cannot read from a write-only stream")),
            KernelStream::Closed => Ok(Vec::new()),
        }
    }

    fn write(
        &mut self,
        self_: Resource<types::ByteStream>,
        data: Vec<u8>,
    ) -> Result<(), types::Error> {
        let stream = self
            .table()
            .get_mut(&self_)
            .map_err(|e| err(404, format!("no such stream: {e}")))?;
        match stream {
            KernelStream::Writer(w) => {
                use std::io::Write;
                w.write_all(&data)
                    .and_then(|()| w.flush())
                    .map_err(|e| err(500, format!("write failed: {e}")))
            }
            KernelStream::Reader(_) => Err(err(400, "cannot write to a read-only stream")),
            KernelStream::Closed => Err(err(400, "stream is closed")),
        }
    }

    /// Closing a writer is what tells the child its input has ended, so this
    /// drops the handle rather than merely marking the resource.
    fn close(&mut self, self_: Resource<types::ByteStream>) {
        if let Ok(stream) = self.table().get_mut(&self_) {
            *stream = KernelStream::Closed;
        }
    }

    fn drop(&mut self, rep: Resource<types::ByteStream>) -> wasmtime::Result<()> {
        let _ = self.table().delete(rep);
        Ok(())
    }
}
