//! Shared LLM-endpoint utilities used on both sides of the host/extension
//! boundary: URL normalization (core's `/llm` admin commands and the
//! `llm-connector` extension each need to turn a user-supplied base URL
//! into a specific endpoint path) and context-window budgeting (core
//! detects a model's context length; `rad-orchestrator` turns it into a
//! character budget for `context-tools`). Living here means neither side
//! reimplements the other's copy.

#[cfg(test)]
mod tests;

/// Strips a trailing `/chat/completions` and/or `/v1` from a user-supplied
/// base URL to get the server's bare root, regardless of which shape the
/// user provided it in (bare root, `.../v1`, or `.../v1/chat/completions`).
/// Callers append whatever path they actually need (`/v1/models`, `/props`,
/// `/api/show`, `/v1/chat/completions`, ...) to the result.
#[must_use]
pub fn normalize_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    let trimmed = trimmed.strip_suffix("/chat/completions").unwrap_or(trimmed).trim_end_matches('/');
    let trimmed = trimmed.strip_suffix("/v1").unwrap_or(trimmed).trim_end_matches('/');
    trimmed.to_string()
}

/// Converts a model's raw context window (tokens) into a conservative
/// character budget for `context-tools`' size-based windowing: reserves
/// headroom for the model's own output plus a safety margin (chars-per-token
/// is only a rough approximation, since not every local backend exposes a
/// tokenizer `rad` can call), then converts the remainder to characters.
#[must_use]
pub fn budget_chars_from_context_length(context_length: u32) -> u32 {
    const RESERVED_OUTPUT_TOKENS: u32 = 1024;
    const SAFETY_MARGIN_PERCENT: u32 = 10;
    const CHARS_PER_TOKEN_APPROX: u32 = 4;

    let usable = context_length.saturating_sub(RESERVED_OUTPUT_TOKENS);
    let usable = usable - usable.saturating_mul(SAFETY_MARGIN_PERCENT) / 100;
    usable.saturating_mul(CHARS_PER_TOKEN_APPROX)
}
