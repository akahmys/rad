use parking_lot::Mutex;
use rad::command::{Command, CommandManager, CommandParser, CommandResult};
use rad::config::{Config, CoreConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use std::sync::Arc;
use tempfile::tempdir;

#[test]
fn test_slash_command_parsing() {
    assert_eq!(CommandParser::parse("/help"), Some(Command::Help));
    assert_eq!(CommandParser::parse("/quit"), Some(Command::Quit));
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
    assert_eq!(CommandParser::parse("/tree"), Some(Command::Tree));
    assert_eq!(CommandParser::parse("/tools"), Some(Command::Tools));
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
            hitl_enabled: false,
            verification_command: None,
        },
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let orchestrator = Orchestrator::new(config, "test_session".to_string(), dag.clone(), None);

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
        let mut dag_guard = dag.lock();
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

    // 2.5 Test Tree Command
    let res = CommandManager::execute(Command::Tree, &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("node_0"));
        assert!(info.contains("node_1"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2.6 Test Tools Command
    let res = CommandManager::execute(Command::Tools, &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Active Permissions:"));
        assert!(info.contains("Available Tools (from Wasm tool-provider):"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 3. Rollback to node_0 (which exists)
    let snapshot_node_path = snapshot.join("node_0");
    std::fs::create_dir_all(&snapshot_node_path).unwrap();

    let res = CommandManager::execute(Command::Rollback("node_0".to_string()), &orchestrator);
    match res {
        CommandResult::Continue => {
            let dag_guard = dag.lock();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }

    // 4. Rollback to non-existent node
    let res = CommandManager::execute(Command::Rollback("non_existent".to_string()), &orchestrator);
    match res {
        CommandResult::Continue => {
            // current node should still be node_0
            let dag_guard = dag.lock();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }

    // 5. Test Reload Command
    let res = CommandManager::execute(Command::Reload, &orchestrator);
    match res {
        CommandResult::StatusInfo(info) => {
            assert!(info.contains("Failed to reload") || info.contains("reloaded successfully"));
        }
        _ => panic!("Expected CommandResult::StatusInfo"),
    }

    // 6. Test Reset Command
    let res = CommandManager::execute(Command::Reset, &orchestrator);
    match res {
        CommandResult::StatusInfo(info) => {
            assert!(info.contains("Session reset successfully"));
            // Verify session ID has changed from "test_session"
            let final_id = orchestrator.session_id.lock().clone();
            assert_ne!(final_id, "test_session");
            // Verify DAG was cleared
            let dag_guard = orchestrator.dag.lock();
            assert_eq!(dag_guard.nodes.len(), 0);
        }
        _ => panic!("Expected CommandResult::StatusInfo"),
    }
}

#[test]
fn test_command_completion() {
    use rustyline::completion::Completer;
    let helper = rad::command::CommandHelper::new();
    let history = rustyline::history::MemHistory::new();
    let ctx = rustyline::Context::new(&history);

    // 1. "/" input
    let res = helper.complete("/", 1, &ctx).unwrap();
    assert_eq!(res.0, 0);
    assert!(res.1.contains(&"/help".to_string()));
    assert!(res.1.contains(&"/quit".to_string()));
    assert!(res.1.contains(&"/tree".to_string()));
    assert!(res.1.contains(&"/tools".to_string()));

    // 2. "/he" input
    let res = helper.complete("/he", 3, &ctx).unwrap();
    assert_eq!(res.0, 0);
    assert_eq!(res.1, vec!["/help".to_string()]);

    // 3. Non-slash input (fallback to file completion)
    let tmp = tempdir().unwrap();
    let file_path = tmp.path().join("test_file.rs");
    std::fs::File::create(&file_path).unwrap();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(tmp.path()).unwrap();

    let res = helper.complete("test_", 5, &ctx);
    std::env::set_current_dir(original_dir).unwrap();

    let (pos_out, candidates) = res.unwrap();
    assert_eq!(pos_out, 0);
    assert!(candidates.contains(&"test_file.rs".to_string()));
}
