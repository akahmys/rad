// The main Wasm event-dispatch loop, split out of `runner.rs` to stay under
// the 300-line file limit.
//
// It used to share this file with `verify_rpc_exclude`, the RPC verification
// fan-out that asked every extension but the caller to approve each request.
// That went in AWU 973 with `ext/security-guard`: policy is a module now, and
// `modules/mcp` asks it directly (ARCHITECTURE-NEXT.md §3.4.3).
use super::Orchestrator;
use crate::ipc::{RasCoreEvent, route_event_to_terminal};
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

/// How long the loop waits for an event before draining the post queue anyway.
///
/// A post queued by a module running *on this thread* is drained the moment
/// that call returns, so this only bounds the latency of a post queued by
/// another thread — which, once AWU 979 lands, is every LLM chunk. Small
/// enough not to be felt in a stream, large enough that an idle loop is not a
/// spin.
const TICK: Duration = Duration::from_millis(20);

impl Orchestrator {
    /// Delivers everything the kernel has queued.
    ///
    /// **This runs on the event-loop thread, and that is the design, not a
    /// convenience.** Delivering a post takes the target module's lock, and a
    /// module handling one may call another — two locks, held nested. AWU 977
    /// demonstrated that two threads doing that in opposite orders deadlock
    /// beneath the cycle check. The invariant that keeps it from happening is
    /// that **only this thread ever holds more than one module lock**: other
    /// threads may `post` (which touches the queue and nothing else) but must
    /// not `call` into a module that calls onward. A second draining thread
    /// would break that, which is why there isn't one.
    ///
    /// A failed delivery is logged, not propagated: `post` is fire-and-forget
    /// by definition (§3.6.2), and a task must not die because an event had
    /// nowhere to go.
    fn drain_posts(&self) {
        let Some(kernel) = self.kernel.lock().clone() else {
            return;
        };
        for (posted, outcome) in kernel.drain_posts() {
            if let Err(e) = outcome {
                crate::log_host!(
                    "[HOST] posted '{}' from '{}' was not delivered: {e}",
                    posted.method,
                    posted.from
                );
            }
        }
    }

    pub(crate) fn process_event_loop(
        &self,
        event_rx: &Receiver<RasCoreEvent>,
        wasm_runtimes: &HashMap<String, Arc<Mutex<WasmRuntime>>>,
    ) -> Result<(), String> {
        loop {
            if self.abort_flag.load(Ordering::SeqCst) {
                break;
            }

            let event = match event_rx.recv_timeout(TICK) {
                Ok(event) => event,
                // Not an exit: an idle loop still has posts to deliver.
                Err(RecvTimeoutError::Timeout) => {
                    self.drain_posts();
                    continue;
                }
                // Every sender is gone, so no further event can arrive. The
                // old `recv()` ended the loop here too.
                Err(RecvTimeoutError::Disconnected) => break,
            };

            let _ = route_event_to_terminal(&event);

            if let RasCoreEvent::TaskCompleted = event {
                break;
            }

            for runtime_arc in wasm_runtimes.values() {
                let mut runtime = runtime_arc.lock();
                runtime.on_event(&event)?;
            }

            // Not just a latency trim. If events arrive faster than `TICK`,
            // `recv_timeout` returns `Ok` every time and the timeout branch
            // above never runs — without this the queue would starve for as
            // long as the stream lasts.
            self.drain_posts();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
