use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagNode {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub node_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dag {
    pub nodes: HashMap<String, DagNode>,
    pub current_node_id: Option<String>,
    next_node_index: usize,
}

impl Dag {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new node under the given parent.
    /// If `parent_id` is empty, it creates a root node.
    ///
    /// # Errors
    ///
    /// Returns an error if `parent_id` is not empty and does not exist in the DAG.
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
        };

        self.nodes.insert(new_id.clone(), node);
        self.current_node_id = Some(new_id.clone());

        Ok(new_id)
    }

    /// Sets or updates the text of a specified node.
    ///
    /// # Errors
    ///
    /// Returns an error if the `node_id` does not exist.
    pub fn set_node_text(&mut self, node_id: &str, text: &str) -> Result<(), String> {
        let node = self.nodes.get_mut(node_id).ok_or_else(|| format!("Node '{node_id}' not found"))?;
        node.text = text.to_string();
        Ok(())
    }

    /// Merges multiple nodes into a new merge node, and removes original nodes.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the target `node_ids` do not exist.
    pub fn merge_nodes(&mut self, node_ids: &[String], summary_text: &str) -> Result<String, String> {
        if node_ids.is_empty() {
            return Err("Cannot merge empty list of nodes".to_string());
        }

        let mut collected_parents = HashSet::new();
        let target_set: HashSet<&String> = node_ids.iter().collect();

        for id in node_ids {
            let node = self.nodes.get(id).ok_or_else(|| format!("Node '{id}' not found"))?;
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
        };

        self.redirect_children(node_ids, &new_id);

        for id in node_ids {
            self.nodes.remove(id);
        }

        self.nodes.insert(new_id.clone(), merge_node);
        self.current_node_id = Some(new_id.clone());

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

    /// Deletes a node and removes its references from child nodes.
    ///
    /// # Errors
    ///
    /// Returns an error if the node does not exist.
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

pub struct DagSubsystemImpl {
    pub dag: std::sync::Arc<std::sync::Mutex<Dag>>,
}

impl crate::subsystems::DagSubsystem for DagSubsystemImpl {
    fn create_node(&self, parent_id: &str, node_type: &str) -> Result<String, String> {
        let mut dag = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        dag.create_node(parent_id, node_type)
    }

    fn set_node_text(&self, node_id: &str, text: &str) -> Result<(), String> {
        let mut dag = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        dag.set_node_text(node_id, text)
    }

    fn merge_nodes(&self, node_ids: &[String], summary_text: &str) -> Result<String, String> {
        let mut dag = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        dag.merge_nodes(node_ids, summary_text)
    }

    fn delete_node(&self, node_id: &str) -> Result<(), String> {
        let mut dag = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        dag.delete_node(node_id)
    }

    fn get_dag(&self) -> Result<serde_json::Value, String> {
        let dag = self.dag.lock().map_err(|e| format!("DAG lock error: {e}"))?;
        serde_json::to_value(&*dag).map_err(|e| format!("Serialization error: {e}"))
    }
}
