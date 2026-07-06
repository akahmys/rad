use super::*;

#[test]
fn test_orchestrator_creation() {
    let config = Config::default();
    let dag = Arc::new(Mutex::new(Dag::new()));
    let orch = Orchestrator::new(config, "test_session".to_string(), dag, None);
    assert_eq!(*orch.session_id.lock().unwrap(), "test_session");
}
