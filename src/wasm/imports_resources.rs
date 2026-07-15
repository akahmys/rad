/// Host-side WIT resource implementations for `stream-handle`, `file-handle`,
/// and `execution-handle`.
///
/// All `unwrap()`/`panic!()` patterns have been replaced with graceful
/// fallback-to-`Closed` strategies where the WIT signature forbids `Result`.
use crate::wasm::{WasmState, bindings};
use parking_lot::Mutex;
use wasmtime_wasi::WasiView;

/// Push a `Closed` stream into the resource table as a last-resort fallback.
/// This is used when a WIT method returns a bare resource (non-Result) and
/// the normal path fails.
fn push_closed_fallback(
    state: &mut WasmState,
) -> wasmtime::component::Resource<crate::wasm::HostStream> {
    match state
        .table()
        .push(crate::wasm::HostStream::Closed)
    {
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
        use std::io::Read;
        let stream = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        match stream {
            crate::wasm::HostStream::File(file) => {
                let mut buf = vec![0u8; max_bytes as usize];
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
                match rx.try_recv() {
                    Ok(data) => Ok(data),
                    Err(std::sync::mpsc::TryRecvError::Empty) => Ok(vec![]),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(vec![]),
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
            crate::wasm::HostStream::PipeReader(_) => {
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

impl bindings::wit::HostFileHandle for WasmState {
    fn read_at(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
        offset: u64,
        len: u32,
    ) -> Result<Vec<u8>, String> {
        use std::os::unix::fs::FileExt;
        let file_ref = self.table().get(&self_).map_err(|e| e.to_string())?;
        let file = &file_ref.file;
        let mut buf = vec![0u8; len as usize];
        match file.read_exact_at(&mut buf, offset) {
            Ok(()) => Ok(buf),
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                let mut partial_buf = vec![0u8; len as usize];
                let n = file.read_at(&mut partial_buf, offset).unwrap_or(0);
                partial_buf.truncate(n);
                Ok(partial_buf)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn write_at(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<(), String> {
        use std::os::unix::fs::FileExt;
        let file_ref = self.table().get(&self_).map_err(|e| e.to_string())?;
        let file = &file_ref.file;
        match file.write_all_at(&data, offset) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_stream(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let file_ref = match self.table().get(&self_) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[WASM] get_stream: failed to get file: {e}");
                return push_closed_fallback(self);
            }
        };
        let file_dup = match file_ref.file.try_clone() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[WASM] get_stream: failed to clone file: {e}");
                return push_closed_fallback(self);
            }
        };
        match self
            .table()
            .push(crate::wasm::HostStream::File(file_dup))
        {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[WASM] get_stream: failed to push stream: {e}");
                push_closed_fallback(self)
            }
        }
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostFile>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}

impl bindings::wit::HostExecutionHandle for WasmState {
    fn get_stdout(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let rx_opt = match self.table().get_mut(&self_) {
            Ok(exec) => exec.stdout.lock().take(),
            Err(e) => {
                eprintln!("[WASM] get_stdout: {e}");
                None
            }
        };
        if let Some(rx) = rx_opt {
            match self
                .table()
                .push(crate::wasm::HostStream::PipeReader(Mutex::new(rx)))
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[WASM] get_stdout push failed: {e}");
                    push_closed_fallback(self)
                }
            }
        } else {
            eprintln!("[WASM] Stdout stream already acquired or unavailable");
            push_closed_fallback(self)
        }
    }

    fn get_stderr(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let rx_opt = match self.table().get_mut(&self_) {
            Ok(exec) => exec.stderr.lock().take(),
            Err(e) => {
                eprintln!("[WASM] get_stderr: {e}");
                None
            }
        };
        if let Some(rx) = rx_opt {
            match self
                .table()
                .push(crate::wasm::HostStream::PipeReader(Mutex::new(rx)))
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[WASM] get_stderr push failed: {e}");
                    push_closed_fallback(self)
                }
            }
        } else {
            eprintln!("[WASM] Stderr stream already acquired or unavailable");
            push_closed_fallback(self)
        }
    }

    fn get_stdin(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let stdin_opt = match self.table().get_mut(&self_) {
            Ok(exec) => exec.stdin.lock().take(),
            Err(e) => {
                eprintln!("[WASM] get_stdin: {e}");
                None
            }
        };
        if let Some(stdin) = stdin_opt {
            match self
                .table()
                .push(crate::wasm::HostStream::PipeWriter(Mutex::new(stdin)))
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("[WASM] get_stdin push failed: {e}");
                    push_closed_fallback(self)
                }
            }
        } else {
            eprintln!("[WASM] Stdin stream already acquired or unavailable");
            push_closed_fallback(self)
        }
    }

    fn wait(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> Result<i32, String> {
        let exec = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        let mut running = exec.running.lock();
        match running.child.wait() {
            Ok(status) => {
                running.unregister_pgid();
                Ok(status.exit_code() as i32)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn kill(&mut self, self_: wasmtime::component::Resource<crate::wasm::HostExecution>) {
        match self.table().get_mut(&self_) {
            Ok(exec) => {
                let mut running = exec.running.lock();
                running.kill_group();
            }
            Err(e) => {
                eprintln!("[WASM] kill: failed to get execution handle: {e}");
            }
        }
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}

impl crate::wasm::bindings::rad_llm_connector::radcomp::connector::types::Host for WasmState {}



impl crate::wasm::bindings::rad_llm_connector::radcomp::connector::types::HostStreamHandle for WasmState {
    fn read(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        max_bytes: u32,
    ) -> Result<Vec<u8>, String> {
        bindings::wit::HostStreamHandle::read(self, self_, max_bytes)
    }

    fn write(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        data: Vec<u8>,
    ) -> Result<(), String> {
        bindings::wit::HostStreamHandle::write(self, self_, data)
    }

    fn close(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
    ) {
        bindings::wit::HostStreamHandle::close(self, self_);
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostStream>,
    ) -> Result<(), wasmtime::Error> {
        bindings::wit::HostStreamHandle::drop(self, rep)
    }
}

impl crate::wasm::bindings::rad_llm_connector::LlmConnectorImports for WasmState {
    fn open_http_stream(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        body: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostStream>, String> {
        <WasmState as bindings::RadExtensionImports>::open_http_stream(self, url, headers, body)
    }
}
