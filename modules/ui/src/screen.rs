//! The terminal's state machine, moved from `src/terminal.rs`.
//!
//! Every transition and every buffering rule comes across unchanged. The state
//! is what makes this worth moving as one piece: `write_log` defers while a
//! response is streaming and flushes when it stops, so a host that kept the
//! state while a module printed the tokens would have two halves of one
//! decision — the divergence AWU 986 spent its length avoiding.
//!
//! Printing goes to `println!`, which reaches the real terminal because the
//! kernel gives every module `inherit_stdout` (`src/kernel/loader.rs`).
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum State {
    /// Awaiting user input or processing commands.
    Idle,
    /// Thinking, before the first token.
    Thinking,
    /// Streaming tokens to stdout.
    Streaming,
}

impl State {
    pub(crate) fn parse(name: &str) -> Option<Self> {
        match name {
            "idle" => Some(Self::Idle),
            "thinking" => Some(Self::Thinking),
            "streaming" => Some(Self::Streaming),
            _ => None,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Thinking => "thinking",
            Self::Streaming => "streaming",
        }
    }
}

struct Screen {
    state: State,
    /// Logs that arrived while the terminal was busy, kept until it is idle
    /// again so they cannot land in the middle of a streamed response.
    deferred: Vec<String>,
}

thread_local! {
    static SCREEN: RefCell<Screen> = const {
        RefCell::new(Screen {
            state: State::Idle,
            deferred: Vec::new(),
        })
    };
}

/// Erases the thinking indicator. `\x1b[2K\r` — the exact sequence the host
/// used, because anything else changes what a user sees.
fn erase_indicator() {
    print!("\x1b[2K\r");
    flush();
}

fn flush() {
    use std::io::Write as _;
    let _ = std::io::stdout().flush();
}

pub(crate) fn set_state(new_state: State) {
    SCREEN.with_borrow_mut(|screen| {
        let old = screen.state;
        if old == new_state {
            return;
        }
        screen.state = new_state;

        match new_state {
            State::Thinking => {}
            State::Idle => {
                // A task that ends while still thinking leaves the indicator on
                // screen otherwise.
                if old == State::Thinking {
                    erase_indicator();
                }
                for log in std::mem::take(&mut screen.deferred) {
                    println!("{log}");
                }
                flush();
            }
            State::Streaming => {
                // Erased just before the first token, not when thinking ends,
                // so the indicator stays up for the whole wait.
                if old == State::Thinking {
                    erase_indicator();
                }
            }
        }
    });
}

/// One token of a streamed response. Moves the terminal to `Streaming` on its
/// own, which is what erases the indicator before the first one prints.
pub(crate) fn write_token(token: &str) {
    set_state(State::Streaming);
    print!("{token}");
    flush();
}

/// A log line. Printed when idle, deferred otherwise.
pub(crate) fn write_log(log: String) {
    SCREEN.with_borrow_mut(|screen| match screen.state {
        State::Idle => {
            println!("{log}");
            flush();
        }
        State::Thinking | State::Streaming => screen.deferred.push(log),
    });
}

pub(crate) fn state() -> State {
    SCREEN.with_borrow(|screen| screen.state)
}

/// How many logs are waiting. Exposed because printing is not observable from
/// a test and this is: it is the part of `write_log` that can be wrong.
pub(crate) fn deferred_count() -> usize {
    SCREEN.with_borrow(|screen| screen.deferred.len())
}

#[cfg(test)]
mod tests;
