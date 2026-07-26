// Host-side WIT `file-handle` implementation, split out of
// `imports_resources.rs` to stay under the 300-line file limit.
use crate::wasm::{WasmState, bindings};
use wasmtime_wasi::WasiView;

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
                return crate::wasm::imports_resources::push_closed_fallback(self);
            }
        };
        let file_dup = match file_ref.file.try_clone() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[WASM] get_stream: failed to clone file: {e}");
                return crate::wasm::imports_resources::push_closed_fallback(self);
            }
        };
        match self.table().push(crate::wasm::HostStream::File(file_dup)) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[WASM] get_stream: failed to push stream: {e}");
                crate::wasm::imports_resources::push_closed_fallback(self)
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
