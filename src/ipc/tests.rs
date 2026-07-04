use std::io::Cursor;
use std::path::PathBuf;
use serde_json::json;
use crate::ipc::{IpcBridge, RasCoreEvent, RasRpcCommand, RasRpcRequest, RasRpcResponse};

#[test]
fn test_serialize_deserialize_request() {
    let raw_json = r#"{"id":"req_1","method":"CreateNode","params":{"parent_id":"","node_type":"root"}}"#;
    let req: RasRpcRequest = serde_json::from_str(raw_json).unwrap();

    assert_eq!(req.id.as_deref(), Some("req_1"));
    match &req.command {
        RasRpcCommand::CreateNode { parent_id, node_type } => {
            assert_eq!(parent_id, "");
            assert_eq!(node_type, "root");
        }
        _ => panic!("Expected CreateNode command"),
    }

    let serialized = serde_json::to_string(&req).unwrap();
    assert!(serialized.contains(r#""id":"req_1""#));
    assert!(serialized.contains(r#""method":"CreateNode""#));
}

#[test]
fn test_ipc_bridge_read() {
    let input_data = "{\"id\":\"1\",\"method\":\"FileRead\",\"params\":{\"path\":\"/tmp/test.txt\"}}\n";
    let reader = Cursor::new(input_data);
    let writer = Vec::new();

    let mut bridge = IpcBridge::new(reader, writer);
    let req = bridge.read_request().unwrap().unwrap();

    assert_eq!(req.id.as_deref(), Some("1"));
    match req.command {
        RasRpcCommand::FileRead { path } => {
            assert_eq!(path, PathBuf::from("/tmp/test.txt"));
        }
        _ => panic!("Expected FileRead command"),
    }
}

#[test]
fn test_ipc_bridge_write() {
    let reader = Cursor::new(Vec::new());
    let writer = Vec::new();

    let mut bridge = IpcBridge::new(reader, writer);

    let resp = RasRpcResponse {
        id: Some("1".to_string()),
        result: Ok(json!({"success": true})),
    };

    bridge.write_response(&resp).unwrap();

    let event = RasCoreEvent::TokenReceived {
        token: "hello".to_string(),
    };

    bridge.write_event(&event).unwrap();

    let output = String::from_utf8(bridge.writer).unwrap();
    let mut lines = output.lines();

    let resp_line = lines.next().unwrap();
    let decoded_resp: RasRpcResponse = serde_json::from_str(resp_line).unwrap();
    assert_eq!(decoded_resp.id.as_deref(), Some("1"));

    let event_line = lines.next().unwrap();
    let decoded_event: RasCoreEvent = serde_json::from_str(event_line).unwrap();
    match decoded_event {
        RasCoreEvent::TokenReceived { token } => {
            assert_eq!(token, "hello");
        }
        _ => panic!("Expected TokenReceived event"),
    }
}
