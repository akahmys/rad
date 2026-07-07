use crate::wasm::{WasmState, permissions, rpc, bindings};

use crate::ipc::RasRpcRequest;

impl bindings::wit::Host for WasmState {}

impl bindings::RadExtensionImports for WasmState {
    fn host_rpc(
        &mut self,
        command: bindings::wit::RasRpcCommand,
    ) -> Result<String, String> {
        let rpc_cmd = rad_models::RasRpcCommand::from(command);
        
        permissions::check_permissions(&rpc_cmd, &self.permissions)
            .map_err(|e| format!("Permission denied in extension '{}': {e}", self.name))?;
 
        let orchestrator = self.orchestrator.as_ref().and_then(|w| w.upgrade());
        if let Some(ref orch) = orchestrator {
            let req = RasRpcRequest {
                id: Some("wasm_call".to_string()),
                command: rpc_cmd.clone(),
            };
            if let Ok(buf) = serde_json::to_vec(&req) {
                orch.verify_rpc_exclude(&self.name, &req, &buf)
                    .map_err(|e| format!("Extension '{}' RPC verification failed: {e}", self.name))?;
            }
        }

        let result = rpc::execute_rpc_command(
            &rpc_cmd,
            &*self.sandbox,
            &*self.process_manager,
            &*self.dag,
            &*self.network,
            &self.active_processes,
            &self.active_mcp_servers,
            &self.event_tx,
            &self.llm_timeout_policy,
            orchestrator.as_ref(),
            "wasm_call".to_string(),
            self.hitl_enabled,
        );

        match result {
            Ok(val) => Ok(val.to_string()),
            Err(e) => Err(format!("RPC command execution failed: {e}")),
        }
    }
}

impl bindings::rad_orchestrator::RadOrchestratorImports for WasmState {
    fn host_rpc(
        &mut self,
        command: bindings::wit::RasRpcCommand,
    ) -> Result<String, String> {
        bindings::RadExtensionImports::host_rpc(self, command)
    }
}

impl bindings::rad_security_guard::RadSecurityGuardImports for WasmState {
    fn host_rpc(
        &mut self,
        command: bindings::wit::RasRpcCommand,
    ) -> Result<String, String> {
        bindings::RadExtensionImports::host_rpc(self, command)
    }
}

impl bindings::rad_tool_provider::RadToolProviderImports for WasmState {
    fn host_rpc(
        &mut self,
        command: bindings::wit::RasRpcCommand,
    ) -> Result<String, String> {
        bindings::RadExtensionImports::host_rpc(self, command)
    }
}
