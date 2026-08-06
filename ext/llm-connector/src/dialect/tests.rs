use super::{AZURE, GEMINI, OPENAI, resolve};

#[test]
fn unknown_and_empty_names_fall_back_to_openai() {
    for name in [None, Some(""), Some("   "), Some("no-such-dialect")] {
        assert_eq!(resolve(name).path, OPENAI.path, "input: {name:?}");
    }
}

#[test]
fn known_names_resolve_case_insensitively() {
    assert_eq!(resolve(Some("gemini")).path, GEMINI.path);
    assert_eq!(resolve(Some("GEMINI")).path, GEMINI.path);
    assert_eq!(resolve(Some(" Azure ")).path, AZURE.path);
}

#[test]
fn struct_update_inherits_everything_but_the_stated_difference() {
    // The whole point of writing dialects as a delta from OPENAI: Gemini states
    // only `path`, so every parsing pointer must still match OPENAI's.
    assert_ne!(GEMINI.path, OPENAI.path);
    assert_eq!(GEMINI.auth_header, OPENAI.auth_header);
    assert_eq!(GEMINI.content_ptr, OPENAI.content_ptr);
    assert_eq!(GEMINI.tool_calls_ptr, OPENAI.tool_calls_ptr);
    assert_eq!(GEMINI.reasoning_ptr, OPENAI.reasoning_ptr);
}

#[test]
fn url_appends_path_and_substitutes_model() {
    assert_eq!(
        OPENAI.url("https://api.openai.com", "gpt-4"),
        "https://api.openai.com/v1/chat/completions"
    );
    assert_eq!(
        GEMINI.url("https://generativelanguage.googleapis.com", "gemini-3-pro"),
        "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions"
    );
    // Azure is the reason `{model}` substitution exists: the deployment name
    // lives in the path, not the body.
    assert_eq!(
        AZURE.url("https://acme.openai.azure.com", "my-deployment"),
        "https://acme.openai.azure.com/openai/deployments/my-deployment/chat/completions?api-version=2024-10-21"
    );
}

#[test]
fn headers_use_the_dialect_auth_scheme() {
    let openai = OPENAI.headers(Some("sk-test"));
    assert!(openai.contains(&("Authorization".to_string(), "Bearer sk-test".to_string())));

    // Azure sends the bare key under a different header name.
    let azure = AZURE.headers(Some("sk-test"));
    assert!(azure.contains(&("api-key".to_string(), "sk-test".to_string())));
    assert!(!azure.iter().any(|(name, _)| name == "Authorization"));
}

#[test]
fn headers_omit_auth_when_no_key_is_configured() {
    // Local llama.cpp servers take no key; sending an empty Bearer header
    // makes some of them reject the request.
    for key in [None, Some(""), Some("  ")] {
        let headers = OPENAI.headers(key);
        assert_eq!(headers.len(), 1, "input: {key:?}");
        assert_eq!(headers[0].0, "Content-Type");
    }
}

#[test]
fn api_key_is_trimmed_before_substitution() {
    let headers = OPENAI.headers(Some("  sk-test\n"));
    assert!(headers.contains(&("Authorization".to_string(), "Bearer sk-test".to_string())));
}

/// Regression guard for the migration: every profile that existed before
/// dialects were introduced has `dialect: None`, and must keep hitting exactly
/// the same URL with exactly the same headers as the hardcoded implementation
/// did. If this breaks, existing local llama.cpp setups break.
#[test]
fn pre_existing_profiles_are_bit_identical_to_the_old_hardcoded_behaviour() {
    let d = resolve(None);

    // The old code was `format!("{}/v1/chat/completions", normalize_base_url(..))`.
    assert_eq!(
        d.url("http://127.0.0.1:8080", "qwen2.5-coder-14b-instruct-q8_0"),
        "http://127.0.0.1:8080/v1/chat/completions"
    );
    // ...and the OpenAI fallback when only a key was configured.
    assert_eq!(
        d.url("https://api.openai.com", "gpt-4"),
        "https://api.openai.com/v1/chat/completions"
    );

    // The old header block was Content-Type plus `Authorization: Bearer {key}`,
    // and nothing at all when the key was absent or blank.
    assert_eq!(
        d.headers(Some("sk-abc")),
        vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("Authorization".to_string(), "Bearer sk-abc".to_string()),
        ]
    );
    assert_eq!(
        d.headers(None),
        vec![("Content-Type".to_string(), "application/json".to_string())]
    );

    // The old parser read these three pointers literally.
    assert_eq!(d.content_ptr, "/choices/0/delta/content");
    assert_eq!(d.reasoning_ptr, Some("/choices/0/delta/reasoning_content"));
    assert_eq!(d.tool_calls_ptr, "/choices/0/delta/tool_calls");
}
