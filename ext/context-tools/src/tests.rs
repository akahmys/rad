use self::bindings::exports::radcomp::extension::context_tools::Message;
use super::*;

fn msg(id: &str, role: &str, content: &str) -> Message {
    Message {
        node_id: Some(id.to_string()),
        role: role.to_string(),
        content: content.to_string(),
    }
}

#[test]
fn test_optimize_no_compression() {
    let request = OptimizationRequest {
        messages: vec![msg("1", "user", "Hello"), msg("2", "assistant", "Hi!")],
        max_history: None,
        max_content_chars: None,
    };
    let result = MyContextTools::optimize(request).unwrap();
    assert_eq!(result.optimized_messages.len(), 2);
    assert_eq!(result.summary, "No messages were compressed.");
}

#[test]
fn test_optimize_does_not_squash_parallel_tool_results() {
    // An assistant turn with 2 parallel tool_calls produces 2 consecutive
    // `tool` messages. Both must survive intact: dropping either would
    // leave the assistant message's tool_calls array referencing a
    // tool_call_id with no matching reply, which real LLM APIs reject.
    let request = OptimizationRequest {
        messages: vec![
            msg("1", "user", "Hello"),
            msg("2", "tool", "First tool result"),
            msg("3", "tool", "Second tool result"),
            msg("4", "assistant", "I got it."),
        ],
        max_history: None,
        max_content_chars: None,
    };
    let result = MyContextTools::optimize(request).unwrap();
    assert_eq!(result.optimized_messages.len(), 4);
    assert_eq!(result.optimized_messages[1].content, "First tool result");
    assert_eq!(result.optimized_messages[2].content, "Second tool result");
    assert_eq!(result.summary, "No messages were compressed.");
}

#[test]
fn test_optimize_windowing_only() {
    // 1 goal + 9 subsequent user/assistant turns = 10 messages, capped to 5.
    let mut messages = vec![msg("0", "user", "goal")];
    for i in 1..10 {
        let role = if i % 2 == 0 { "assistant" } else { "user" };
        messages.push(msg(&i.to_string(), role, "turn"));
    }
    let request = OptimizationRequest {
        messages,
        max_history: Some(5),
        max_content_chars: None,
    };
    let result = MyContextTools::optimize(request).unwrap();
    // First (goal) + most recent 4 = 5 messages retained.
    assert_eq!(result.optimized_messages.len(), 5);
    assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
    assert_eq!(result.optimized_messages[1].node_id.as_deref(), Some("6"));
    assert_eq!(result.optimized_messages[4].node_id.as_deref(), Some("9"));
    assert!(result.summary.contains("Windowed history from 10 to 5"));
}

#[test]
fn test_optimize_windowing_under_limit_is_noop() {
    let request = OptimizationRequest {
        messages: vec![msg("1", "user", "Hello"), msg("2", "assistant", "Hi!")],
        max_history: Some(10),
        max_content_chars: None,
    };
    let result = MyContextTools::optimize(request).unwrap();
    assert_eq!(result.optimized_messages.len(), 2);
    assert_eq!(result.summary, "No messages were compressed.");
}

#[test]
fn test_optimize_size_based_windowing_only() {
    // No message-count cap, but a tiny char budget: a single large
    // tool result must get trimmed away even though there are only 3
    // messages total, since count-based windowing alone would never
    // have caught this (local models can overflow well under any
    // reasonable message-count cap).
    let messages = vec![
        msg("0", "user", "goal"),
        msg("1", "tool", &"x".repeat(1000)),
        msg("2", "assistant", "done"),
    ];
    let request = OptimizationRequest {
        messages,
        max_history: None,
        max_content_chars: Some(20),
    };
    let result = MyContextTools::optimize(request).unwrap();
    // Budget (20 chars) can't fit "goal" (4) + the 1000-char tool
    // result, so only the most recent message that still fits ("done",
    // 4 chars) survives alongside the goal.
    assert_eq!(result.optimized_messages.len(), 2);
    assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
    assert_eq!(result.optimized_messages[1].node_id.as_deref(), Some("2"));
    assert!(result.summary.contains("Windowed history from 3 to 2"));
}

#[test]
fn test_optimize_size_based_windowing_under_budget_is_noop() {
    let request = OptimizationRequest {
        messages: vec![msg("1", "user", "Hello"), msg("2", "assistant", "Hi!")],
        max_history: None,
        max_content_chars: Some(10_000),
    };
    let result = MyContextTools::optimize(request).unwrap();
    assert_eq!(result.optimized_messages.len(), 2);
    assert_eq!(result.summary, "No messages were compressed.");
}

#[test]
fn test_optimize_combined_constraints_takes_more_restrictive() {
    // 6 messages: count-based (max_history=5) would only drop 1 message,
    // but the char budget is tight enough to force a smaller window.
    // The more restrictive (smaller) result must win.
    let messages = vec![
        msg("0", "user", "goal"),
        msg("1", "assistant", &"a".repeat(50)),
        msg("2", "user", &"b".repeat(50)),
        msg("3", "assistant", &"c".repeat(50)),
        msg("4", "user", &"d".repeat(50)),
        msg("5", "assistant", "short"),
    ];
    let request = OptimizationRequest {
        messages,
        max_history: Some(5),
        max_content_chars: Some(30),
    };
    let result = MyContextTools::optimize(request).unwrap();
    // Char budget (30) fits goal (4) + "short" (5) = 9 chars, but
    // adding "d".repeat(50) on top (59 total) would blow it. Count-based
    // alone would have kept 5 messages, so the size constraint must win.
    assert_eq!(result.optimized_messages.len(), 2);
    assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
    assert_eq!(result.optimized_messages[1].node_id.as_deref(), Some("5"));
}

#[test]
fn test_optimize_windowing_preserves_tool_pairs_in_window() {
    // Goal + a turn with 2 parallel tool_calls; windowing trims the
    // list but must not further collapse the surviving tool pair.
    let messages = vec![
        msg("0", "user", "goal"),
        msg("1", "tool", "old result"),
        msg("2", "assistant", "old reply"),
        msg("3", "tool", "result A"),
        msg("4", "tool", "result B"),
        msg("5", "assistant", "final reply"),
    ];
    let request = OptimizationRequest {
        messages,
        max_history: Some(4),
        max_content_chars: None,
    };
    let result = MyContextTools::optimize(request).unwrap();
    // Window keeps [goal, result A, result B, final reply] (4 messages);
    // both tool results in the surviving pair remain, unsquashed.
    assert_eq!(result.optimized_messages.len(), 4);
    assert_eq!(result.optimized_messages[0].node_id.as_deref(), Some("0"));
    assert_eq!(result.optimized_messages[1].content, "result A");
    assert_eq!(result.optimized_messages[2].content, "result B");
    assert_eq!(result.optimized_messages[3].content, "final reply");
    assert!(result.summary.contains("Windowed history from 6 to 4"));
    assert!(!result.summary.contains("Compressed"));
}
