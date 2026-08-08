//! The `byte-stream` resource, shared by `proc-spawn` and `net-open`.
//!
//! Split out of `proc.rs` when `net-open` arrived (AWU 966). Both syscalls hand
//! back the same resource, so the file that owns `process` should not also have
//! to own HTTP's failure modes — and neither file has room for both under the
//! 300-line limit.

use super::host::KernelState;
use crate::wasm::bindings::rad_kernel::rad::kernel::types;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
use wasmtime::component::Resource;
use wasmtime_wasi::WasiView;

/// How long a `read` waits before reporting that nothing has arrived.
///
/// It must be bounded. Epoch interruption only preempts *guest* code, so a host
/// call blocked in `recv()` cannot be interrupted at all — a module reading from
/// a live but silent peer would hang the kernel with the deadline machinery
/// looking on. Returning control to the guest is what lets the deadline fire.
/// The extension host does block here, which is a hazard this does not inherit.
pub const READ_POLL: Duration = Duration::from_millis(100);

/// The call did not fail — it has nothing to report *yet*. Retryable.
///
/// The same number `process.wait` uses for the same meaning, and deliberately
/// so: a module sees one "call again" convention across every syscall rather
/// than one per resource.
pub const PENDING: u32 = 504;

/// The transport failed. Not retryable, and not an HTTP status code — a peer
/// answering `504 Gateway Timeout` would otherwise be indistinguishable from
/// [`PENDING`]. The status, when there is one, is in the message.
pub const TRANSPORT_FAILED: u32 = 502;

pub fn err(code: u32, message: impl Into<String>) -> types::Error {
    types::Error {
        code,
        message: message.into(),
    }
}

/// A receive end plus whatever the guest has not asked for yet.
///
/// The buffer is not an optimisation. `read(max)` is a request for *at most*
/// `max` bytes, but chunk sizes are decided by whoever fills the channel, so a
/// chunk can arrive larger than `max`. Truncating it to fit — which is what
/// this code did before `net-open` — silently discards the remainder. That was
/// latent for processes, where the reader thread chunks at 1024 bytes and every
/// caller asks for 4096; it is not latent for HTTP, where a single frame off
/// the socket routinely exceeds 4096.
pub struct Incoming<T> {
    rx: Receiver<T>,
    leftover: Vec<u8>,
}

impl<T> Incoming<T> {
    pub fn new(rx: Receiver<T>) -> Self {
        Self {
            rx,
            leftover: Vec::new(),
        }
    }

    /// Bytes held back from an earlier oversized chunk, up to `max`.
    fn drain(&mut self, max: usize) -> Option<Vec<u8>> {
        if self.leftover.is_empty() {
            return None;
        }
        let n = max.min(self.leftover.len());
        Some(self.leftover.drain(..n).collect())
    }

    /// Hands back `max` bytes and keeps the rest for the next call.
    fn split(&mut self, mut data: Vec<u8>, max: usize) -> Vec<u8> {
        if data.len() > max {
            self.leftover.extend_from_slice(&data[max..]);
            data.truncate(max);
        }
        data
    }
}

pub enum KernelStream {
    /// A child process's stdout or stderr.
    ///
    /// End-of-stream and "nothing yet" are both an empty read here, which is
    /// honest because `process.wait` is available to tell them apart.
    Reader(Incoming<Vec<u8>>),
    /// An HTTP response body. Differs from [`Self::Reader`] on two counts, and
    /// both are load-bearing.
    ///
    /// **It can fail mid-stream** — a dropped connection, a stalled peer, a
    /// non-2xx status. The extension host arrived at the same split
    /// (`PipeReader` / `PipeReaderFallible` in `src/wasm/imports_resources.rs`)
    /// for the same reason: an error delivered as bytes is an error the SSE
    /// parser downstream swallows without noticing.
    ///
    /// **There is no `wait` to ask.** So an empty read cannot also mean
    /// "nothing yet" — that is what [`PENDING`] is for. Collapsing the two, as
    /// `Reader` legitimately does, would make a slow first token
    /// indistinguishable from the end of the response.
    Fallible(Incoming<Result<Vec<u8>, String>>),
    Writer(Box<dyn std::io::Write + Send>),
    /// Handed back when a stream was already taken. A second `stdout()` reads
    /// nothing rather than trapping: the guest asked for something reasonable
    /// and got an honest empty answer.
    Closed,
}

impl types::HostByteStream for KernelState {
    fn read(
        &mut self,
        self_: Resource<types::ByteStream>,
        max: u32,
    ) -> Result<Vec<u8>, types::Error> {
        let max = max as usize;
        // Rejected rather than answered with an empty slice, which on a
        // `Fallible` stream is the end-of-response signal. A guest asking for
        // nothing must not be told the response is over.
        if max == 0 {
            return Err(err(400, "read(0) requests no bytes"));
        }
        let stream = self
            .table()
            .get_mut(&self_)
            .map_err(|e| err(404, format!("no such stream: {e}")))?;
        match stream {
            KernelStream::Reader(incoming) => {
                if let Some(held) = incoming.drain(max) {
                    return Ok(held);
                }
                match incoming.rx.recv_timeout(READ_POLL) {
                    Ok(data) => Ok(incoming.split(data, max)),
                    // Both arms are empty rather than an error: a timeout means
                    // "nothing yet" and a disconnect means "the child's stdout
                    // closed". Neither is a failure the guest can act on
                    // differently, and `wait` is how it learns which happened.
                    Err(_) => Ok(Vec::new()),
                }
            }
            KernelStream::Fallible(incoming) => {
                if let Some(held) = incoming.drain(max) {
                    return Ok(held);
                }
                match incoming.rx.recv_timeout(READ_POLL) {
                    Ok(Ok(data)) => Ok(incoming.split(data, max)),
                    Ok(Err(message)) => Err(err(TRANSPORT_FAILED, message)),
                    Err(RecvTimeoutError::Timeout) => Err(err(PENDING, "no data yet")),
                    // The sender is gone and nothing failed on the way, so the
                    // body is complete.
                    Err(RecvTimeoutError::Disconnected) => Ok(Vec::new()),
                }
            }
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
            KernelStream::Reader(_) | KernelStream::Fallible(_) => {
                Err(err(400, "cannot write to a read-only stream"))
            }
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
