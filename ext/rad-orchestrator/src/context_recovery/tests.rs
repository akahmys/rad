use super::{is_context_exhaustion, next_budget_scale_percent, scale_budget};

#[test]
fn test_recognizes_common_backend_phrasings() {
    // OpenAI-style
    assert!(is_context_exhaustion(
        "This model's maximum context length is 8192 tokens, however you requested 9000"
    ));
    // Anthropic-style
    assert!(is_context_exhaustion("prompt is too long: 210000 tokens"));
    // llama.cpp-style
    assert!(is_context_exhaustion(
        "the request exceeds the available context size (n_ctx = 4096)"
    ));
    // Generic wording
    assert!(is_context_exhaustion("Context window exceeded"));
}

#[test]
fn test_is_case_insensitive() {
    assert!(is_context_exhaustion("MAXIMUM CONTEXT LENGTH EXCEEDED"));
}

#[test]
fn test_does_not_match_unrelated_failures() {
    // A false positive here would burn retries on a permanent failure.
    assert!(!is_context_exhaustion(
        "429 Too Many Requests: rate limited"
    ));
    assert!(!is_context_exhaustion(
        "Model stopped because it reached the maximum output token limit"
    ));
    assert!(!is_context_exhaustion("connection refused"));
    assert!(!is_context_exhaustion("unknown error"));
}

#[test]
fn test_backoff_compounds_downward() {
    let first = next_budget_scale_percent(100);
    let second = next_budget_scale_percent(first);
    assert_eq!(first, 60);
    assert_eq!(second, 36);
    assert!(second < first);
}

#[test]
fn test_backoff_never_reaches_zero() {
    let mut scale = 100;
    for _ in 0..50 {
        scale = next_budget_scale_percent(scale);
    }
    assert!(scale >= 1, "scale collapsed to {scale}");
}

#[test]
fn test_scale_budget_shrinks_and_is_a_noop_at_full_scale() {
    assert_eq!(scale_budget(10_000, 100), 10_000);
    assert_eq!(scale_budget(10_000, 60), 6_000);
    assert_eq!(scale_budget(10_000, 36), 3_600);
}

#[test]
fn test_scale_budget_never_returns_zero() {
    // A zero budget would make compaction discard everything.
    assert!(scale_budget(10, 1) >= 1);
    assert!(scale_budget(1, 1) >= 1);
}
