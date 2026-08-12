use parking_lot::Mutex;
use std::sync::OnceLock;

/// Represents the active phase of the REPL CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalState {
    /// REPL is awaiting user input or processing commands.
    Idle,
    /// Agent/LLM is actively thinking (prior to first token stream).
    Thinking,
    /// Agent/LLM is actively streaming response tokens to stdout.
    Streaming,
}

/// Unified terminal output controller.
/// Manages standard output printing, thinking indicator display/erasure,
/// and log buffering during task execution to prevent prompt corruption.
///
/// **`write_raw` was removed in AWU 987.** It read the state directly, and with
/// a `ui` module loaded that copy freezes at `Idle` — `set_state` delegates and
/// returns before touching it — so raw bytes would have printed straight
/// through a streaming response. It had no callers: `route_event_to_terminal`
/// stopped feeding it when it became a no-op, to keep process output from
/// polluting the REPL layout. Anything that needs it again should add a `ui`
/// method rather than a fourth reader of a state that lives elsewhere.
pub struct TerminalController {
    state: Mutex<TerminalState>,
    deferred_buffer: Mutex<Vec<String>>,
    /// The `ui` module, when one is loaded. It owns the state machine then;
    /// the two fields above go unused and are deleted in AWU 988 along with
    /// the rest of the host's copy.
    ///
    /// Set once at boot rather than passed in per call, because the callers
    /// reach this through `get_terminal()` — a process-wide singleton with no
    /// context to thread a handle through.
    kernel: Mutex<Option<std::sync::Arc<crate::kernel::KernelShared>>>,
}

impl TerminalController {
    /// Creates a new `TerminalController` initialized in the `Idle` state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TerminalState::Idle),
            deferred_buffer: Mutex::new(Vec::new()),
            kernel: Mutex::new(None),
        }
    }

    /// Hands the controller the kernel, so it can find a `ui` module. Called
    /// once, from `Orchestrator::new`.
    pub fn attach_kernel(&self, kernel: std::sync::Arc<crate::kernel::KernelShared>) {
        *self.kernel.lock() = Some(kernel);
    }

    /// Runs one call on the `ui` module, or `None` if nothing provides it.
    fn on_module(&self, method: &str, payload: &serde_json::Value) -> Option<()> {
        let kernel = self.kernel.lock().clone()?;
        kernel.provider_of(method)?;
        // A terminal write that fails is not worth failing a task over, and
        // there is nowhere to report it that is not itself the terminal.
        let _ = kernel.call("host", "ui", method, &payload.to_string());
        Some(())
    }

    /// Sets the terminal state and handles transition actions (e.g. erasing Thinking indicator).
    pub fn set_state(&self, new_state: TerminalState) {
        let name = match new_state {
            TerminalState::Idle => "idle",
            TerminalState::Thinking => "thinking",
            TerminalState::Streaming => "streaming",
        };
        if self
            .on_module("ui.state", &serde_json::json!({ "state": name }))
            .is_some()
        {
            return;
        }
        let mut state_guard = self.state.lock();
        let old_state = *state_guard;
        if old_state == new_state {
            return;
        }

        *state_guard = new_state;

        match new_state {
            TerminalState::Thinking => {}
            TerminalState::Idle => {
                // If task ends while thinking, erase indicator
                if old_state == TerminalState::Thinking {
                    print!("\x1b[2K\r");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }

                // Flush deferred logs gathered during task execution
                let mut buffer_guard = self.deferred_buffer.lock();
                for log in std::mem::take(&mut *buffer_guard) {
                    println!("{log}");
                }
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            TerminalState::Streaming => {
                // Erase Thinking indicator just before printing the first token
                if old_state == TerminalState::Thinking {
                    print!("\x1b[2K\r");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
            }
        }
    }

    /// Outputs a response token from LLM stream, transitioning to `Streaming` state automatically.
    pub fn write_llm_token(&self, token: &str) {
        if self
            .on_module("ui.token", &serde_json::json!({ "text": token }))
            .is_some()
        {
            return;
        }
        self.set_state(TerminalState::Streaming);
        print!("{token}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }

    /// Outputs a system log/event.
    /// If LLM execution is active, defers output to memory buffer to avoid display pollution.
    pub fn write_log(&self, log: String) {
        if self
            .on_module("ui.log", &serde_json::json!({ "text": log }))
            .is_some()
        {
            return;
        }
        let state_guard = self.state.lock();
        match *state_guard {
            TerminalState::Idle => {
                println!("{log}");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            TerminalState::Thinking | TerminalState::Streaming => {
                let mut buffer_guard = self.deferred_buffer.lock();
                buffer_guard.push(log);
            }
        }
    }
}

impl Default for TerminalController {
    fn default() -> Self {
        Self::new()
    }
}

/// Retrieves the global singleton instance of `TerminalController`.
#[must_use]
pub fn get_terminal() -> &'static TerminalController {
    static TERM_CTRL: OnceLock<TerminalController> = OnceLock::new();
    TERM_CTRL.get_or_init(TerminalController::new)
}
