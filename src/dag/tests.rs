use super::Dag;

#[test]
fn test_create_and_text() {
    let mut dag = Dag::new();
    let root_id = dag.create_node("", "root").unwrap();
    assert_eq!(root_id, "node_0");
    assert_eq!(dag.current_node_id.as_deref(), Some("node_0"));

    let child_id = dag.create_node(&root_id, "child").unwrap();
    assert_eq!(child_id, "node_1");
    assert_eq!(dag.current_node_id.as_deref(), Some("node_1"));

    let child_node = dag.nodes.get(&child_id).unwrap();
    assert_eq!(child_node.parent_ids, vec![root_id.clone()]);

    dag.set_node_text(&child_id, "hello world").unwrap();
    assert_eq!(dag.nodes.get(&child_id).unwrap().text, "hello world");
}

#[test]
fn test_invalid_parent() {
    let mut dag = Dag::new();
    let res = dag.create_node("invalid_id", "node");
    assert!(res.is_err());
}

#[test]
fn test_merge_nodes() {
    let mut dag = Dag::new();
    let root = dag.create_node("", "root").unwrap();
    let child1 = dag.create_node(&root, "child1").unwrap();
    let child2 = dag.create_node(&root, "child2").unwrap();

    let grand_child = dag.create_node(&child1, "grand_child").unwrap();
    // Manually add child2 as a parent to grand_child to test multi-parent merge
    dag.nodes.get_mut(&grand_child).unwrap().parent_ids.push(child2.clone());

    let merge_id = dag.merge_nodes(&[child1.clone(), child2.clone()], "merged summary").unwrap();
    assert_eq!(merge_id, "node_4"); // node_0: root, node_1: child1, node_2: child2, node_3: grand_child, node_4: merge

    assert!(!dag.nodes.contains_key(&child1));
    assert!(!dag.nodes.contains_key(&child2));

    let merge_node = dag.nodes.get(&merge_id).unwrap();
    assert_eq!(merge_node.parent_ids, vec![root.clone()]);
    assert_eq!(merge_node.text, "merged summary");

    let gc_node = dag.nodes.get(&grand_child).unwrap();
    assert_eq!(gc_node.parent_ids, vec![merge_id.clone()]);
}

#[test]
fn test_delete_node() {
    let mut dag = Dag::new();
    let root = dag.create_node("", "root").unwrap();
    let child = dag.create_node(&root, "child").unwrap();

    dag.delete_node(&root).unwrap();
    assert!(!dag.nodes.contains_key(&root));
    assert!(dag.nodes.contains_key(&child));

    let child_node = dag.nodes.get(&child).unwrap();
    assert!(child_node.parent_ids.is_empty());
}
