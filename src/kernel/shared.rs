//! State reachable from inside a module's host calls.
//!
//! `dispatch.call` runs while the *calling* module's `Store` is borrowed, and
//! it has to reach a different module's `Store` to deliver the message. So the
//! runtimes cannot live behind one lock: locking the map would deadlock on the
//! caller's own entry. Each module gets its own `Mutex`, and a call only ever
//! locks the target's.
//!
//! That leaves one hazard, and it is the reason §3.6.3 exists: A→B→A would have
//! B wait on A's lock, which the in-flight call still holds. The call stack
//! below turns that from a hang into an error before the lock is ever reached.

use super::loader::ModuleRuntime;
use super::posts::Posted;
use super::registry::Registry;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// How long a single `handle()` may run before the kernel traps it.
///
/// Not a performance budget — it is the ceiling on how long a module can hold
/// the process. A third-party module must not be able to freeze rad
/// (`ARCHITECTURE-NEXT.md` §3.6.5), and `src/esc_abort.rs`'s cooperative flag
/// cannot stop code that never checks it.
pub const EPOCH_TICK_MS: u64 = 50;
pub const HANDLE_DEADLINE_TICKS: u64 = 600; // 30s

/// Everything a host call needs to route a message.
pub struct KernelShared {
    /// One `Engine` for every module, so a single ticker thread drives all
    /// their epoch deadlines. Per-module engines would each need their own.
    pub engine: wasmtime::Engine,
    pub registry: Mutex<Registry>,
    /// Per-module locks, never a lock over the whole map — see the module docs.
    pub modules: Mutex<HashMap<String, Arc<Mutex<ModuleRuntime>>>>,
    /// Modules currently mid-`call`, outermost first. Shared across every store
    /// because a chain spans several of them.
    pub call_stack: Mutex<Vec<String>>,
    pub post_queue: Mutex<VecDeque<Posted>>,
    /// Per-module config from `rad.json`, fetched by a module through
    /// `kernel.config`. Opaque to the kernel — it stores and returns it.
    module_config: Mutex<HashMap<String, serde_json::Value>>,
    /// Shared by every module rather than one per store, so that process-group
    /// cleanup on shutdown covers all of them. `ProcessManager::drop` kills
    /// every group it spawned; per-module managers would each have to outlive
    /// their store for that to hold.
    pub processes: Arc<crate::process::ProcessManager>,
    /// Where a module's children start, and what it sees as its filesystem.
    ///
    /// The extension host spawns with `cwd = workspace` and the kernel has to
    /// match it: a module resolving a relative path — an MCP server writing a
    /// file, a skill looking for `.agents/skills` — must land in the same place
    /// as the extension it replaced, not wherever rad happened to be started.
    pub workspace: std::path::PathBuf,
    /// Whether a module's `proc-spawn` needs human approval.
    ///
    /// The extension host asks before spawning (`src/wasm/imports_process.rs`).
    /// Routing a tool provider through a module removed that gate entirely
    /// until this was added — a user who had turned HITL on would have lost it
    /// silently. `tests/hitl_tests.rs` is what caught it.
    pub hitl_enabled: std::sync::atomic::AtomicBool,
    /// Connect and per-chunk-heartbeat budgets for `net-open`.
    ///
    /// One per kernel, where the extension host has one per extension
    /// (`src/wasm/loader.rs` builds a fresh policy for every `WasmState`). The
    /// finer grain there is unused — no extension calls
    /// `SetStreamTimeoutPolicy` — so this is the same behaviour with one owner
    /// rather than a capability nobody exercises.
    pub llm_timeout_policy: Arc<Mutex<crate::ipc::TimeoutPolicy>>,
    ticker_stop: Arc<AtomicBool>,
}

/// The kernel answers dispatch like any other target, so a module reaching
/// `kernel.config` uses the same call it would use for a peer and never has to
/// special-case the host (§3.6.7).
pub const KERNEL_TARGET: &str = "kernel";

impl KernelShared {
    /// # Panics
    ///
    /// Panics if wasmtime cannot build an engine, which means the process
    /// cannot host modules at all — there is nothing to degrade to.
    #[must_use]
    pub fn new() -> Arc<Self> {
        Self::with_workspace(".")
    }

    /// The heartbeat a kernel built without a config uses.
    ///
    /// Matches `config::default_llm_stream_heartbeat_ms`. Duplicated rather
    /// than imported because that function is private to `config`, and a test
    /// kernel should not need a `Config` to exist.
    const DEFAULT_HEARTBEAT_MS: u64 = 15_000;

    /// The kernel, rooted at a workspace.
    ///
    /// # Panics
    ///
    /// Panics if wasmtime cannot build an engine, which means the process
    /// cannot host modules at all — there is nothing to degrade to.
    #[must_use]
    pub fn with_workspace(workspace: impl Into<std::path::PathBuf>) -> Arc<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        // Lets the kernel preempt a module that never returns. Cheap enough to
        // leave on unconditionally — the check is a load and compare per loop
        // back-edge, versus `consume_fuel`'s per-instruction accounting.
        config.epoch_interruption(true);
        if let Err(e) = config.cache_config_load_default() {
            crate::log_host!("[kernel] compile cache unavailable: {e}");
        }
        let engine = wasmtime::Engine::new(&config).expect("kernel engine");

        let ticker_stop = Arc::new(AtomicBool::new(false));
        Self::spawn_ticker(&engine, &ticker_stop);

        Arc::new(Self {
            engine,
            registry: Mutex::new(Registry::new()),
            modules: Mutex::new(HashMap::new()),
            call_stack: Mutex::new(Vec::new()),
            post_queue: Mutex::new(VecDeque::new()),
            module_config: Mutex::new(HashMap::new()),
            processes: Arc::new(crate::process::ProcessManager::new()),
            workspace: workspace.into(),
            hitl_enabled: std::sync::atomic::AtomicBool::new(false),
            llm_timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Dynamic {
                heartbeat_timeout_ms: Self::DEFAULT_HEARTBEAT_MS,
                max_silent_wait_ms: Self::DEFAULT_HEARTBEAT_MS,
            })),
            ticker_stop,
        })
    }

    /// Advances the epoch on a fixed interval. Without something incrementing
    /// it, `set_epoch_deadline` never fires and the interruption is inert.
    fn spawn_ticker(engine: &wasmtime::Engine, stop: &Arc<AtomicBool>) {
        let engine = engine.weak();
        let stop = Arc::clone(stop);
        std::thread::Builder::new()
            .name("rad-kernel-epoch".to_string())
            .spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(EPOCH_TICK_MS));
                    // A weak handle so this thread cannot keep the engine, and
                    // therefore the process, alive on its own.
                    let Some(engine) = engine.upgrade() else {
                        break;
                    };
                    engine.increment_epoch();
                }
            })
            .expect("epoch ticker thread");
    }

    pub fn set_module_config(&self, module: &str, config: serde_json::Value) {
        self.module_config.lock().insert(module.to_string(), config);
    }

    /// Handles a call addressed to the kernel itself.
    fn handle_kernel(&self, from: &str, method: &str) -> Result<String, String> {
        match method {
            "kernel.config" => {
                let config = self.module_config.lock();
                let value = config.get(from).cloned().unwrap_or(serde_json::Value::Null);
                serde_json::to_string(&value).map_err(|e| e.to_string())
            }
            "kernel.modules" => {
                let registry = self.registry.lock();
                serde_json::to_string(&registry.module_names()).map_err(|e| e.to_string())
            }
            other => Err(format!("the kernel does not provide '{other}'")),
        }
    }

    /// Resolves a dispatch target. `target` is either a module name or a method
    /// name — routing by method is what lets a caller ask for a capability
    /// without knowing which module currently provides it.
    #[must_use]
    pub fn resolve(&self, target: &str, method: &str) -> Option<String> {
        let registry = self.registry.lock();
        if registry.manifest(target).is_some() {
            return Some(target.to_string());
        }
        registry.route(method).map(ToString::to_string)
    }

    /// Every loaded module, in registration order.
    #[must_use]
    pub fn modules(&self) -> Vec<String> {
        self.registry
            .lock()
            .module_names()
            .into_iter()
            .map(ToString::to_string)
            .collect()
    }

    /// The module declaring `method`, by method alone. Unlike [`Self::resolve`],
    /// a live module name is not itself an answer — callers that need to know
    /// whether a module actually offers a method must ask this.
    #[must_use]
    pub fn provider_of(&self, method: &str) -> Option<String> {
        self.registry.lock().route(method).map(ToString::to_string)
    }

    /// Delivers a synchronous call, refusing to re-enter a module already on
    /// the stack.
    ///
    /// # Errors
    ///
    /// Returns a message if the target cannot be resolved, if delivering it
    /// would form a cycle, or if the target module itself returns an error.
    pub fn call(
        &self,
        from: &str,
        target: &str,
        method: &str,
        payload: &str,
    ) -> Result<String, String> {
        if target == KERNEL_TARGET || method.starts_with("kernel.") {
            return self.handle_kernel(from, method);
        }
        let Some(name) = self.resolve(target, method) else {
            return Err(format!(
                "no module provides '{method}' (dispatch from '{from}' to '{target}')"
            ));
        };

        // Checked before taking the target's lock: past that point a cycle is a
        // deadlock, not an error anyone could report.
        {
            let mut stack = self.call_stack.lock();
            if stack.iter().any(|m| m == &name) {
                let mut chain = stack.clone();
                chain.push(name.clone());
                return Err(format!("dispatch cycle: {}", chain.join(" -> ")));
            }
            stack.push(name.clone());
        }

        let result = self.deliver(&name, method, payload);

        self.call_stack.lock().pop();
        result
    }

    pub(super) fn deliver(
        &self,
        name: &str,
        method: &str,
        payload: &str,
    ) -> Result<String, String> {
        // Clone the Arc and drop the map lock immediately: holding it across the
        // guest call would serialise every module in the process.
        let runtime = {
            let modules = self.modules.lock();
            modules.get(name).cloned()
        };
        let Some(runtime) = runtime else {
            return Err(format!("module '{name}' is registered but not loaded"));
        };
        let mut runtime = runtime.lock();
        runtime.handle(method, payload)
    }

    /// Stops the epoch ticker. Called on shutdown; the thread also exits on its
    /// own once the engine is dropped.
    pub fn stop(&self) {
        self.ticker_stop.store(true, Ordering::Relaxed);
    }
}
