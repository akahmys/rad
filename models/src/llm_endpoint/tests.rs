use super::{budget_chars_from_context_length, normalize_base_url};

#[test]
fn test_normalize_bare_root_is_unchanged() {
    assert_eq!(
        normalize_base_url("http://localhost:8080"),
        "http://localhost:8080"
    );
}

#[test]
fn test_normalize_strips_v1_suffix() {
    assert_eq!(
        normalize_base_url("http://localhost:8080/v1"),
        "http://localhost:8080"
    );
    assert_eq!(
        normalize_base_url("http://localhost:8080/v1/"),
        "http://localhost:8080"
    );
}

#[test]
fn test_normalize_strips_chat_completions_suffix() {
    assert_eq!(
        normalize_base_url("http://localhost:8080/v1/chat/completions"),
        "http://localhost:8080"
    );
}

#[test]
fn test_budget_chars_reserves_output_and_safety_margin() {
    // 8192 tokens - 1024 reserved for output = 7168, minus a 10% safety
    // margin (716, rounded down) = 6452, * 4 chars/token = 25808.
    assert_eq!(budget_chars_from_context_length(8192), 25_808);
}

#[test]
fn test_budget_chars_saturates_instead_of_underflowing_on_tiny_windows() {
    // A context window smaller than the reserved-output allowance must
    // not panic or wrap around; it should bottom out at zero.
    assert_eq!(budget_chars_from_context_length(100), 0);
    assert_eq!(budget_chars_from_context_length(0), 0);
}
