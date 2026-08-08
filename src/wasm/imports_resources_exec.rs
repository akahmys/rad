// Host-side WIT `execution-handle` implementation, split out of
// `imports_resources.rs` to stay under the 300-line file limit.
use crate::wasm::imports_resources::push_closed_fallback;
use crate::wasm::{WasmState, bindings};
use parking_lot::Mutex;
use wasmtime_wasi::WasiView;

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
        if self.is_aborted() {
            return Err("Task aborted by user".to_string());
        }
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
