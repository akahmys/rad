pub use rad_models::{Dag, DagNode};

#[cfg(test)]
mod tests;

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
