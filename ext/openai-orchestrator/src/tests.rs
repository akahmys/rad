use crate::orchestrator::{STATE, handle_event};
use crate::sse::process_sse_buffer;
use crate::types::OrchestratorState;
use rad_models::RasCoreEvent;
use std::collections::HashMap;

#[test]
fn test_sse_parsing() {
    let mut state = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\ndata: [DONE]\n"
            .to_string(),
        is_reasoning: false,
        reasoning_buffered: String::new(),
        tool_calls: HashMap::new(),
        pending_tool_calls: Vec::new(),
        expected_mcp_servers: Vec::new(),
        mcp_tools: Vec::new(),
        mcp_tool_providers: HashMap::new(),
        max_history_messages: None,
        max_tool_output_chars: None,
    };

    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream, "");
    assert_eq!(state.assistant, "hello");
}

#[test]
fn test_sse_parsing_tool_call() {
    let mut state = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"read\",\"arguments\":\"{\\\"path\\\":\\\"/tmp/foo\\\"}\"}}]}}]}\n\ndata: [DONE]\n".to_string(),
        is_reasoning: false,
        reasoning_buffered: String::new(),
        tool_calls: HashMap::new(),
        pending_tool_calls: Vec::new(),
        expected_mcp_servers: Vec::new(),
        mcp_tools: Vec::new(),
        mcp_tool_providers: HashMap::new(),
        max_history_messages: None,
        max_tool_output_chars: None,
    };

    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream, "");
}

#[test]
fn test_sse_parsing_reasoning() {
    // Test parsing of reasoning_content
    let mut state = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"thinking step 1\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"hello response\"}}]}\n\ndata: [DONE]\n".to_string(),
        is_reasoning: false,
        reasoning_buffered: String::new(),
        tool_calls: HashMap::new(),
        pending_tool_calls: Vec::new(),
        expected_mcp_servers: Vec::new(),
        mcp_tools: Vec::new(),
        mcp_tool_providers: HashMap::new(),
        max_history_messages: None,
        max_tool_output_chars: None,
    };

    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream, "");
    assert_eq!(state.reasoning_buffered, "thinking step 1");
    assert_eq!(state.assistant, "hello response");
    assert_eq!(state.is_reasoning, false);

    // Test parsing of content with inline <thought> tags
    let mut state_inline = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"content\":\"<thought>thinking inline\"}}]}\n\ndata: {\"choices\":[{\"delta\":{\"content\":\"</thought>hello inline response\"}}]}\n\ndata: [DONE]\n".to_string(),
        is_reasoning: false,
        reasoning_buffered: String::new(),
        tool_calls: HashMap::new(),
        pending_tool_calls: Vec::new(),
        expected_mcp_servers: Vec::new(),
        mcp_tools: Vec::new(),
        mcp_tool_providers: HashMap::new(),
        max_history_messages: None,
        max_tool_output_chars: None,
    };

    let res_inline = process_sse_buffer(&mut state_inline);
    assert!(res_inline.is_ok());
    assert_eq!(state_inline.stream, "");
    assert_eq!(state_inline.reasoning_buffered, "thinking inline");
    assert_eq!(state_inline.assistant, "hello inline response");
    assert_eq!(state_inline.is_reasoning, false);
}

#[test]
fn test_handle_event_human_input() {
    if let Ok(mut state_guard) = STATE.lock() {
        *state_guard = None;
    }

    let event = RasCoreEvent::HumanInputReceived {
        text: "test task".to_string(),
    };

    let res = handle_event(event);
    assert!(res.is_ok());

    let state_guard = STATE.lock().unwrap();
    let state = state_guard.as_ref().unwrap();
    assert!(state.stream.is_empty());
}
