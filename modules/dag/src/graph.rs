//! The conversation graph.
//!
//! Copied from `models/src/dag.rs` with no change to a single operation. The
//! graph is the one piece of state a session cannot lose, so this port is
//! deliberately mechanical: `create_node`, `set_node_text`, `merge_nodes` and
//! `delete_node` behave exactly as they did, including the two conditional
//! `current_node_id` rules that took a bug each to find.
//!
//! What is new is where it lives and who saves it — see `store.rs`.
//!
//! **One method did not come across**: `set_node_semantic_references`. It has
//! no production caller in the host either — `tests/repo_map_tests.rs` is the
//! only thing that calls it, so the host's copy is kept alive by a test alone.
//! The `semantic_references` *field* stays, because it is in the on-disk shape
//! and sessions written before now may carry one.
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagNode {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub node_type: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_references: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dag {
    pub nodes: HashMap<String, DagNode>,
    pub current_node_id: Option<String>,
    pub next_node_index: usize,
}

impl Dag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_node(&mut self, parent_id: &str, node_type: &str) -> Result<String, String> {
        let mut parent_ids = Vec::new();
        if !parent_id.is_empty() {
            if !self.nodes.contains_key(parent_id) {
                return Err(format!("Parent node '{parent_id}' not found"));
            }
            parent_ids.push(parent_id.to_string());
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let node = DagNode {
            id: new_id.clone(),
            parent_ids,
            node_type: node_type.to_string(),
            text: String::new(),
            semantic_references: None,
        };

        self.nodes.insert(new_id.clone(), node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    pub fn set_node_text(&mut self, node_id: &str, text: &str) -> Result<(), String> {
        let node = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| format!("Node '{node_id}' not found"))?;
        node.text = text.to_string();
        Ok(())
    }

    pub fn merge_nodes(
        &mut self,
        node_ids: &[String],
        summary_text: &str,
    ) -> Result<String, String> {
        if node_ids.is_empty() {
            return Err("Cannot merge empty list of nodes".to_string());
        }

        let mut collected_parents = HashSet::new();
        let target_set: HashSet<&String> = node_ids.iter().collect();
        // Only follow `current_node_id` onto the new merge node if it
        // actually pointed at one of the merged nodes — mirrors
        // `delete_node`'s existing conditional-clear. Without this,
        // merging a historical (non-tip) range would silently rewind the
        // active conversation pointer to the merge node.
        let current_is_merged = self
            .current_node_id
            .as_deref()
            .is_some_and(|cur| node_ids.iter().any(|id| id == cur));

        for id in node_ids {
            let node = self
                .nodes
                .get(id)
                .ok_or_else(|| format!("Node '{id}' not found"))?;
            for parent in &node.parent_ids {
                if !target_set.contains(parent) {
                    collected_parents.insert(parent.clone());
                }
            }
        }

        let new_id = format!("node_{}", self.next_node_index);
        self.next_node_index += 1;

        let merge_node = DagNode {
            id: new_id.clone(),
            parent_ids: collected_parents.into_iter().collect(),
            node_type: "merge".to_string(),
            text: summary_text.to_string(),
            semantic_references: None,
        };

        self.redirect_children(node_ids, &new_id);

        for id in node_ids {
            self.nodes.remove(id);
        }

        self.nodes.insert(new_id.clone(), merge_node);
        if current_is_merged {
            self.current_node_id = Some(new_id.clone());
        }

        Ok(new_id)
    }

    fn redirect_children(&mut self, merged_ids: &[String], new_parent_id: &str) {
        let target_set: HashSet<&String> = merged_ids.iter().collect();
        for node in self.nodes.values_mut() {
            if target_set.contains(&node.id) {
                continue;
            }
            for parent in &mut node.parent_ids {
                if target_set.contains(parent) {
                    *parent = new_parent_id.to_string();
                }
            }
            node.parent_ids.sort();
            node.parent_ids.dedup();
        }
    }

    pub fn delete_node(&mut self, node_id: &str) -> Result<(), String> {
        if !self.nodes.contains_key(node_id) {
            return Err(format!("Node '{node_id}' not found"));
        }

        self.nodes.remove(node_id);

        for node in self.nodes.values_mut() {
            node.parent_ids.retain(|x| x != node_id);
        }

        if self.current_node_id.as_deref() == Some(node_id) {
            self.current_node_id = None;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
