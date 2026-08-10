//! The DAG walk and the orphan filter, both pure.
//!
//! `system_prompt` is not tested here: it reads the process's working
//! directory, and these run in a shared one. `tests/agent_loop_tests.rs` drives
//! it across dispatch, where the module has a directory of its own.
use super::{
    Dag, DagNode, Message, ToolCall, ToolCallFunction, filter_orphaned_tool_messages, traverse,
};
use std::collections::HashMap;

fn dag(nodes: &[(&str, &[&str], &str, &str)], current: &str) -> Dag {
    let mut map = HashMap::new();
    for (id, parents, node_type, text) in nodes {
        map.insert(
            (*id).to_string(),
            DagNode {
                parent_ids: parents.iter().map(|p| (*p).to_string()).collect(),
                node_type: (*node_type).to_string(),
                text: (*text).to_string(),
            },
        );
    }
    Dag {
        nodes: map,
        current_node_id: Some(current.to_string()),
    }
}

fn assistant_with_call(id: &str) -> Message {
    Message {
        role: "assistant".to_string(),
        content: None,
        name: None,
        tool_call_id: None,
        tool_calls: Some(vec![ToolCall {
            id: id.to_string(),
            tool_type: "function".to_string(),
            function: ToolCallFunction {
                name: "write".to_string(),
                arguments: "{}".to_string(),
            },
        }]),
    }
}

fn tool_reply(call_id: &str) -> Message {
    Message {
        role: "tool".to_string(),
        content: Some("done".to_string()),
        name: None,
        tool_call_id: Some(call_id.to_string()),
        tool_calls: None,
    }
}

/// The walk runs child-to-root and reverses, so this is the property that
/// matters: what comes back is in the order the conversation happened.
#[test]
fn traversal_returns_messages_oldest_first() {
    let d = dag(
        &[
            ("n0", &[], "user", "first"),
            ("n1", &["n0"], "assistant", "second"),
            ("n2", &["n1"], "user", "third"),
        ],
        "n2",
    );
    let msgs = traverse(&d);
    let texts: Vec<_> = msgs.iter().map(|m| m.content.clone().unwrap()).collect();
    assert_eq!(texts, vec!["first", "second", "third"]);
}

/// Only the first parent is followed — a rollback leaves siblings behind, and
/// the active branch is the one being replayed.
#[test]
fn only_the_first_parent_is_followed() {
    let d = dag(
        &[
            ("root", &[], "user", "root"),
            ("other", &[], "user", "not on this branch"),
            ("tip", &["root", "other"], "assistant", "tip"),
        ],
        "tip",
    );
    let msgs = traverse(&d);
    assert_eq!(msgs.len(), 2);
    assert!(
        msgs.iter()
            .all(|m| m.content.as_deref() != Some("not on this branch"))
    );
}

#[test]
fn nodes_that_are_not_conversation_roles_are_skipped() {
    let d = dag(
        &[
            ("n0", &[], "user", "kept"),
            ("n1", &["n0"], "snapshot", "bookkeeping"),
            ("n2", &["n1"], "assistant", "also kept"),
        ],
        "n2",
    );
    let msgs = traverse(&d);
    assert_eq!(msgs.len(), 2);
}

/// A node written by the orchestrator holds a serialised `Message`; one written
/// by a human holds plain text. Both have to work.
#[test]
fn a_node_holding_serialised_json_is_parsed_as_a_message() {
    let json = serde_json::to_string(&assistant_with_call("call_1")).unwrap();
    let d = dag(&[("n0", &[], "assistant", &json)], "n0");
    let msgs = traverse(&d);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].tool_calls.as_ref().unwrap()[0].id, "call_1");
}

/// The DAG knows who spoke; a `role` inside the stored JSON does not override
/// it.
#[test]
fn the_node_type_wins_over_a_role_inside_the_json() {
    let d = dag(
        &[("n0", &[], "user", r#"{"role":"assistant","content":"hi"}"#)],
        "n0",
    );
    assert_eq!(traverse(&d)[0].role, "user");
}

#[test]
fn an_empty_node_produces_no_message() {
    let d = dag(
        &[
            ("n0", &[], "user", "kept"),
            ("n1", &["n0"], "assistant", ""),
        ],
        "n1",
    );
    let msgs = traverse(&d);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].content.as_deref(), Some("kept"));
}

/// A dangling parent ends the walk rather than panicking or looping.
#[test]
fn a_missing_parent_ends_the_walk() {
    let d = dag(&[("n1", &["gone"], "assistant", "tip")], "n1");
    assert_eq!(traverse(&d).len(), 1);
}

#[test]
fn a_tool_reply_with_a_matching_call_survives() {
    let msgs = vec![assistant_with_call("call_1"), tool_reply("call_1")];
    assert_eq!(filter_orphaned_tool_messages(msgs).len(), 2);
}

/// The whole point: an unanswered `tool` message makes the request invalid at
/// the backend, which is what AWU 78 was about.
#[test]
fn a_tool_reply_with_no_matching_call_is_dropped() {
    let msgs = vec![assistant_with_call("call_1"), tool_reply("call_OTHER")];
    let filtered = filter_orphaned_tool_messages(msgs);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].role, "assistant");
}

/// Order matters: the matching call has to come *earlier*. A reply preceding
/// its call is still an orphan as far as the backend is concerned.
#[test]
fn a_tool_reply_before_its_call_is_still_dropped() {
    let msgs = vec![tool_reply("call_1"), assistant_with_call("call_1")];
    let filtered = filter_orphaned_tool_messages(msgs);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].role, "assistant");
}

#[test]
fn a_tool_reply_with_no_call_id_at_all_is_dropped() {
    let mut orphan = tool_reply("call_1");
    orphan.tool_call_id = None;
    let filtered = filter_orphaned_tool_messages(vec![assistant_with_call("call_1"), orphan]);
    assert_eq!(filtered.len(), 1);
}

#[test]
fn non_tool_messages_pass_through_untouched() {
    let msgs = vec![
        Message {
            role: "system".to_string(),
            content: Some("rules".to_string()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
        Message {
            role: "user".to_string(),
            content: Some("hello".to_string()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
        },
    ];
    assert_eq!(filter_orphaned_tool_messages(msgs).len(), 2);
}
