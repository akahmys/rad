use crate::error::{UnifiedError, ErrorLevel};

#[test]
fn test_unified_error_serialization() {
    let err = UnifiedError::l1("file not found", "Fs");
    assert_eq!(err.error_level(), ErrorLevel::L1);

    let json_str = err.to_json_string();
    assert!(json_str.contains("\"level\":\"L1\""));
    assert!(json_str.contains("\"message\":\"file not found\""));
    assert!(json_str.contains("\"category\":\"Fs\""));

    let deserialized = UnifiedError::from_json_string(&json_str).unwrap();
    match deserialized {
        UnifiedError::L1 { message, category } => {
            assert_eq!(message, "file not found");
            assert_eq!(category, "Fs");
        }
        _ => panic!("Expected L1 error"),
    }
}

#[test]
fn test_unified_error_l2() {
    let err = UnifiedError::l2("invalid syntax", "Parser");
    assert_eq!(err.error_level(), ErrorLevel::L2);

    let json_str = err.to_json_string();
    let deserialized = UnifiedError::from_json_string(&json_str).unwrap();
    match deserialized {
        UnifiedError::L2 { message, rollback_target } => {
            assert_eq!(message, "invalid syntax");
            assert_eq!(rollback_target, "Parser");
        }
        _ => panic!("Expected L2 error"),
    }
}

#[test]
fn test_unified_error_l3() {
    let err = UnifiedError::l3("token budget exceeded", Some(100), Some(80));
    assert_eq!(err.error_level(), ErrorLevel::L3);

    let json_str = err.to_json_string();
    let deserialized = UnifiedError::from_json_string(&json_str).unwrap();
    match deserialized {
        UnifiedError::L3 { message, prompt_tokens, limit } => {
            assert_eq!(message, "token budget exceeded");
            assert_eq!(prompt_tokens, Some(100));
            assert_eq!(limit, Some(80));
        }
        _ => panic!("Expected L3 error"),
    }
}
