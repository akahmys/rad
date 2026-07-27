use super::*;

fn msg(id: &str, role: &str, content: &str) -> Message {
    Message {
        node_id: Some(id.to_string()),
        role: role.to_string(),
        content: content.to_string(),
    }
}

#[test]
fn test_clear_stale_tool_results_fits_budget_without_dropping_messages() {
    // An old, large tool result sits well before the tail; clearing it in
    // place (not dropping the message) should be enough to fit the
    // budget, so windowing afterward has nothing left to remove.
    let messages = vec![
        msg("0", "user", "goal"),
        msg("1", "tool", &"x".repeat(200)),
        msg("2", "assistant", "ack"),
        msg("3", "tool", "small recent result"),
        msg("4", "assistant", "done"),
    ];
    let mut summary_parts = Vec::new();
    let cleared = clear_stale_tool_results(messages, Some(100), &mut summary_parts);

    assert_eq!(cleared.len(), 5);
    assert!(cleared[1].content.starts_with("[tool output cleared"));
    assert_eq!(cleared[3].content, "small recent result");
    assert!(summary_parts.iter().any(|s| s.contains("Cleared 1 stale tool result")));

    // Windowing afterward finds everything already fits — a no-op.
    let windowed = apply_history_window(cleared, None, Some(100), &mut Vec::new());
    assert_eq!(windowed.len(), 5);
}

#[test]
fn test_clear_stale_tool_results_never_clears_the_sole_tool_result() {
    // Only one tool message exists, so clearing must never touch it
    // (always preserves at least the most recent tool result in full) —
    // windowing is left as the only mechanism that can shrink it away.
    let messages = vec![msg("0", "user", "goal"), msg("1", "tool", &"x".repeat(500))];
    let mut summary_parts = Vec::new();
    let cleared = clear_stale_tool_results(messages, Some(10), &mut summary_parts);

    assert!(
        !cleared.iter().any(|m| m.content.starts_with("[tool output cleared")),
        "the sole tool result must never be cleared: {cleared:?}"
    );
    assert!(summary_parts.is_empty());
}

#[test]
fn test_apply_history_window_reinstates_relevant_earlier_turn_when_budget_allows() {
    // Goal mentions "database migration". An early turn discusses exactly
    // that (and would normally be dropped by count-based windowing), while
    // several irrelevant filler turns sit in between. With slack left over
    // in the char budget after the mandatory tail, the relevant early turn
    // should be reinstated ahead of the irrelevant ones.
    let messages = vec![
        msg("0", "user", "please investigate the database migration failure"),
        msg("1", "assistant", "found it: the migration script has a typo"),
        msg("2", "user", "unrelated aside about the weather today"),
        msg("3", "assistant", "sure, it looks sunny"),
        msg("4", "user", "another unrelated aside about lunch plans"),
        msg("5", "assistant", "tacos sound great"),
        msg("6", "user", "ok back to the task, what did you find"),
        msg("7", "assistant", "final summary of findings"),
    ];
    let mut summary_parts = Vec::new();
    // max_history=3 would keep only [goal, turn6, turn7] by position alone.
    let result = apply_history_window(messages, Some(3), Some(400), &mut summary_parts);

    assert!(
        result.iter().any(|m| m.content.contains("migration script")),
        "the lexically relevant earlier turn should have been reinstated: {result:?}"
    );
    assert!(summary_parts.iter().any(|s| s.contains("Reinstated")), "{summary_parts:?}");
    // Chronological order must be preserved even with a turn spliced back in.
    let node_ids: Vec<_> = result.iter().map(|m| m.node_id.clone().unwrap()).collect();
    let mut sorted_ids = node_ids.clone();
    sorted_ids.sort_by_key(|s| s.parse::<u32>().unwrap());
    assert_eq!(node_ids, sorted_ids, "messages must stay in chronological order");
}

#[test]
fn test_apply_history_window_does_not_reinstate_without_lexical_overlap() {
    let messages = vec![
        msg("0", "user", "please investigate the database migration failure"),
        msg("1", "assistant", "unrelated filler about nothing in particular"),
        msg("2", "user", "another aside"),
        msg("3", "assistant", "yet more filler"),
        msg("4", "user", "final question"),
        msg("5", "assistant", "final answer"),
    ];
    let mut summary_parts = Vec::new();
    let result = apply_history_window(messages, Some(3), Some(400), &mut summary_parts);
    assert!(!summary_parts.iter().any(|s| s.contains("Reinstated")), "{summary_parts:?}");
    assert_eq!(result.len(), 3);
}

#[test]
fn test_apply_history_window_does_not_reinstate_without_size_budget() {
    // No `max_content_chars` at all — relevance retention has no size
    // signal to work from and must stay a strict no-op, identical to
    // plain positional windowing.
    let messages = vec![
        msg("0", "user", "please investigate the database migration failure"),
        msg("1", "assistant", "found it: the migration script has a typo"),
        msg("2", "user", "another aside"),
        msg("3", "assistant", "filler"),
        msg("4", "user", "final question"),
        msg("5", "assistant", "final answer"),
    ];
    let mut summary_parts = Vec::new();
    let result = apply_history_window(messages, Some(3), None, &mut summary_parts);
    assert!(!summary_parts.iter().any(|s| s.contains("Reinstated")));
    assert_eq!(result.len(), 3);
}
