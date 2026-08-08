//! The one in-flight generation, held between `llm.generate` and `llm.next`.
//!
//! A `thread_local` rather than a `static Mutex`: the stream is a guest-side
//! resource handle and so is not `Send`, and `Mutex<T>` would need it to be.
//! `modules/mcp` reaches for `unsafe impl Send` at this point; a thread-local
//! needs no such claim, and CODING.md §4 prohibits `unsafe` outright.
//! Single-threaded is not an assumption here — a module's store is entered by
//! one caller at a time, by construction.

use crate::sse::{LlmEvent, Parser};
use crate::types::ByteStream;
use std::cell::RefCell;

/// Bytes per `read`. The kernel holds back whatever exceeds this, so the value
/// is a pacing choice rather than a limit on what the peer may send.
const READ_MAX: u32 = 4096;

/// Nothing arrived yet — call again. Not the end of the response, which is an
/// empty read (`src/kernel/stream.rs`).
const PENDING: u32 = 504;

/// `read` waits 100ms per [`PENDING`], so this is a ~15s ceiling on a server
/// that has accepted the request and then gone quiet. Below the kernel's 30s
/// handle deadline on purpose: a stalled model should be reported as a stall,
/// not killed as a runaway module.
const MAX_PENDING: u32 = 150;

pub struct Session {
    stream: ByteStream,
    dialect: &'static crate::dialect::Dialect,
    parser: Parser,
    /// Bytes received but not yet decodable as text.
    ///
    /// A chunk boundary can fall in the middle of a multi-byte character —
    /// nothing aligns TCP segments to UTF-8. The extension called
    /// `String::from_utf8` on each chunk and returned "Invalid UTF-8 chunk
    /// received" when that happened, so a model answering in Japanese, or
    /// emitting an emoji, could fail mid-response for no reason but where the
    /// packet split. The tail waits here for the rest of its character.
    tail: Vec<u8>,
    /// The connection closed. Distinct from the parser's `done`, which means
    /// the server sent `data: [DONE]`.
    closed: bool,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
}

/// Replaces any generation already in flight. Dropping the previous session
/// drops its stream, which is what tells the kernel to let go of the response.
pub fn start(stream: ByteStream, dialect: &'static crate::dialect::Dialect) {
    SESSION.with_borrow_mut(|slot| {
        *slot = Some(Session {
            stream,
            dialect,
            parser: Parser::default(),
            tail: Vec::new(),
            closed: false,
        });
    });
}

/// Runs `f` against the in-flight generation.
///
/// # Errors
///
/// Returns a message if no generation is in flight.
pub fn with<T>(f: impl FnOnce(&mut Session) -> Result<T, String>) -> Result<T, String> {
    SESSION.with_borrow_mut(|slot| match slot.as_mut() {
        Some(session) => f(session),
        None => Err("no generation is in flight; call llm.generate first".to_string()),
    })
}

impl Session {
    /// Everything the stream has produced since the last call.
    ///
    /// Returns as soon as it has something, so a caller relaying tokens does not
    /// wait on the whole response. An empty result with `finished()` false means
    /// the budget ran out with the peer silent.
    ///
    /// # Errors
    ///
    /// Returns a message if the transport fails or the peer sends bytes that
    /// are not valid UTF-8.
    pub fn pump(&mut self) -> Result<Vec<LlmEvent>, String> {
        let mut events = Vec::new();
        let mut pending = 0;

        while events.is_empty() && !self.finished() {
            match self.stream.read(READ_MAX) {
                // Empty is end-of-body, and only that. The kernel reports
                // "nothing yet" as PENDING precisely so this stays unambiguous.
                Ok(chunk) if chunk.is_empty() => {
                    self.closed = true;
                    // A final line with no trailing newline would otherwise be
                    // left in the buffer, so give the parser a terminator.
                    self.parser.push("\n", self.dialect, &mut events);
                }
                Ok(chunk) => {
                    let text = self.decode(&chunk)?;
                    self.parser.push(&text, self.dialect, &mut events);
                }
                Err(e) if e.code == PENDING => {
                    pending += 1;
                    if pending > MAX_PENDING {
                        return Err("LLM stream stalled: no data for 15s".to_string());
                    }
                }
                Err(e) => return Err(format!("HTTP stream read error: {}", e.message)),
            }
        }

        Ok(events)
    }

    /// Whether anything more can arrive.
    pub fn finished(&self) -> bool {
        self.closed || self.parser.done
    }

    /// Decodes as much of `tail + chunk` as forms whole characters, keeping any
    /// partial character for the next call.
    fn decode(&mut self, chunk: &[u8]) -> Result<String, String> {
        self.tail.extend_from_slice(chunk);
        let valid_up_to = match std::str::from_utf8(&self.tail) {
            Ok(text) => text.len(),
            // No `error_len` means the input simply stops mid-character; the
            // rest is still in flight. A length means the bytes are genuinely
            // malformed, which is the case the extension meant to catch.
            Err(e) if e.error_len().is_none() => e.valid_up_to(),
            Err(e) => return Err(format!("invalid UTF-8 in response body: {e}")),
        };
        let text = String::from_utf8_lossy(&self.tail[..valid_up_to]).into_owned();
        self.tail.drain(..valid_up_to);
        Ok(text)
    }
}
