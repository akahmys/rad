use std::sync::Mutex;
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
pub struct TerminalController {
    state: Mutex<TerminalState>,
    deferred_buffer: Mutex<Vec<String>>,
}

impl TerminalController {
    /// Creates a new `TerminalController` initialized in the `Idle` state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(TerminalState::Idle),
            deferred_buffer: Mutex::new(Vec::new()),
        }
    }

    /// Sets the terminal state and handles transition actions (e.g. erasing Thinking indicator).
    pub fn set_state(&self, new_state: TerminalState) {
        let mut state_guard = self.state.lock().unwrap();
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
                let mut buffer_guard = self.deferred_buffer.lock().unwrap();
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
        self.set_state(TerminalState::Streaming);
        print!("{token}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }

    /// Outputs a system log/event.
    /// If LLM execution is active, defers output to memory buffer to avoid display pollution.
    pub fn write_log(&self, log: String) {
        let state_guard = self.state.lock().unwrap();
        match *state_guard {
            TerminalState::Idle => {
                println!("{log}");
                let _ = std::io::Write::flush(&mut std::io::stdout());
            }
            TerminalState::Thinking | TerminalState::Streaming => {
                let mut buffer_guard = self.deferred_buffer.lock().unwrap();
                buffer_guard.push(log);
            }
        }
    }

    /// Outputs raw bytes to stdout or stderr.
    /// If LLM execution is active, defers output as a string to memory buffer.
    pub fn write_raw(&self, data: &[u8], is_stderr: bool) {
        use std::io::Write;
        let state_guard = self.state.lock().unwrap();
        match *state_guard {
            TerminalState::Idle => {
                if is_stderr {
                    let _ = std::io::stderr().write_all(data);
                    let _ = std::io::stderr().flush();
                } else {
                    let _ = std::io::stdout().write_all(data);
                    let _ = std::io::stdout().flush();
                }
            }
            TerminalState::Thinking | TerminalState::Streaming => {
                let log = String::from_utf8_lossy(data).into_owned();
                let mut buffer_guard = self.deferred_buffer.lock().unwrap();
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
