//! Endpoint resolution — the half of the transport that stayed on the host.

use super::resolve_base_url;
use crate::wasm::rpc_meta::ActiveLlmProfile;

fn profile(base_url: Option<&str>, api_key: Option<&str>) -> ActiveLlmProfile {
    ActiveLlmProfile {
        model: None,
        context_length: None,
        base_url: base_url.map(ToString::to_string),
        api_key: api_key.map(ToString::to_string),
        dialect: None,
    }
}

#[test]
fn a_configured_base_url_is_normalized() {
    // `normalize_base_url` strips a trailing `/v1`, which the dialect then
    // re-adds as part of its path. Without it the request goes to `/v1/v1/...`.
    let resolved = resolve_base_url(&profile(Some("http://localhost:8080/v1"), None)).unwrap();
    assert_eq!(resolved, "http://localhost:8080");
}

#[test]
fn a_key_with_no_base_url_falls_back_to_openai() {
    let resolved = resolve_base_url(&profile(None, Some("sk-x"))).unwrap();
    assert_eq!(resolved, "https://api.openai.com");
}

#[test]
fn a_blank_base_url_is_treated_as_unset() {
    let err = resolve_base_url(&profile(Some("   "), None)).unwrap_err();
    assert!(err.contains("No LLM endpoint configured"), "{err}");
}

/// The exact wording, not merely "an error".
///
/// `tests/llm_endpoint_config_tests.rs` asserts this string is **absent**
/// from the conversation, which is a check that passes vacuously the moment the
/// wording drifts. This is the other half: something has to fail when it
/// changes, or that test stops meaning anything.
#[test]
fn the_unconfigured_message_is_the_one_the_eager_load_test_looks_for() {
    let err = resolve_base_url(&profile(None, None)).unwrap_err();
    assert_eq!(
        err,
        "No LLM endpoint configured. Set one up with /llm add <name> <url>."
    );
}
