//! Pattern matching only. Nothing here calls [`Rules::fetch`]: it reaches the
//! kernel through generated bindings that have no host behind them in a native
//! test binary. The config *parsing* is covered through [`string_array`]
//! instead, which is the part that can actually be wrong.
use super::{Rules, string_array};

fn rules(patterns: &[&str]) -> Rules {
    Rules {
        block_command_patterns: patterns.iter().map(|p| (*p).to_string()).collect(),
    }
}

#[test]
fn an_empty_policy_blocks_nothing() {
    assert!(rules(&[]).refuse("rm -rf /").is_none());
}

#[test]
fn a_matching_substring_refuses_and_names_the_pattern() {
    let refusal = rules(&["blocked_command"])
        .refuse(r#"{"command":"blocked_command --now"}"#)
        .expect("a matching pattern must refuse");
    assert!(
        refusal.contains("blocked_command"),
        "the refusal has to say which pattern matched, or the model \
         cannot tell a policy denial from a tool failure: {refusal}"
    );
}

#[test]
fn a_non_matching_argument_is_allowed() {
    assert!(rules(&["blocked_command"]).refuse("ls -la").is_none());
}

/// The first pattern in the list is not privileged; any one of them refuses.
#[test]
fn a_later_pattern_still_matches() {
    assert!(
        rules(&["never", "blocked.txt"])
            .refuse("write blocked.txt")
            .is_some()
    );
}

/// Substring, not word or path matching — inherited from the extension
/// deliberately, so a ported config keeps behaving as it did.
#[test]
fn matching_is_a_plain_substring() {
    assert!(rules(&["cat"]).refuse("concatenate").is_some());
}

#[test]
fn a_missing_key_yields_no_patterns() {
    let config = serde_json::json!({});
    assert!(string_array(&config, "block_command_patterns").is_empty());
}

#[test]
fn a_non_array_value_yields_no_patterns_rather_than_failing() {
    let config = serde_json::json!({ "block_command_patterns": "blocked" });
    assert!(string_array(&config, "block_command_patterns").is_empty());
}

#[test]
fn non_string_entries_are_dropped_and_the_rest_kept() {
    let config = serde_json::json!({ "block_command_patterns": ["a", 7, "b"] });
    assert_eq!(
        string_array(&config, "block_command_patterns"),
        vec!["a".to_string(), "b".to_string()]
    );
}
