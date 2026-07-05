use super::*;

#[test]
fn test_sse_parsing() {
    let mut state = OrchestratorState {
        assistant_buffer: String::new(),
        stream_buffer: "data: {\"choices\":[{\"delta\":{\"content\":\"hello\"}}]}\n\ndata: [DONE]\n".to_string(),
    };
    
    let res = process_sse_buffer(&mut state);
    assert!(res.is_ok());
    assert_eq!(state.stream_buffer, "");
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
    assert!(state.stream_buffer.is_empty());
}
