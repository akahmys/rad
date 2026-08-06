//! Host side of the kernel's imports.
//!
//! The implementations are stubs at this stage. That is not a placeholder in
//! the sense `CODING.md` §3 forbids — the linker must supply every import a
//! module might declare in order to instantiate it at all, and instantiation is
//! what AWU 952 is verifying. `dispatch` gains a real implementation in AWU 953
//! and the syscalls follow; until then a module calling one gets a clear error
//! rather than a link failure naming an internal symbol.
//!
//! Note that a module importing *nothing* from `rad:kernel` still instantiates:
//! `modules/echo` uses neither interface and its component declares no
//! `rad:kernel` imports at all.

use super::shared::KernelShared;
use crate::wasm::bindings::rad_kernel::rad::kernel::{dispatch, syscall, types};
use std::sync::Weak;

/// Per-module store state.
pub struct KernelState {
    /// The module this store belongs to. Every syscall and dispatch is
    /// attributable because the kernel owns the store — the identity never
    /// comes from the guest.
    pub module_name: String,
    /// Weak because `KernelShared` owns the runtimes, and each runtime's store
    /// holds this state — a strong reference either way would be a cycle.
    pub shared: Weak<KernelShared>,
    pub wasi: wasmtime_wasi::WasiCtx,
    pub table: wasmtime::component::ResourceTable,
}

impl KernelState {
    #[must_use]
    pub fn new(
        module_name: String,
        shared: Weak<KernelShared>,
        wasi: wasmtime_wasi::WasiCtx,
        table: wasmtime::component::ResourceTable,
    ) -> Self {
        Self {
            module_name,
            shared,
            wasi,
            table,
        }
    }

    fn unimplemented(&self, what: &str) -> types::Error {
        types::Error {
            code: 501,
            message: format!(
                "module '{}' called {what}, which the kernel does not implement yet",
                self.module_name
            ),
        }
    }
}

impl wasmtime_wasi::WasiView for KernelState {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi
    }

    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.table
    }
}

impl types::Host for KernelState {}

impl types::HostByteStream for KernelState {
    fn read(
        &mut self,
        _self_: wasmtime::component::Resource<types::ByteStream>,
        _max: u32,
    ) -> Result<Vec<u8>, types::Error> {
        Err(self.unimplemented("byte-stream.read"))
    }

    fn write(
        &mut self,
        _self_: wasmtime::component::Resource<types::ByteStream>,
        _data: Vec<u8>,
    ) -> Result<(), types::Error> {
        Err(self.unimplemented("byte-stream.write"))
    }

    fn close(&mut self, _self_: wasmtime::component::Resource<types::ByteStream>) {}

    fn drop(
        &mut self,
        _rep: wasmtime::component::Resource<types::ByteStream>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl types::HostProcess for KernelState {
    fn stdout(
        &mut self,
        _self_: wasmtime::component::Resource<types::Process>,
    ) -> wasmtime::component::Resource<types::ByteStream> {
        unreachable!("proc-spawn cannot succeed yet, so no Process resource exists")
    }

    fn stderr(
        &mut self,
        _self_: wasmtime::component::Resource<types::Process>,
    ) -> wasmtime::component::Resource<types::ByteStream> {
        unreachable!("proc-spawn cannot succeed yet, so no Process resource exists")
    }

    fn stdin(
        &mut self,
        _self_: wasmtime::component::Resource<types::Process>,
    ) -> wasmtime::component::Resource<types::ByteStream> {
        unreachable!("proc-spawn cannot succeed yet, so no Process resource exists")
    }

    fn wait(
        &mut self,
        _self_: wasmtime::component::Resource<types::Process>,
    ) -> Result<i32, types::Error> {
        Err(self.unimplemented("process.wait"))
    }

    fn kill(&mut self, _self_: wasmtime::component::Resource<types::Process>) {}

    fn drop(
        &mut self,
        _rep: wasmtime::component::Resource<types::Process>,
    ) -> wasmtime::Result<()> {
        Ok(())
    }
}

impl syscall::Host for KernelState {
    fn proc_spawn(
        &mut self,
        _argv: Vec<String>,
    ) -> Result<wasmtime::component::Resource<types::Process>, types::Error> {
        Err(self.unimplemented("proc-spawn"))
    }

    fn net_open(
        &mut self,
        _url: String,
        _headers: Vec<(String, String)>,
        _body: Vec<u8>,
    ) -> Result<wasmtime::component::Resource<types::ByteStream>, types::Error> {
        Err(self.unimplemented("net-open"))
    }

    fn log(&mut self, trace_id: String, level: String, message: String) {
        crate::log_host!(
            "[module {}] [{level}] {message} (trace {trace_id})",
            self.module_name
        );
    }
}

impl dispatch::Host for KernelState {
    fn call(&mut self, target: String, method: String, payload: String) -> Result<String, String> {
        let Some(shared) = self.shared.upgrade() else {
            return Err("kernel is shutting down".to_string());
        };
        shared.call(&self.module_name, &target, &method, &payload)
    }

    fn post(&mut self, target: String, method: String, payload: String) {
        // Returns nothing by design: a caller must not be able to observe
        // whether delivery has happened, or it would start depending on timing.
        if let Some(shared) = self.shared.upgrade() {
            shared.post(&self.module_name, &target, &method, &payload);
        }
    }
}
