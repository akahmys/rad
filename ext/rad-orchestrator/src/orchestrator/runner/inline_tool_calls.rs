// Fallback parser for plain-text tool calls, split out of `runner.rs` to
// stay under the 300-line file limit. Handles two textual conventions
// models fall back to when the endpoint doesn't populate the structured
// `delta.tool_calls` SSE field:
//   1. `<|tool_call>call:rad:execute_command{...}<tool_call|>`
//   2. a bare OpenAI-style object echoed as plain content, e.g.
//      `{"name": "smart_search", "arguments": {"query": "..."}}`
use crate::tool::{ToolCall, ToolCallFunction};
use crate::types::PendingToolCall;

pub(crate) fn parse_inline_tool_calls(
    text: &str,
    assistant_tool_calls: &mut Vec<ToolCall>,
    pending_calls: &mut Vec<PendingToolCall>,
) {
    let mut search_str = text;
    let mut call_count = 0;
    while let Some(start_pos) = search_str.find("call:") {
        let after_call = &search_str[start_pos + 5..];
        if let Some(brace_pos) = after_call.find('{') {
            let raw_name = &after_call[..brace_pos];
            let name = raw_name
                .trim_start_matches("rad:")
                .trim_start_matches("default:")
                .trim();

            if let Some(end_idx) = find_balanced_brace_end(after_call, brace_pos) {
                let json_slice = &after_call[brace_pos..end_idx];
                let norm_args = normalize_args_str(json_slice);
                push_tool_call(name, norm_args, &mut call_count, assistant_tool_calls, pending_calls);
                search_str = &after_call[end_idx..];
                continue;
            }
        }
        search_str = &search_str[start_pos + 5..];
    }

    if call_count == 0 {
        parse_bare_json_tool_calls(text, &mut call_count, assistant_tool_calls, pending_calls);
    }
}

/// Finds the index just past the closing `}` that balances the `{` at
/// `haystack[open_pos]`, or `None` if the braces never balance. Does not
/// account for braces inside quoted strings, matching the pre-existing
/// (imperfect but adequate) behavior of the `call:` parser.
fn find_balanced_brace_end(haystack: &str, open_pos: usize) -> Option<usize> {
    let mut brace_count = 0;
    for (idx, ch) in haystack[open_pos..].char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    return Some(open_pos + idx + 1);
                }
            }
            _ => {}
        }
    }
    None
}

/// Re-serializes `json_slice` through `serde_json::Value` to normalize it;
/// falls back to the raw slice if it doesn't parse as JSON.
fn normalize_args_str(json_slice: &str) -> String {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_slice) {
        val.to_string()
    } else {
        json_slice.to_string()
    }
}

fn push_tool_call(
    name: &str,
    arguments: String,
    call_count: &mut usize,
    assistant_tool_calls: &mut Vec<ToolCall>,
    pending_calls: &mut Vec<PendingToolCall>,
) {
    let call_id = format!("inline_call_{call_count}");
    *call_count += 1;

    assistant_tool_calls.push(ToolCall {
        id: call_id.clone(),
        tool_type: "function".to_string(),
        function: ToolCallFunction {
            name: name.to_string(),
            arguments: arguments.clone(),
        },
    });

    pending_calls.push(PendingToolCall {
        id: call_id,
        name: name.to_string(),
        arguments,
        result: None,
    });
}

/// Fallback for models that emit the `OpenAI` function-call *shape* as plain
/// text content instead of populating `delta.tool_calls`, e.g.
/// `{"name": "smart_search", "arguments": {"query": "..."}}`. Only matches
/// objects with a string `name` field and an `arguments` field (object or
/// string), to avoid treating unrelated JSON-looking text as a tool call.
fn parse_bare_json_tool_calls(
    text: &str,
    call_count: &mut usize,
    assistant_tool_calls: &mut Vec<ToolCall>,
    pending_calls: &mut Vec<PendingToolCall>,
) {
    let mut search_from = 0;
    while let Some(rel_open) = text[search_from..].find('{') {
        let open_pos = search_from + rel_open;
        let Some(end_idx) = find_balanced_brace_end(text, open_pos) else {
            break;
        };
        let json_slice = &text[open_pos..end_idx];
        if let Ok(serde_json::Value::Object(obj)) =
            serde_json::from_str::<serde_json::Value>(json_slice)
        {
            let name = obj.get("name").and_then(serde_json::Value::as_str);
            if let (Some(name), Some(arguments)) = (name, obj.get("arguments"))
                && !name.is_empty()
            {
                let norm_args = match arguments {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                push_tool_call(name, norm_args, call_count, assistant_tool_calls, pending_calls);
            }
        }
        search_from = end_idx;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_legacy_call_colon_format() {
        let mut tool_calls = Vec::new();
        let mut pending = Vec::new();
        parse_inline_tool_calls(
            "<|tool_call>call:rad:execute_command{\"command\": \"ls\"}<tool_call|>",
            &mut tool_calls,
            &mut pending,
        );
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "execute_command");
        assert_eq!(tool_calls[0].function.arguments, "{\"command\":\"ls\"}");
    }

    #[test]
    fn parses_bare_openai_shaped_json() {
        let mut tool_calls = Vec::new();
        let mut pending = Vec::new();
        parse_inline_tool_calls(
            r#"{"name": "smart_search", "arguments": {"max_pages": 1, "query": "横浜 今日の天気"}}"#,
            &mut tool_calls,
            &mut pending,
        );
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "smart_search");
        assert_eq!(pending.len(), 1);
        let parsed_args: serde_json::Value =
            serde_json::from_str(&tool_calls[0].function.arguments).unwrap();
        assert_eq!(parsed_args["query"], "横浜 今日の天気");
        assert_eq!(parsed_args["max_pages"], 1);
    }

    #[test]
    fn parses_bare_json_with_stringified_arguments() {
        let mut tool_calls = Vec::new();
        let mut pending = Vec::new();
        parse_inline_tool_calls(
            r#"{"name": "smart_search", "arguments": "{\"query\": \"test\"}"}"#,
            &mut tool_calls,
            &mut pending,
        );
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.arguments, "{\"query\": \"test\"}");
    }

    #[test]
    fn ignores_unrelated_json_without_name_and_arguments() {
        let mut tool_calls = Vec::new();
        let mut pending = Vec::new();
        parse_inline_tool_calls(
            r#"Sure, here's the config: {"host": "localhost", "port": 8080}"#,
            &mut tool_calls,
            &mut pending,
        );
        assert!(tool_calls.is_empty());
        assert!(pending.is_empty());
    }

    #[test]
    fn plain_text_without_json_yields_no_calls() {
        let mut tool_calls = Vec::new();
        let mut pending = Vec::new();
        parse_inline_tool_calls("横浜は今日晴れです。", &mut tool_calls, &mut pending);
        assert!(tool_calls.is_empty());
    }
}
