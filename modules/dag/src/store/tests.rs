//! Persistence, which is the only part of this module that is new. The graph's
//! own behaviour is covered by `graph/tests.rs`, moved across unchanged from
//! `src/dag/tests.rs`.
use super::{mutate, open, read};

/// Each test gets its own workspace, since `STATE` is a thread-local the test
/// harness reuses across tests on the same thread.
fn workspace() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

#[test]
fn a_mutation_reaches_disk_without_being_asked_to() {
    let ws = workspace();
    let root = ws.path().to_string_lossy().to_string();
    open(&root, "s1").unwrap();
    mutate(|dag| dag.create_node("", "user")).unwrap();

    let file = ws.path().join(".rad/sessions/s1.json");
    assert!(file.exists(), "the graph never reached {}", file.display());
    let on_disk: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(on_disk["nodes"]["node_0"]["node_type"], "user");
}

/// Reopening is what a restart does, and what §3.6.6's reload after a trap
/// would do. Whatever was saved has to come back.
#[test]
fn reopening_a_session_restores_what_was_saved() {
    let ws = workspace();
    let root = ws.path().to_string_lossy().to_string();

    open(&root, "s2").unwrap();
    let id = mutate(|dag| dag.create_node("", "user")).unwrap();
    mutate(|dag| dag.set_node_text(&id, "remember me")).unwrap();

    // Attaching elsewhere and back again is the only way to prove the value
    // came off disk rather than out of the cell it was already in.
    open(&root, "other").unwrap();
    assert_eq!(read(|dag| dag.nodes.len()).unwrap(), 0);

    open(&root, "s2").unwrap();
    assert_eq!(
        read(|dag| dag.nodes[&id].text.clone()).unwrap(),
        "remember me"
    );
}

/// A session that has never been written is an empty graph, not a failure —
/// that is what starting a new one looks like.
#[test]
fn opening_a_session_that_does_not_exist_yet_is_empty_rather_than_an_error() {
    let ws = workspace();
    open(&ws.path().to_string_lossy(), "brand_new").unwrap();
    assert_eq!(read(|dag| dag.nodes.len()).unwrap(), 0);
}

/// A corrupt file is reported rather than silently replaced with an empty
/// graph. Silently starting over would look like the conversation vanished.
#[test]
fn a_corrupt_session_file_is_reported_not_discarded() {
    let ws = workspace();
    let dir = ws.path().join(".rad/sessions");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("bad.json"), "{not json").unwrap();

    let err = open(&ws.path().to_string_lossy(), "bad").unwrap_err();
    assert!(err.contains("unreadable"), "{err}");
}

/// Every method goes through `mutate`/`read`, so this is the error a caller
/// gets if the host ever forgets to open a session first. Worth pinning: the
/// wording is what tells someone it is a wiring mistake and not a missing node.
#[test]
fn using_the_graph_before_opening_a_session_says_so() {
    // Detach first — an earlier test on this thread may have left one open.
    super::STATE.with_borrow_mut(|slot| *slot = None);
    let err = mutate(|dag| dag.create_node("", "user")).unwrap_err();
    assert!(err.contains("dag.open"), "{err}");
}

/// The on-disk shape is `src/session.rs`'s, so a session written by either side
/// is readable by the other — which is what lets AWU 986 route the host through
/// this module with no migration.
#[test]
fn the_file_is_where_and_what_session_rs_writes() {
    let ws = workspace();
    let root = ws.path().to_string_lossy().to_string();
    open(&root, "compat").unwrap();
    mutate(|dag| dag.create_node("", "user")).unwrap();

    let json = std::fs::read_to_string(ws.path().join(".rad/sessions/compat.json")).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for key in ["nodes", "current_node_id", "next_node_index"] {
        assert!(parsed.get(key).is_some(), "'{key}' missing from {json}");
    }
}
