// Provider wire-format differences, as `const` data rather than code branches.
//
// Everything that varies across OpenAI-compatible providers is one of four
// things: the URL path, the auth header, extra static headers, and where the
// interesting fields sit in the SSE payload. All four fit in a struct, so a new
// provider is a struct literal rather than a new branch.
//
// Deliberately not a config file. Adding a provider means rebuilding one wasm
// crate, which costs nothing here; a config schema would cost validation,
// migration, and documentation. If `rad` ever ships binaries whose users add
// providers themselves, this struct is the shape that config would deserialize
// into — the work is not wasted.

/// One provider's wire format. Every field except `path` defaults to the
/// OpenAI-compatible behaviour, so a new dialect is written as the difference
/// from [`OPENAI`] via struct update syntax.
pub struct Dialect {
    /// Appended to the normalized base URL. `{model}` is substituted.
    pub path: &'static str,
    pub auth_header: &'static str,
    /// `{key}` is substituted with the resolved API key.
    pub auth_format: &'static str,
    pub extra_headers: &'static [(&'static str, &'static str)],
    /// JSON Pointers into one SSE `data:` object.
    pub content_ptr: &'static str,
    pub reasoning_ptr: Option<&'static str>,
    pub tool_calls_ptr: &'static str,
}

pub const OPENAI: Dialect = Dialect {
    path: "/v1/chat/completions",
    auth_header: "Authorization",
    auth_format: "Bearer {key}",
    extra_headers: &[],
    content_ptr: "/choices/0/delta/content",
    // Not an OpenAI-compatible standard: `reasoning_content` originates with
    // DeepSeek and is carried by llama.cpp and xAI. Providers without it simply
    // never match, and thinking output stays empty.
    reasoning_ptr: Some("/choices/0/delta/reasoning_content"),
    tool_calls_ptr: "/choices/0/delta/tool_calls",
};

pub const GEMINI: Dialect = Dialect {
    path: "/v1beta/openai/chat/completions",
    ..OPENAI
};

pub const AZURE: Dialect = Dialect {
    path: "/openai/deployments/{model}/chat/completions?api-version=2024-10-21",
    auth_header: "api-key",
    auth_format: "{key}",
    ..OPENAI
};

/// Lookup table. An unknown name falls back to [`OPENAI`] rather than failing:
/// a typo should not take the agent offline, and the `OpenAI` shape is what most
/// endpoints speak.
const ALL: &[(&str, &Dialect)] = &[("openai", &OPENAI), ("gemini", &GEMINI), ("azure", &AZURE)];

/// Resolves a dialect name. `None`, empty, or unrecognized all yield [`OPENAI`].
#[must_use]
pub fn resolve(name: Option<&str>) -> &'static Dialect {
    let Some(name) = name.map(str::trim).filter(|n| !n.is_empty()) else {
        return &OPENAI;
    };
    for (candidate, dialect) in ALL {
        if candidate.eq_ignore_ascii_case(name) {
            return dialect;
        }
    }
    eprintln!("[llm-connector] Unknown dialect '{name}', falling back to 'openai'");
    &OPENAI
}

impl Dialect {
    /// Builds the request URL from a normalized base URL.
    #[must_use]
    pub fn url(&self, base_url: &str, model: &str) -> String {
        format!("{base_url}{}", self.path.replace("{model}", model))
    }

    /// Builds the request headers. Returns `Content-Type` plus the auth header
    /// (only when a key is present) plus any dialect-specific extras.
    #[must_use]
    pub fn headers(&self, api_key: Option<&str>) -> Vec<(String, String)> {
        let mut headers = vec![("Content-Type".to_string(), "application/json".to_string())];
        if let Some(key) = api_key.map(str::trim).filter(|k| !k.is_empty()) {
            headers.push((
                self.auth_header.to_string(),
                self.auth_format.replace("{key}", key),
            ));
        }
        for (name, value) in self.extra_headers {
            headers.push(((*name).to_string(), (*value).to_string()));
        }
        headers
    }
}

#[cfg(test)]
mod tests;
