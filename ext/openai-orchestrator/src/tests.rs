use super::*;
use crate::orchestrator::{OrchestratorState, process_sse_buffer, handle_event, STATE};
use std::collections::HashMap;

#[test]
fn test_sse_parsing() {
    let mut state = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\ndata: [DONE]\n".to_string(),
        tool_calls: HashMap::new(),
    };
    
    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream, "");
}

#[test]
fn test_sse_parsing_tool_call() {
    let mut state = OrchestratorState {
        assistant: String::new(),
        stream: "data: {\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"id\":\"call_1\",\"type\":\"function\",\"function\":{\"name\":\"file_read\",\"arguments\":\"{\\\"path\\\":\\\"/tmp/foo\\\"}\"}}]}}]}\n\ndata: [DONE]\n".to_string(),
        tool_calls: HashMap::new(),
    };
    
    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream, "");
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
