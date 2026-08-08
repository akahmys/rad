//! Frame parsing, against the dialect table it is driven by.
//!
//! The extension had no test for any of this — `dialect/tests.rs` was its only
//! test file, so nothing checked that a `data:` line became the right event.

use super::{CompletionUsage, LlmEvent, Parser, ToolCallChunk};
use crate::dialect::OPENAI;

fn parse(chunks: &[&str]) -> Vec<LlmEvent> {
    let mut parser = Parser::default();
    let mut out = Vec::new();
    for chunk in chunks {
        parser.push(chunk, &OPENAI, &mut out);
    }
    out
}

#[test]
fn a_content_frame_becomes_a_content_chunk() {
    let events = parse(&["data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n"]);
    assert_eq!(events, vec![LlmEvent::ContentChunk("hi".to_string())]);
}

/// A frame split across reads must not produce a half-parsed event. The buffer
/// exists for exactly this, and SSE over a socket splits constantly.
#[test]
fn a_frame_split_across_chunks_is_parsed_once_whole() {
    let events = parse(&[
        "data: {\"choices\":[{\"delta\":",
        "{\"content\":\"split\"}}]}\n",
    ]);
    assert_eq!(events, vec![LlmEvent::ContentChunk("split".to_string())]);
}

/// `reasoning_content` wins over `content` in the same frame. The agent loop
/// renders the two differently, so a thought reported as an answer would print
/// into the user's transcript as if the model had said it.
#[test]
fn reasoning_takes_precedence_over_content() {
    let events = parse(&[
        "data: {\"choices\":[{\"delta\":{\"content\":\"answer\",\"reasoning_content\":\"thought\"}}]}\n",
    ]);
    assert_eq!(
        events,
        vec![LlmEvent::ReasoningChunk("thought".to_string())]
    );
}

#[test]
fn a_tool_call_frame_carries_index_id_name_and_arguments() {
    let events = parse(&[
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":2,\"id\":\"call_7\",\"function\":{\"name\":\"read_file\",\"arguments\":\"{\\\"p\\\":\"}}]}}]}\n",
    ]);
    assert_eq!(
        events,
        vec![LlmEvent::ToolCallChunk(ToolCallChunk {
            index: 2,
            id: Some("call_7".to_string()),
            name: Some("read_file".to_string()),
            arguments_chunk: "{\"p\":".to_string(),
        })]
    );
}

/// A continuation frame carries only the argument text — no id, no name. The
/// agent loop reassembles by index, so dropping the frame loses arguments.
#[test]
fn a_tool_call_continuation_keeps_its_index() {
    let events = parse(&[
        "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":2,\"function\":{\"arguments\":\"1}\"}}]}}]}\n",
    ]);
    assert_eq!(
        events,
        vec![LlmEvent::ToolCallChunk(ToolCallChunk {
            index: 2,
            id: None,
            name: None,
            arguments_chunk: "1}".to_string(),
        })]
    );
}

#[test]
fn usage_becomes_a_completion_event() {
    let events = parse(&["data: {\"usage\":{\"prompt_tokens\":11,\"completion_tokens\":4}}\n"]);
    assert_eq!(
        events,
        vec![LlmEvent::CompletionComplete(CompletionUsage {
            prompt_tokens: 11,
            completion_tokens: 4,
        })]
    );
}

/// Providers pad frames with a zeroed usage block. Reporting those would end
/// the turn's accounting on the first chunk.
#[test]
fn a_zeroed_usage_block_is_not_a_completion() {
    let events = parse(&[
        "data: {\"choices\":[{\"delta\":{\"content\":\"x\"}}],\"usage\":{\"prompt_tokens\":0,\"completion_tokens\":0}}\n",
    ]);
    assert_eq!(events, vec![LlmEvent::ContentChunk("x".to_string())]);
}

#[test]
fn done_sets_done_and_stops_parsing() {
    let mut parser = Parser::default();
    let mut out = Vec::new();
    parser.push(
        "data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\ndata: [DONE]\n",
        &OPENAI,
        &mut out,
    );
    assert!(parser.done);
    assert_eq!(out, vec![LlmEvent::ContentChunk("a".to_string())]);
}

#[test]
fn a_malformed_frame_is_skipped_rather_than_failing_the_stream() {
    let events = parse(&[
        "data: not json\n",
        "data: {\"choices\":[{\"delta\":{\"content\":\"after\"}}]}\n",
    ]);
    assert_eq!(events, vec![LlmEvent::ContentChunk("after".to_string())]);
}

/// The wire shape the agent loop deserializes
/// (`ext/rad-orchestrator/src/orchestrator/reasoning.rs`). A port must not
/// change it: the consumer is still an extension until stage 8.
#[test]
fn events_serialize_to_the_shape_the_agent_loop_reads() {
    let content = serde_json::to_string(&LlmEvent::ContentChunk("hi".to_string())).unwrap();
    assert_eq!(content, r#"{"ContentChunk":"hi"}"#);

    let tool = serde_json::to_string(&LlmEvent::ToolCallChunk(ToolCallChunk {
        index: 0,
        id: None,
        name: Some("f".to_string()),
        arguments_chunk: "{}".to_string(),
    }))
    .unwrap();
    assert_eq!(
        tool,
        r#"{"ToolCallChunk":{"index":0,"id":null,"name":"f","arguments_chunk":"{}"}}"#
    );

    let usage = serde_json::to_string(&LlmEvent::CompletionComplete(CompletionUsage {
        prompt_tokens: 1,
        completion_tokens: 2,
    }))
    .unwrap();
    assert_eq!(
        usage,
        r#"{"CompletionComplete":{"prompt_tokens":1,"completion_tokens":2}}"#
    );
}
