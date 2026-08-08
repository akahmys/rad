/// Host-side WIT resource implementations for `stream-handle`, `file-handle`,
/// and `execution-handle`.
///
/// All `unwrap()`/`panic!()` patterns have been replaced with graceful
/// fallback-to-`Closed` strategies where the WIT signature forbids `Result`.
use crate::wasm::{WasmState, bindings};
use wasmtime_wasi::WasiView;

/// Push a `Closed` stream into the resource table as a last-resort fallback.
/// This is used when a WIT method returns a bare resource (non-Result) and
/// the normal path fails. `pub(crate)` so the `file`/`exec` resource-impl
/// sibling files can reuse it.
pub(crate) fn push_closed_fallback(
    state: &mut WasmState,
) -> wasmtime::component::Resource<crate::wasm::HostStream> {
    match state.table().push(crate::wasm::HostStream::Closed) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[WASM] Critical: failed to push Closed fallback: {e}");
            // ResourceTable::push only fails if the table is at capacity.
            // At this point recovery is impossible — propagate as unreachable.
            unreachable!("ResourceTable exhausted: {e}")
        }
    }
}

impl bindings::wit::HostStreamHandle for WasmState {
    fn read(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        max_bytes: u32,
    ) -> Result<Vec<u8>, String> {
        if self.is_aborted() {
            return Err("Task aborted by user".to_string());
        }
        use std::io::Read;
        let stream = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        match stream {
            crate::wasm::HostStream::File(file) => {
                let chunk_size = (max_bytes as usize).min(65536);
                let mut buf = vec![0u8; chunk_size];
                match file.read(&mut buf) {
                    Ok(n) => {
                        buf.truncate(n);
                        Ok(buf)
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            crate::wasm::HostStream::PipeReader(rx_mutex) => {
                let rx = rx_mutex.lock();
                match rx.recv() {
                    Ok(data) => Ok(data),
                    Err(_) => Ok(vec![]),
                }
            }
            crate::wasm::HostStream::PipeReaderFallible(rx_mutex) => {
                let rx = rx_mutex.lock();
                match rx.recv() {
                    Ok(Ok(data)) => Ok(data),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Ok(vec![]),
                }
            }
            crate::wasm::HostStream::PipeWriter(_) => {
                Err("Cannot read from a write-only stream".to_string())
            }
            crate::wasm::HostStream::Closed => Ok(vec![]),
        }
    }

    fn write(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        data: Vec<u8>,
    ) -> Result<(), String> {
        if self.is_aborted() {
            return Err("Task aborted by user".to_string());
        }
        use std::io::Write;
        let stream = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        match stream {
            crate::wasm::HostStream::File(file) => match file.write_all(&data) {
                Ok(()) => Ok(()),
                Err(e) => Err(e.to_string()),
            },
            crate::wasm::HostStream::PipeWriter(stdin_mutex) => {
                let mut stdin = stdin_mutex.lock();
                match stdin.write_all(&data) {
                    Ok(()) => {
                        let _ = stdin.flush();
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            crate::wasm::HostStream::PipeReader(_)
            | crate::wasm::HostStream::PipeReaderFallible(_) => {
                Err("Cannot write to a read-only stream".to_string())
            }
            crate::wasm::HostStream::Closed => Err("Stream is closed".to_string()),
        }
    }

    fn close(&mut self, self_: wasmtime::component::Resource<crate::wasm::HostStream>) {
        if let Ok(stream) = self.table().get_mut(&self_) {
            *stream = crate::wasm::HostStream::Closed;
        }
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostStream>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}

// `file-handle` (imports_resources_file.rs) and `execution-handle`
// (imports_resources_exec.rs) implementations live in sibling files to keep
// this one under the 300-line limit.
