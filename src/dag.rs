pub use rad_models::{Dag, DagNode};

#[cfg(test)]
mod tests;

pub struct DagSubsystemImpl {
    pub dag: std::sync::Arc<parking_lot::Mutex<Dag>>,
}

impl crate::subsystems::DagSubsystem for DagSubsystemImpl {
    fn create_node(
        &self,
        parent_id: &str,
        node_type: &str,
    ) -> Result<String, crate::error::UnifiedError> {
        let mut dag = self.dag.lock();
        dag.create_node(parent_id, node_type)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn set_node_text(&self, node_id: &str, text: &str) -> Result<(), crate::error::UnifiedError> {
        let mut dag = self.dag.lock();
        dag.set_node_text(node_id, text)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn merge_nodes(
        &self,
        node_ids: &[String],
        summary_text: &str,
    ) -> Result<String, crate::error::UnifiedError> {
        let mut dag = self.dag.lock();
        dag.merge_nodes(node_ids, summary_text)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn delete_node(&self, node_id: &str) -> Result<(), crate::error::UnifiedError> {
        let mut dag = self.dag.lock();
        dag.delete_node(node_id)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn get_dag(&self) -> Result<serde_json::Value, crate::error::UnifiedError> {
        let dag = self.dag.lock();
        serde_json::to_value(&*dag)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Serialization error: {e}"), "Dag"))
    }
}
