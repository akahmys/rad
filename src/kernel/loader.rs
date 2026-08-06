//! Loading a module: instantiate the component, read its manifest, hand the
//! manifest to the registry.

use super::host::KernelState;
use crate::wasm::bindings::rad_kernel;
use rad_abi::Manifest;
use std::path::Path;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

/// One loaded module: its own `Store`, so a trap in one cannot reach another
/// (`ARCHITECTURE-NEXT.md` §3.6.1).
pub struct ModuleRuntime {
    pub store: Store<KernelState>,
    pub bindings: rad_kernel::Module,
    pub manifest: Manifest,
}

impl ModuleRuntime {
    /// Instantiates a module and reads its manifest.
    ///
    /// # Errors
    ///
    /// Returns a message if the component fails to load or instantiate, if
    /// `manifest()` traps, or if the manifest is malformed or declares a
    /// different ABI.
    pub fn load(name: &str, wasm_path: &Path) -> Result<Self, String> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        if let Err(e) = config.cache_config_load_default() {
            crate::log_host!("[kernel] compile cache unavailable: {e}");
        }
        let engine = Engine::new(&config).map_err(|e| format!("Failed to create Engine: {e}"))?;
        let component = Component::from_file(&engine, wasm_path)
            .map_err(|e| format!("Failed to load module '{name}': {e}"))?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| format!("Linker error WASI: {e}"))?;
        // Supplies every `rad:kernel` import. A module that imports none of them
        // — `modules/echo` imports nothing — instantiates just the same.
        rad_kernel::Module::add_to_linker(&mut linker, |s: &mut KernelState| s)
            .map_err(|e| format!("Linker error rad:kernel: {e}"))?;

        // Modules read the filesystem through `std::fs`, so reachability is
        // whatever is preopened here rather than a capability mask
        // (ARCHITECTURE-NEXT.md §3.4).
        let mut wasi_builder = WasiCtxBuilder::new();
        wasi_builder.inherit_stdout().inherit_stderr();
        let _ = wasi_builder.preopened_dir(
            ".",
            ".",
            wasmtime_wasi::DirPerms::all(),
            wasmtime_wasi::FilePerms::all(),
        );

        let state = KernelState::new(name.to_string(), wasi_builder.build(), ResourceTable::new());
        let mut store = Store::new(&engine, state);
        let bindings = rad_kernel::Module::instantiate(&mut store, &component, &linker)
            .map_err(|e| format!("Failed to instantiate module '{name}': {e}"))?;

        // `manifest()` is required to be pure (§3.2): the kernel calls it before
        // the module has been granted anything, precisely so that what it
        // reports cannot depend on what it was allowed to do.
        let manifest_json = bindings
            .call_manifest(&mut store)
            .map_err(|e| format!("Module '{name}' trapped in manifest(): {e}"))?;
        let manifest = Manifest::parse(&manifest_json)
            .map_err(|e| format!("Module '{name}' returned an unusable manifest: {e}"))?;

        // The configured name and the self-declared name must agree, or routing
        // and diagnostics would disagree about what to call this module.
        if manifest.name != name {
            return Err(format!(
                "module configured as '{name}' declares itself '{}'",
                manifest.name
            ));
        }

        Ok(Self {
            store,
            bindings,
            manifest,
        })
    }

    /// Sends one message to the module.
    ///
    /// # Errors
    ///
    /// Returns a message if the module traps, or the module's own error if it
    /// rejects the call.
    pub fn handle(&mut self, method: &str, payload: &str) -> Result<String, String> {
        self.bindings
            .call_handle(&mut self.store, method, payload)
            .map_err(|e| {
                format!(
                    "module '{}' trapped handling '{method}': {e}",
                    self.manifest.name
                )
            })?
    }
}
