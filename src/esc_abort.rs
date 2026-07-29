//! Lets the user interrupt a running task by pressing Esc, without needing
//! to press Enter first. `rustyline`'s `Editor::readline()` isn't being
//! polled while a task runs (`main.rs`'s `run_agent_task` blocks in its own
//! poll loop instead), so nothing else reads stdin during that window —
//! this module is what makes Esc reach anything at all in that state.
//!
//! Manipulates only the *input* side of termios (`ICANON`/`ECHO` off,
//! `VMIN`=0/`VTIME`=0 for a non-blocking-style read) and deliberately
//! leaves output processing (`OPOST`/`ONLCR`) untouched, unlike
//! `crossterm::terminal::enable_raw_mode` (which clears both) — the
//! background task concurrently `println!`s streamed tokens on a different
//! thread, and disabling `\n` -> `\r\n` translation would turn that output
//! into a staircase mess while this is active.
use nix::sys::termios::{self, SetArg, SpecialCharacterIndices};

/// RAII guard: restores the original termios settings on drop, regardless
/// of how the caller's loop exits (normal completion, early return, panic
/// unwind).
pub struct RawInputGuard {
    original: termios::Termios,
}

impl RawInputGuard {
    /// Puts stdin into input-only raw mode. `None` if stdin isn't backed by
    /// a real terminal (e.g. piped input in scripts/tests) — Esc-abort is
    /// simply unavailable in that case, not an error condition.
    #[must_use]
    pub fn enable() -> Option<Self> {
        let stdin = std::io::stdin();
        let original = termios::tcgetattr(&stdin).ok()?;
        let mut raw = original.clone();
        raw.local_flags.remove(termios::LocalFlags::ICANON | termios::LocalFlags::ECHO);
        raw.control_chars[SpecialCharacterIndices::VMIN as usize] = 0;
        raw.control_chars[SpecialCharacterIndices::VTIME as usize] = 0;
        termios::tcsetattr(&stdin, SetArg::TCSANOW, &raw).ok()?;
        Some(Self { original })
    }
}

impl Drop for RawInputGuard {
    fn drop(&mut self) {
        let stdin = std::io::stdin();
        let _ = termios::tcsetattr(&stdin, SetArg::TCSANOW, &self.original);
    }
}

const ESC: u8 = 0x1b;

fn contains_esc(buf: &[u8]) -> bool {
    buf.contains(&ESC)
}

/// Non-blocking check: `true` if an Esc byte is currently waiting on
/// stdin. Takes `&RawInputGuard` so it can only be called while raw mode
/// is actually active — calling the underlying read in cooked mode would
/// block waiting for a full line, hanging the poll loop.
#[must_use]
pub fn esc_pressed(_guard: &RawInputGuard) -> bool {
    let mut buf = [0u8; 8];
    match nix::unistd::read(0, &mut buf) {
        Ok(n) if n > 0 => contains_esc(&buf[..n]),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_esc_detects_the_byte_anywhere_in_the_slice() {
        assert!(contains_esc(&[0x1b]));
        assert!(contains_esc(&[b'a', 0x1b, b'b']));
        assert!(!contains_esc(b"abc"));
        assert!(!contains_esc(&[]));
    }

    #[test]
    fn test_enable_does_not_hang_or_panic_on_non_tty_stdin() {
        // `cargo test` itself runs with non-tty stdin, so this exercises
        // the real fallback path for free: tcgetattr fails, `enable`
        // returns `None` rather than hanging or erroring.
        assert!(RawInputGuard::enable().is_none());
    }
}
