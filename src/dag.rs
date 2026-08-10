pub use rad_models::{Dag, DagNode};

#[cfg(test)]
mod tests;

pub struct DagSubsystemImpl {
    /// The host's copy.
    ///
    /// **Authoritative only while no `dag` module is loaded.** With one, the
    /// module owns the graph and this becomes a cache refreshed from its
    /// replies after every mutation — which is what keeps the readers that
    /// still hold this `Arc` directly (`src/main.rs`'s auto-save,
    /// `command/tree.rs`, `command/compact.rs`) correct without touching them.
    /// They move in AWU 988 and this field goes with them.
    pub dag: std::sync::Arc<parking_lot::Mutex<Dag>>,
    /// Set when a kernel is available; `None` in the test harnesses that build
    /// a subsystem without one.
    pub kernel: Option<std::sync::Arc<crate::kernel::KernelShared>>,
}

impl DagSubsystemImpl {
    /// Runs one operation on the `dag` module, or `None` if no module provides
    /// it — in which case the caller works on the local copy as before.
    ///
    /// Routed by method rather than by module name, so a replacement
    /// registered under another name still answers (§3.6.8).
    fn on_module(
        &self,
        method: &str,
        payload: &serde_json::Value,
    ) -> Option<Result<String, String>> {
        let kernel = self.kernel.as_ref()?;
        kernel.provider_of(method)?;
        Some(kernel.call("host", "dag", method, &payload.to_string()))
    }

    /// Pulls the module's graph into the local copy.
    ///
    /// Called after every mutation rather than lazily: the readers that still
    /// hold the `Arc` do not know a module exists, so the copy has to be right
    /// whenever they look, not whenever someone remembers to refresh it.
    fn refresh_from_module(&self) {
        let Some(Ok(reply)) = self.on_module("dag.get", &serde_json::json!({})) else {
            return;
        };
        if let Ok(fresh) = serde_json::from_str::<Dag>(&reply) {
            *self.dag.lock() = fresh;
        }
    }

    /// The shared shape of a mutating call: ask the module, refresh the copy,
    /// and hand back whatever it replied.
    fn mutate_on_module(
        &self,
        method: &str,
        payload: &serde_json::Value,
    ) -> Option<Result<serde_json::Value, crate::error::UnifiedError>> {
        let reply = self.on_module(method, payload)?;
        let outcome = reply
            .and_then(|r| serde_json::from_str(&r).map_err(|e| e.to_string()))
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"));
        if outcome.is_ok() {
            self.refresh_from_module();
        }
        Some(outcome)
    }
}

impl crate::subsystems::DagSubsystem for DagSubsystemImpl {
    fn create_node(
        &self,
        parent_id: &str,
        node_type: &str,
    ) -> Result<String, crate::error::UnifiedError> {
        let payload = serde_json::json!({ "parent_id": parent_id, "node_type": node_type });
        if let Some(outcome) = self.mutate_on_module("dag.create_node", &payload) {
            return outcome.map(|v| v["id"].as_str().unwrap_or_default().to_string());
        }
        let mut dag = self.dag.lock();
        dag.create_node(parent_id, node_type)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn set_node_text(&self, node_id: &str, text: &str) -> Result<(), crate::error::UnifiedError> {
        let payload = serde_json::json!({ "node_id": node_id, "text": text });
        if let Some(outcome) = self.mutate_on_module("dag.set_node_text", &payload) {
            return outcome.map(|_| ());
        }
        let mut dag = self.dag.lock();
        dag.set_node_text(node_id, text)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn merge_nodes(
        &self,
        node_ids: &[String],
        summary_text: &str,
    ) -> Result<String, crate::error::UnifiedError> {
        let payload = serde_json::json!({ "node_ids": node_ids, "summary_text": summary_text });
        if let Some(outcome) = self.mutate_on_module("dag.merge_nodes", &payload) {
            return outcome.map(|v| v["id"].as_str().unwrap_or_default().to_string());
        }
        let mut dag = self.dag.lock();
        dag.merge_nodes(node_ids, summary_text)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn delete_node(&self, node_id: &str) -> Result<(), crate::error::UnifiedError> {
        let payload = serde_json::json!({ "node_id": node_id });
        if let Some(outcome) = self.mutate_on_module("dag.delete_node", &payload) {
            return outcome.map(|_| ());
        }
        let mut dag = self.dag.lock();
        dag.delete_node(node_id)
            .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"))
    }

    fn get_dag(&self) -> Result<serde_json::Value, crate::error::UnifiedError> {
        if let Some(reply) = self.on_module("dag.get", &serde_json::json!({})) {
            return reply
                .and_then(|r| serde_json::from_str(&r).map_err(|e| e.to_string()))
                .map_err(|e| crate::error::UnifiedError::l1(e, "Dag"));
        }
        let dag = self.dag.lock();
        serde_json::to_value(&*dag)
            .map_err(|e| crate::error::UnifiedError::l1(format!("Serialization error: {e}"), "Dag"))
    }
}
