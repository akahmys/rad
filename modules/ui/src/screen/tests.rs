//! The buffering rules. Printing itself is not observable from here — the
//! module writes to inherited stdout — so what is asserted is the decision:
//! which state the terminal is in, and whether a log was deferred or emitted.
use super::{State, deferred_count, set_state, state, write_log, write_token};

/// One `SCREEN` per thread, reused across tests on that thread.
fn idle() {
    set_state(State::Idle);
}

#[test]
fn a_log_while_idle_is_not_deferred() {
    idle();
    write_log("visible".to_string());
    assert_eq!(deferred_count(), 0);
}

/// The rule the whole state machine exists for: a log arriving mid-response
/// must not land in the middle of it.
#[test]
fn a_log_while_streaming_is_held_back() {
    idle();
    set_state(State::Streaming);
    write_log("held".to_string());
    assert_eq!(deferred_count(), 1);
}

#[test]
fn a_log_while_thinking_is_held_back_too() {
    idle();
    set_state(State::Thinking);
    write_log("held".to_string());
    assert_eq!(deferred_count(), 1);
}

/// Going idle is what releases them, and it must release *all* of them.
#[test]
fn returning_to_idle_flushes_everything_deferred() {
    idle();
    set_state(State::Streaming);
    write_log("one".to_string());
    write_log("two".to_string());
    assert_eq!(deferred_count(), 2);

    set_state(State::Idle);
    assert_eq!(deferred_count(), 0);
}

/// A token moves the terminal on its own — that transition is what erases the
/// thinking indicator before the first token appears.
#[test]
fn writing_a_token_moves_the_terminal_to_streaming() {
    idle();
    set_state(State::Thinking);
    write_token("hi");
    assert_eq!(state(), State::Streaming);
}

/// Re-entering the same state is a no-op. Without this, every token would
/// re-run the `Streaming` transition and erase the line it just printed.
#[test]
fn setting_the_state_it_is_already_in_does_nothing() {
    idle();
    set_state(State::Streaming);
    write_log("held".to_string());
    set_state(State::Streaming);
    assert_eq!(deferred_count(), 1, "a no-op transition flushed the buffer");
}

#[test]
fn state_names_round_trip() {
    for s in [State::Idle, State::Thinking, State::Streaming] {
        assert_eq!(State::parse(s.name()), Some(s));
    }
    assert_eq!(State::parse("nonsense"), None);
}
