// Fallback parser for plain-text tool calls like
// `<|tool_call>call:rad:execute_command{...}<tool_call|>`, split out of
// `runner.rs` to stay under the 300-line file limit.
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

            let mut brace_count = 0;
            let mut json_end = None;
            for (idx, ch) in after_call[brace_pos..].char_indices() {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        json_end = Some(brace_pos + idx + 1);
                        break;
                    }
                }
            }

            if let Some(end_idx) = json_end {
                let json_slice = &after_call[brace_pos..end_idx];
                let call_id = format!("inline_call_{call_count}");
                call_count += 1;

                let norm_name = name;

                let norm_args = if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_slice) {
                    val.to_string()
                } else {
                    json_slice.to_string()
                };

                assistant_tool_calls.push(ToolCall {
                    id: call_id.clone(),
                    tool_type: "function".to_string(),
                    function: ToolCallFunction {
                        name: norm_name.to_string(),
                        arguments: norm_args.clone(),
                    },
                });

                pending_calls.push(PendingToolCall {
                    id: call_id,
                    name: norm_name.to_string(),
                    arguments: norm_args,
                    result: None,
                });

                search_str = &after_call[end_idx..];
                continue;
            }
        }
        search_str = &search_str[start_pos + 5..];
    }
}
