//! The `post` side of dispatch — the queue, and draining it.
//!
//! Split out of `shared.rs` (AWU 966) when that file crossed the 300-line
//! limit. A coherent piece to lift: `call` is synchronous and re-entrancy is
//! its whole hazard, while `post` exists precisely so that neither is true
//! (§3.6.2). Nothing here touches the call stack.

use super::shared::KernelShared;

/// A message queued by `dispatch.post`, delivered once the target is idle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Posted {
    pub from: String,
    pub target: String,
    pub method: String,
    pub payload: String,
}

impl KernelShared {
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
