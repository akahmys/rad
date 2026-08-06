use parking_lot::Mutex;
use rad::command::{CommandManager, CommandParser, CommandResult, ParsedCommand};
use rad::config::{Config, CoreConfig};
use rad::dag::Dag;
use rad::orchestrator::Orchestrator;
use std::sync::Arc;
use tempfile::tempdir;

fn parsed(name: &'static str, args: &str) -> ParsedCommand {
    ParsedCommand {
        name,
        args: args.to_string(),
    }
}

#[test]
fn test_slash_command_parsing() {
    assert_eq!(CommandParser::parse("/help"), Some(parsed("help", "")));
    assert_eq!(CommandParser::parse("/quit"), Some(parsed("quit", "")));
    assert_eq!(
        CommandParser::parse("/session 1234"),
        Some(parsed("session", "1234"))
    );
    assert_eq!(
        CommandParser::parse("/rollback node_0"),
        Some(parsed("rollback", "node_0"))
    );
    assert_eq!(CommandParser::parse("/tree"), Some(parsed("tree", "")));
    assert_eq!(CommandParser::parse("/tools"), Some(parsed("tools", "")));
    assert_eq!(CommandParser::parse("/new"), Some(parsed("new", "")));
    assert_eq!(
        CommandParser::parse("/compact"),
        Some(parsed("compact", ""))
    );
    // `/status` and `/clear` no longer exist: `/session` absorbed
    // `/status`'s output, and `/clear` was removed (no external precedent
    // for it — pi-coding-agent doesn't have one either — and terminals
    // already provide Ctrl+L).
    assert_eq!(CommandParser::parse("/status"), None);
    assert_eq!(CommandParser::parse("/clear"), None);
    assert!(matches!(
        CommandParser::parse("/llm"),
        Some(ParsedCommand { name: "llm", .. })
    ));
    // `/models` is an alias for `/llm` — resolves to the same canonical name.
    assert!(matches!(
        CommandParser::parse("/models"),
        Some(ParsedCommand { name: "llm", .. })
    ));
    // A `/`-prefixed but unrecognized command falls through to `None` (sent
    // to the LLM as a task) rather than being rejected.
    assert_eq!(CommandParser::parse("/whatever-this-is"), None);
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
            ..Default::default()
        },
        ..Default::default()
    };

    let dag = Arc::new(Mutex::new(Dag::new()));
    let orchestrator = Arc::new(Orchestrator::new(
        config,
        "test_session".to_string(),
        dag.clone(),
        None,
    ));

    // 1. Test Session Command (merged /status output) on empty DAG
    let res = CommandManager::execute(&parsed("session", ""), &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Session ID: test_session"));
        assert!(info.contains("Total DAG Nodes: 0"));
        assert!(info.contains("Current DAG Node: None"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2. Add nodes and test Session Command again
    {
        let mut dag_guard = dag.lock();
        let n0 = dag_guard.create_node("", "user").unwrap();
        dag_guard.set_node_text(&n0, "Hello").unwrap();
        let _n1 = dag_guard.create_node(&n0, "assistant").unwrap();
    }

    let res = CommandManager::execute(&parsed("session", ""), &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Session ID: test_session"));
        assert!(info.contains("Total DAG Nodes: 2"));
        assert!(info.contains("Current DAG Node: node_1"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2.5 Test Tree Command
    let res = CommandManager::execute(&parsed("tree", ""), &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("node_0"));
        assert!(info.contains("node_1"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2.6 Test Tools Command
    let res = CommandManager::execute(&parsed("tools", ""), &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Active Permissions:"));
        assert!(info.contains("Available Tools"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 2.8 Rollback with no node ID shows a usage error instead of silently
    // no-op'ing or being sent to the LLM as a task.
    let res = CommandManager::execute(&parsed("rollback", ""), &orchestrator);
    if let CommandResult::StatusInfo(info) = res {
        assert!(info.contains("Usage: /rollback"));
    } else {
        panic!("Expected StatusInfo");
    }

    // 3. Rollback to node_0 (which exists)
    let snapshot_node_path = snapshot.join("node_0");
    std::fs::create_dir_all(&snapshot_node_path).unwrap();

    let res = CommandManager::execute(&parsed("rollback", "node_0"), &orchestrator);
    match res {
        CommandResult::Continue => {
            let dag_guard = dag.lock();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }

    // 4. Rollback to non-existent node
    let res = CommandManager::execute(&parsed("rollback", "non_existent"), &orchestrator);
    match res {
        CommandResult::Continue => {
            // current node should still be node_0
            let dag_guard = dag.lock();
            assert_eq!(dag_guard.current_node_id.as_deref(), Some("node_0"));
        }
        _ => panic!("Expected CommandResult::Continue"),
    }

    // 5. Test Reload Command
    let res = CommandManager::execute(&parsed("reload", ""), &orchestrator);
    match res {
        CommandResult::StatusInfo(info) => {
            assert!(info.contains("Failed to reload") || info.contains("reloaded successfully"));
        }
        _ => panic!("Expected CommandResult::StatusInfo"),
    }

    // 6. Test New Command (formerly /reset)
    let res = CommandManager::execute(&parsed("new", ""), &orchestrator);
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
    // Previously missing entirely from the hand-maintained completion list.
    assert!(res.1.contains(&"/llm".to_string()));
    assert!(res.1.contains(&"/models".to_string()));

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
