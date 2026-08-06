//! L3 (context exhaustion) recovery policy — see ARCHITECTURE.md §5.1.2.
//!
//! Compaction sizes each request proactively from a `chars/4` approximation
//! rather than a real tokenizer, so it can occasionally under-estimate (dense
//! code, CJK) and produce a request the backend rejects for exceeding its
//! context window. This module owns the *reactive* half of that story:
//! recognizing such a rejection and deciding whether to retry the turn with a
//! smaller budget.
//!
//! The retry is deliberately bounded. An unbounded "shrink and retry" would
//! spin forever whenever the failure is not actually budget-related but merely
//! *looks* like it — the same reasoning behind the tool-side circuit breaker.

#[cfg(test)]
mod tests;

/// How many times a single turn may be re-issued with a reduced budget
/// before giving up and surfacing the error to the user.
pub const MAX_CONTEXT_RETRIES: u32 = 2;

/// Percentage the character budget is multiplied by on each retry. 60%
/// compounds to 60% then 36% of the original budget across the two allowed
/// attempts — aggressive enough to escape a mis-estimate in few steps, since
/// each attempt costs a full round-trip to the backend.
const BACKOFF_PERCENT: u32 = 60;

/// Substrings that indicate a backend rejected the request specifically for
/// exceeding its context window, matched case-insensitively.
///
/// Deliberately narrow: a false positive here converts an unrelated permanent
/// failure into wasted retries, so generic words like "exceeds" or "limit"
/// (which also appear in rate-limit and output-token errors) are excluded.
const EXHAUSTION_MARKERS: &[&str] = &[
    "context length",
    "context window",
    "context size",
    "maximum context",
    "prompt is too long",
    "too many tokens",
    "n_ctx",
];

/// Whether an LLM error payload names context exhaustion as the cause.
#[must_use]
pub fn is_context_exhaustion(payload: &str) -> bool {
    let lowered = payload.to_lowercase();
    EXHAUSTION_MARKERS
        .iter()
        .any(|marker| lowered.contains(marker))
}

/// The budget scale (as a percentage) after one more retry at `current`.
#[must_use]
pub fn next_budget_scale_percent(current: u32) -> u32 {
    // Saturate at 1% rather than 0: a zero budget would make compaction
    // discard everything, turning a recoverable turn into a guaranteed-useless
    // one.
    (current * BACKOFF_PERCENT / 100).max(1)
}

/// Applies a percentage scale to a character budget, saturating at 1 for the
/// same reason as [`next_budget_scale_percent`].
#[must_use]
pub fn scale_budget(base_chars: u32, scale_percent: u32) -> u32 {
    if scale_percent >= 100 {
        return base_chars;
    }
    let scaled = u64::from(base_chars) * u64::from(scale_percent) / 100;
    u32::try_from(scaled).unwrap_or(base_chars).max(1)
}
