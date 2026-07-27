use super::*;
use crate::tool::{ToolCall, ToolCallFunction};

fn assistant_with_call(name: &str, arguments: &str) -> Message {
    Message {
        role: "assistant".to_string(),
        content: None,
        name: None,
        tool_call_id: None,
        tool_calls: Some(vec![ToolCall {
            id: "call_1".to_string(),
            tool_type: "function".to_string(),
            function: ToolCallFunction {
                name: name.to_string(),
                arguments: arguments.to_string(),
            },
        }]),
    }
}

#[test]
fn test_build_digest_addendum_none_when_no_activity() {
    let messages = vec![Message {
        role: "user".to_string(),
        content: Some("hello".to_string()),
        name: None,
        tool_call_id: None,
        tool_calls: None,
    }];
    assert!(build_digest_addendum(&messages).is_none());
}

#[test]
fn test_build_digest_addendum_extracts_file_path() {
    let messages =
        vec![assistant_with_call("write_file", r#"{"path": "src/main.rs", "content": "x"}"#)];
    let digest = build_digest_addendum(&messages).unwrap();
    assert!(digest.contains("Files touched this session: src/main.rs"), "{digest}");
    assert!(!digest.contains("Commands run"), "{digest}");
}

#[test]
fn test_build_digest_addendum_extracts_command() {
    let messages = vec![assistant_with_call("execute_command", r#"{"command": "cargo test"}"#)];
    let digest = build_digest_addendum(&messages).unwrap();
    assert!(digest.contains("Commands run this session: `cargo test`"), "{digest}");
    assert!(!digest.contains("Files touched"), "{digest}");
}

#[test]
fn test_build_digest_addendum_recognizes_alternate_key_names() {
    let messages = vec![
        assistant_with_call("edit", r#"{"file_path": "a.py"}"#),
        assistant_with_call("shell", r#"{"cmd": "ls -la"}"#),
    ];
    let digest = build_digest_addendum(&messages).unwrap();
    assert!(digest.contains("a.py"), "{digest}");
    assert!(digest.contains("ls -la"), "{digest}");
}

#[test]
fn test_build_digest_addendum_deduplicates_repeated_files_and_commands() {
    let messages = vec![
        assistant_with_call("write_file", r#"{"path": "a.rs"}"#),
        assistant_with_call("write_file", r#"{"path": "a.rs"}"#),
        assistant_with_call("execute_command", r#"{"command": "cargo build"}"#),
        assistant_with_call("execute_command", r#"{"command": "cargo build"}"#),
    ];
    let digest = build_digest_addendum(&messages).unwrap();
    assert_eq!(digest.matches("a.rs").count(), 1, "{digest}");
    assert_eq!(digest.matches("cargo build").count(), 1, "{digest}");
}

#[test]
fn test_build_digest_addendum_ignores_non_assistant_and_malformed_arguments() {
    let mut tool_reply = assistant_with_call("write_file", "not valid json");
    tool_reply.role = "tool".to_string();
    let messages = vec![
        assistant_with_call("write_file", "not valid json"),
        tool_reply,
    ];
    assert!(build_digest_addendum(&messages).is_none());
}

#[test]
fn test_build_digest_addendum_caps_to_most_recent_items() {
    let messages: Vec<Message> = (0..40)
        .map(|i| assistant_with_call("write_file", &format!(r#"{{"path": "file{i}.rs"}}"#)))
        .collect();
    let digest = build_digest_addendum(&messages).unwrap();
    assert!(!digest.contains("file0.rs"), "oldest entries should be dropped: {digest}");
    assert!(digest.contains("file39.rs"), "most recent entry should survive: {digest}");
}
