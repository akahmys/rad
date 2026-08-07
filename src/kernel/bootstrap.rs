//! Bringing the kernel up from config.
//!
//! Runs alongside the existing extension initialisation, not instead of it:
//! for the whole migration both surfaces are live in one process
//! (`ARCHITECTURE-NEXT.md` §9.1).

use super::loader::ModuleRuntime;
use super::shared::KernelShared;
use crate::config::ModuleConfig;
use parking_lot::Mutex;
use std::sync::Arc;

/// Loads every enabled module in `modules`.
///
/// Returns the kernel and the names it loaded. A module that fails to load is
/// reported and skipped rather than aborting startup — one broken third-party
/// module should not stop rad from running, and the same is true of a name
/// collision, which is caught here rather than surfacing as a mystery route.
///
/// A configuration with no modules yields a kernel with nothing in it, which is
/// the normal state until stage 3 moves the first extension across.
#[must_use]
pub fn boot(modules: &[ModuleConfig]) -> (Arc<KernelShared>, Vec<String>) {
    let shared = KernelShared::new();
    let mut loaded = Vec::new();

    for entry in modules.iter().filter(|m| m.enabled) {
        let path = crate::config::expand_tilde(&entry.source);
        let runtime = match ModuleRuntime::load(
            &entry.name,
            &path,
            &shared.engine,
            Arc::downgrade(&shared),
        ) {
            Ok(rt) => rt,
            Err(e) => {
                eprintln!(
                    "\x1b[33mWarning: module '{}' not loaded: {e}\x1b[0m",
                    entry.name
                );
                continue;
            }
        };

        if let Err(e) = shared.registry.lock().register(runtime.manifest.clone()) {
            eprintln!(
                "\x1b[33mWarning: module '{}' not registered: {e}\x1b[0m",
                entry.name
            );
            continue;
        }

        shared.set_module_config(&entry.name, entry.config.clone());
        shared
            .modules
            .lock()
            .insert(entry.name.clone(), Arc::new(Mutex::new(runtime)));
        loaded.push(entry.name.clone());
    }

    (shared, loaded)
}
