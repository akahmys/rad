use std::sync::{Arc, Mutex};
use rad::config::{Config, CoreConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use rad::command::{CommandParser, Command, CommandManager, CommandResult};
use tempfile::tempdir;

#[test]
fn test_slash_command_parsing() {
    assert_eq!(CommandParser::parse("/help"), Some(Command::Help));
    assert_eq!(CommandParser::parse("/exit"), Some(Command::Exit));
    assert_eq!(CommandParser::parse("/status"), Some(Command::Status));
    assert_eq!(CommandParser::parse("/clear"), Some(Command::Clear));
    assert_eq!(
        CommandParser::parse("/session 1234"),
        Some(Command::Session("1234".to_string()))
    );
    assert_eq!(
        CommandParser::parse("/rollback node_0"),
        Some(Command::Rollback("node_0".to_string()))
    );
    assert_eq!(CommandParser::parse("regular text"), None);
}

#[test]
fn test_command_execution() {
    let tmp = tempdir().unwrap();
    let workspace = tmp.path().to_path_buf();
    let snapshot = tmp.path().join("snapshots");
    let log = tmp.path().join("logs");

    std::fs::create_dir_all(&snapshot).unwrap();
    std::fs::create_dir_all(&log).unwrap();

    let config = Config {
        core: CoreConfig {
            workspace: workspace.to_string_lossy().to_string(),
            snapshot: snapshot.to_string_lossy().to_string(),
            log: log.to_string_lossy().to_string(),
        },
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let orchestrator = Orchestrator::new(config, "test_session".to_string(), dag.clone());

    // 1. Test Status Command on empty DAG
    let res = CommandManager::execute(Command::Status, &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Session ID: test_session"));
        assert!(info.contains("Total DAG Nodes: 0"));
        assert!(info.contains("Current DAG Node: None"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2. Add nodes and test Status Command again
    {
        let mut dag_guard = dag.lock().unwrap();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Hello").unwrap();
        let _n1 = dag_guard.create_node(&n0, "assistant").unwrap();
    }

    let res = CommandManager::execute(Command::Status, &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Session ID: test_session"));
        assert!(info.contains("Total DAG Nodes: 2"));
        assert!(info.contains("Current DAG Node: node_1"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 3. Rollback to node_0 (which exists)
    let snapshot_node_path = snapshot.join("node_0");
    std::fs::create_dir_all(&snapshot_node_path).unwrap();

    let res = CommandManager::execute(Command::Rollback("node_0".to_string()), &orchestrator);
    match res {
        CommandResult::Continue => {
            let dag_guard = dag.lock().unwrap();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }

    // 4. Rollback to non-existent node
    let res = CommandManager::execute(Command::Rollback("non_existent".to_string()), &orchestrator);
    match res {
        CommandResult::Continue => {
            // current node should still be node_0
            let dag_guard = dag.lock().unwrap();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }
}
