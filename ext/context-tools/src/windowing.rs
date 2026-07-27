//! Pure message-list compaction logic, split out of `lib.rs` to stay under
//! the 300-line file limit. No `host_rpc`/WIT dependency beyond the shared
//! `Message` record — everything here is deterministic and unit-testable
//! without a WASM runtime.
use crate::bindings::exports::radcomp::extension::context_tools::Message;

#[cfg(test)]
mod tests;

/// Count-based cutoff: the index into `messages` where the "recent tail"
/// must start to keep at most `max_history` messages total (first message
/// + tail). `None` if this constraint doesn't require trimming.
pub(crate) fn count_based_start(len: usize, max_history: Option<u32>) -> Option<usize> {
    let max_history = usize::try_from(max_history?).unwrap_or(usize::MAX);
    if len <= max_history {
        return None;
    }
    let remaining_len = len - 1;
    let limit = max_history.saturating_sub(1);
    Some(if remaining_len > limit { len - limit } else { 1 })
}

/// Size-based cutoff: the index into `messages` where the "recent tail"
/// must start so that the first message's content plus the tail's content
/// stays within `max_content_chars`. Walks backward from the end, greedily
/// keeping the most recent messages that still fit. `None` if this
/// constraint doesn't require trimming.
pub(crate) fn size_based_start(messages: &[Message], max_content_chars: Option<u32>) -> Option<usize> {
    let budget = usize::try_from(max_content_chars?).unwrap_or(usize::MAX);
    let mut used = messages[0].content.len();
    let mut start = messages.len();
    for (i, msg) in messages.iter().enumerate().skip(1).rev() {
        let next_used = used + msg.content.len();
        if next_used > budget {
            break;
        }
        used = next_used;
        start = i;
    }
    if start <= 1 { None } else { Some(start) }
}

/// Size-aware stale tool-result clearing (Phase 51-1): when the full
/// message list exceeds `max_content_chars`, replaces the content of older
/// `tool`-role messages (oldest first, always keeping the most recent one
/// intact) with a short placeholder, instead of relying solely on
/// windowing to drop whole messages. A single large tool output can
/// dominate the budget while sitting inside the "recent tail" windowing
/// always keeps — clearing it in place frees room for windowing to retain
/// more of the actual conversation. The message itself is never removed,
/// only shrunk, so the `tool_calls`/`tool` pairing invariant (AWU 909) is
/// untouched. No-op if `max_content_chars` is `None` (no size budget
/// known) or nothing is worth clearing.
pub(crate) fn clear_stale_tool_results(
    mut messages: Vec<Message>,
    max_content_chars: Option<u32>,
    summary_parts: &mut Vec<String>,
) -> Vec<Message> {
    const MIN_WORTHWHILE_SAVINGS: usize = 80;

    let Some(budget) = max_content_chars.map(|b| usize::try_from(b).unwrap_or(usize::MAX)) else {
        return messages;
    };
    let total: usize = messages.iter().map(|m| m.content.len()).sum();
    if total <= budget {
        return messages;
    }

    let tool_indices: Vec<usize> = messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.role == "tool")
        .map(|(i, _)| i)
        .collect();
    if tool_indices.len() <= 1 {
        return messages;
    }

    let mut freed = 0usize;
    let mut cleared_count = 0usize;
    for &idx in &tool_indices[..tool_indices.len() - 1] {
        if total.saturating_sub(freed) <= budget {
            break;
        }
        let original_len = messages[idx].content.len();
        if original_len < MIN_WORTHWHILE_SAVINGS {
            continue;
        }
        let placeholder =
            format!("[tool output cleared to save context: {original_len} chars removed]");
        freed += original_len.saturating_sub(placeholder.len());
        messages[idx].content = placeholder;
        cleared_count += 1;
    }

    if cleared_count > 0 {
        summary_parts.push(format!(
            "Cleared {cleared_count} stale tool result(s) to fit the size budget, freeing ~{freed} chars."
        ));
    }

    messages
}

/// Windows the message list down to the first message (the original goal)
/// plus as much of the recent tail as fits under both `max_history` (a
/// message count) and `max_content_chars` (a size budget derived from the
/// active model's real context window). Whichever constraint is more
/// restrictive wins. `None`/`None` disables windowing entirely.
pub(crate) fn apply_history_window(
    messages: Vec<Message>,
    max_history: Option<u32>,
    max_content_chars: Option<u32>,
    summary_parts: &mut Vec<String>,
) -> Vec<Message> {
    if messages.is_empty() {
        return messages;
    }

    let count_start = count_based_start(messages.len(), max_history);
    let size_start = size_based_start(&messages, max_content_chars);

    let start_idx = match (count_start, size_start) {
        (None, None) => return messages,
        (Some(a), None) => a,
        (None, Some(b)) => b,
        (Some(a), Some(b)) => a.max(b),
    };

    let original_len = messages.len();
    let trimmed = trim_with_relevance_retention(&messages, start_idx, max_content_chars, summary_parts);

    summary_parts.push(format!(
        "Windowed history from {original_len} to {} messages (kept first + most recent).",
        trimmed.len()
    ));

    trimmed
}

/// Lowercased alphanumeric "words" of 3+ characters, for a simple, no-ML
/// keyword-overlap heuristic (Phase 51-2) — deliberately not
/// stemmed/stopword-filtered/BM25-weighted; this only needs to be good
/// enough to distinguish "mentions the goal's actual vocabulary" from
/// "doesn't," not to be a real search-relevance engine.
fn tokenize_lowercase_words(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 2)
        .map(str::to_lowercase)
        .collect()
}

/// Count of distinct goal words that also appear anywhere in `turn`.
fn relevance_score(turn: &[Message], goal_words: &std::collections::HashSet<String>) -> usize {
    let turn_words: std::collections::HashSet<String> =
        turn.iter().flat_map(|m| tokenize_lowercase_words(&m.content)).collect();
    goal_words.intersection(&turn_words).count()
}

/// Groups `messages` into turns: each `user` message starts a new turn,
/// and every following non-`user` message (assistant replies, `tool_calls`
/// results) attaches to that same turn. Guarantees a reinstated turn is
/// always taken or left whole, so relevance-based retention can never
/// split an `assistant`/`tool_calls` message from its paired `tool` reply
/// (the invariant AWU 909 protects).
fn split_into_turns(messages: &[Message]) -> Vec<Vec<Message>> {
    let mut turns: Vec<Vec<Message>> = Vec::new();
    for m in messages {
        if m.role == "user" || turns.is_empty() {
            turns.push(vec![m.clone()]);
        } else {
            turns.last_mut().expect("just ensured non-empty").push(m.clone());
        }
    }
    turns
}

/// Builds the final windowed list: the goal, plus the recent tail from
/// `start_idx` onward, plus — when `max_content_chars` leaves slack after
/// that — as many earlier ("middle") turns as fit, chosen by lexical
/// overlap with the goal rather than being discarded outright (Phase
/// 51-2). Purely additive: can only add back messages the positional
/// window already decided to drop, never remove anything it decided to
/// keep. No-op (plain trim) when there's no size budget, no slack, no
/// middle turns, or none of them share any vocabulary with the goal.
fn trim_with_relevance_retention(
    messages: &[Message],
    start_idx: usize,
    max_content_chars: Option<u32>,
    summary_parts: &mut Vec<String>,
) -> Vec<Message> {
    let plain_trim = |messages: &[Message]| -> Vec<Message> {
        let mut trimmed = vec![messages[0].clone()];
        trimmed.extend(messages[start_idx..].iter().cloned());
        trimmed
    };

    let Some(budget) = max_content_chars.map(|b| usize::try_from(b).unwrap_or(usize::MAX)) else {
        return plain_trim(messages);
    };
    if start_idx <= 1 {
        return plain_trim(messages);
    }

    let kept_chars: usize = messages[0].content.len()
        + messages[start_idx..].iter().map(|m| m.content.len()).sum::<usize>();
    let mut remaining = budget.saturating_sub(kept_chars);

    let goal_words = tokenize_lowercase_words(&messages[0].content);
    if remaining == 0 || goal_words.is_empty() {
        return plain_trim(messages);
    }

    let middle_turns = split_into_turns(&messages[1..start_idx]);
    let mut scored: Vec<(usize, usize, Vec<Message>)> = middle_turns
        .into_iter()
        .enumerate()
        .map(|(order, turn)| {
            let score = relevance_score(&turn, &goal_words);
            (order, score, turn)
        })
        .collect();
    // Highest relevance first; ties broken toward the more recent turn.
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.cmp(&a.0)));

    let mut reinstated: Vec<(usize, Vec<Message>)> = Vec::new();
    for (order, score, turn) in scored {
        if score == 0 {
            continue;
        }
        let turn_chars: usize = turn.iter().map(|m| m.content.len()).sum();
        if turn_chars > remaining {
            continue;
        }
        remaining -= turn_chars;
        reinstated.push((order, turn));
    }

    if reinstated.is_empty() {
        return plain_trim(messages);
    }

    reinstated.sort_by_key(|(order, _)| *order);
    let reinstated_msg_count: usize = reinstated.iter().map(|(_, t)| t.len()).sum();
    let reinstated_turn_count = reinstated.len();
    summary_parts.push(format!(
        "Reinstated {reinstated_msg_count} message(s) from {reinstated_turn_count} earlier turn(s) via lexical relevance to the goal."
    ));

    let mut result = vec![messages[0].clone()];
    for (_, turn) in reinstated {
        result.extend(turn);
    }
    result.extend(messages[start_idx..].iter().cloned());
    result
}
