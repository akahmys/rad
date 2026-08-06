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
use super::registry::Registry;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

/// A message queued by `dispatch.post`, delivered once the target is idle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    pub from: String,
    pub target: String,
    pub method: String,
    pub payload: String,
}

/// Everything a host call needs to route a message.
pub struct KernelShared {
    pub registry: Mutex<Registry>,
    /// Per-module locks, never a lock over the whole map — see the module docs.
    pub modules: Mutex<HashMap<String, Arc<Mutex<ModuleRuntime>>>>,
    /// Modules currently mid-`call`, outermost first. Shared across every store
    /// because a chain spans several of them.
    pub call_stack: Mutex<Vec<String>>,
    pub post_queue: Mutex<VecDeque<Posted>>,
}

impl KernelShared {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            registry: Mutex::new(Registry::new()),
            modules: Mutex::new(HashMap::new()),
            call_stack: Mutex::new(Vec::new()),
            post_queue: Mutex::new(VecDeque::new()),
        })
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

    fn deliver(&self, name: &str, method: &str, payload: &str) -> Result<String, String> {
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

    /// Queues a message. Never fails and never blocks — that is the whole point
    /// of `post` versus `call` (§3.6.2).
    pub fn post(&self, from: &str, target: &str, method: &str, payload: &str) {
        self.post_queue.lock().push_back(Posted {
            from: from.to_string(),
            target: target.to_string(),
            method: method.to_string(),
            payload: payload.to_string(),
        });
    }

    /// Delivers everything currently queued, and anything queued while doing so.
    ///
    /// Runs outside any in-flight `call`, so the stack is empty and a posted
    /// message can legitimately reach a module that posted it — which is how
    /// event loops are supposed to work.
    ///
    /// Returns each delivery's outcome, oldest first.
    pub fn drain_posts(&self) -> Vec<(Posted, Result<String, String>)> {
        let mut delivered = Vec::new();
        // Bounded so a module posting to itself on every message cannot spin
        // here forever; the remainder stays queued for the next drain.
        const MAX_PER_DRAIN: usize = 1024;
        while delivered.len() < MAX_PER_DRAIN {
            let Some(posted) = self.post_queue.lock().pop_front() else {
                break;
            };
            let outcome = match self.resolve(&posted.target, &posted.method) {
                Some(name) => self.deliver(&name, &posted.method, &posted.payload),
                None => Err(format!(
                    "no module provides '{}' (posted from '{}')",
                    posted.method, posted.from
                )),
            };
            delivered.push((posted, outcome));
        }
        delivered
    }
}
