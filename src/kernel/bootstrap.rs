//! Bringing the kernel up from config.
//!
//! Runs alongside the existing extension initialisation, not instead of it:
//! for the whole migration both surfaces are live in one process
//! (`ARCHITECTURE-NEXT.md` §9.1).

use super::loader::ModuleRuntime;
use super::shared::KernelShared;
use crate::config::Config;
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
///
/// Takes the whole `Config` rather than the fields it reads. Those now live
/// under three different sections (`modules`, `core`, `default_timeout`), and
/// a positional list of a `&str`, a `bool` and a `u64` is a transposition
/// waiting to happen every time the kernel needs one more of them.
#[must_use]
pub fn boot(config: &Config) -> (Arc<KernelShared>, Vec<String>) {
    let shared = KernelShared::with_workspace(&config.core.workspace);
    shared.hitl_enabled.store(
        config.core.hitl_enabled,
        std::sync::atomic::Ordering::Relaxed,
    );
    let heartbeat_ms = config.default_timeout.llm_stream_heartbeat_ms;
    *shared.llm_timeout_policy.lock() = crate::ipc::TimeoutPolicy::Dynamic {
        heartbeat_timeout_ms: heartbeat_ms,
        max_silent_wait_ms: heartbeat_ms,
    };
    let mut loaded = Vec::new();

    for entry in config.modules.iter().filter(|m| m.enabled) {
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
