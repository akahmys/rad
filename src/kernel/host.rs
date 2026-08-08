//! Host side of the kernel's imports.
//!
//! Every one of them is now implemented: `dispatch` here, `proc-spawn` in
//! `proc.rs`, `net-open` in `net.rs`. The syscall surface is closed at three
//! (§3.1) — anything else a module needs, it reaches through `std` on WASI.
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

impl syscall::Host for KernelState {
    fn proc_spawn(
        &mut self,
        argv: Vec<String>,
    ) -> Result<wasmtime::component::Resource<types::Process>, types::Error> {
        super::proc::spawn(self, argv)
    }

    fn net_open(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
    ) -> Result<wasmtime::component::Resource<types::ByteStream>, types::Error> {
        super::net::open(self, url, headers, body)
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
