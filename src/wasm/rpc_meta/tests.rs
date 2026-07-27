use super::active_llm_profile_json;
use crate::config::{Config, LlmEndpointProfile};
use crate::dag::Dag;
use crate::orchestrator::Orchestrator;
use parking_lot::Mutex;
use std::sync::Arc;

fn orchestrator_with_profile(model: Option<&str>, context_length: Option<u32>) -> Orchestrator {
    let mut config = Config::default();
    config.llm.active = Some("local".to_string());
    config.llm.endpoints.insert(
        "local".to_string(),
        LlmEndpointProfile {
            base_url: "http://localhost:8080/v1".to_string(),
            api_key: None,
            model: model.map(ToString::to_string),
            context_length,
        },
    );
    Orchestrator::new(config, "test".to_string(), Arc::new(Mutex::new(Dag::new())), None)
}

#[test]
fn test_active_llm_profile_json_reports_known_values() {
    let orch = orchestrator_with_profile(Some("qwen"), Some(8192));
    let val = active_llm_profile_json(&orch);
    assert_eq!(val["model"], serde_json::json!("qwen"));
    assert_eq!(val["context_length"], serde_json::json!(8192));
}

#[test]
fn test_active_llm_profile_json_nulls_when_unknown() {
    let orch = orchestrator_with_profile(None, None);
    let val = active_llm_profile_json(&orch);
    assert!(val["model"].is_null());
    assert!(val["context_length"].is_null());
}

#[test]
fn test_active_llm_profile_json_nulls_when_no_active_profile() {
    let orch = Orchestrator::new(
        Config::default(),
        "test".to_string(),
        Arc::new(Mutex::new(Dag::new())),
        None,
    );
    let val = active_llm_profile_json(&orch);
    assert!(val["model"].is_null());
    assert!(val["context_length"].is_null());
}
