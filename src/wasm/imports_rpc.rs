/// Host-side RPC import implementations for all Wasm extension roles.
///
/// Contains the `host_rpc` and `open_file` trait implementations directly;
/// `open_process`, `execute_tool`, and `open_http_stream` are implemented in
/// `imports_process.rs`, `imports_tool.rs`, and `imports_http.rs`
/// respectively (split out to stay under the 300-line file limit), with
/// this file's trait impl methods delegating to them. Cross-world trait
/// delegation boilerplate lives in `imports_delegate.rs`.
use crate::ipc::RasRpcRequest;
use crate::wasm::{WasmState, bindings, permissions, rpc};
use wasmtime_wasi::WasiView;

impl bindings::wit::Host for WasmState {}

impl bindings::RadExtensionImports for WasmState {
    fn host_rpc(&mut self, command: bindings::wit::RasRpcCommand) -> Result<String, String> {
        if self.is_aborted() {
            return Err("Task aborted by user".to_string());
        }
        let rpc_cmd = rad_models::RasRpcCommand::from(command);

        permissions::check_permissions(&rpc_cmd, &self.permissions, self.sandbox.workspace_dir())
            .map_err(|e| format!("Permission denied in extension '{}': {e}", self.name))?;

        let orchestrator = self.orchestrator.as_ref().and_then(|w| w.upgrade());
        if let Some(ref orch) = orchestrator {
            let req = RasRpcRequest {
                id: Some("wasm_call".to_string()),
                command: rpc_cmd.clone(),
            };
            if let Ok(buf) = serde_json::to_vec(&req) {
                orch.verify_rpc_exclude(&self.name, &req, &buf)
                    .map_err(|e| {
                        format!("Extension '{}' RPC verification failed: {e}", self.name)
                    })?;
            }
        }

        let result = rpc::execute_rpc_command(
            &rpc_cmd,
            &*self.sandbox,
            &*self.process_manager,
            &*self.dag,
            &*self.network,
            &self.active_processes,
            &self.event_tx,
            &self.llm_timeout_policy,
            orchestrator.as_ref(),
            "wasm_call".to_string(),
            self.hitl_enabled,
            &self.name,
        );

        match result {
            Ok(val) => Ok(val.to_string()),
            Err(e) => {
                if crate::error::UnifiedError::from_json_string(&e).is_some() {
                    Err(e)
                } else {
                    let wrapped = crate::error::UnifiedError::l1(e, "Internal");
                    Err(wrapped.to_json_string())
                }
            }
        }
    }

    fn open_file(
        &mut self,
        path: String,
        writeable: bool,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostFile>, String> {
        let workspace = self.sandbox.workspace_dir();
        let resolved = resolve_and_verify_path(workspace, &path)?;

        // Validate via security guard (verify_rpc)
        let cmd = rad_models::RasRpcCommand::OpenFile {
            path: resolved.clone(),
            writeable,
        };

        permissions::check_permissions(&cmd, &self.permissions, self.sandbox.workspace_dir())
            .map_err(|e| format!("Permission denied in extension '{}': {e}", self.name))?;

        let orchestrator = self.orchestrator.as_ref().and_then(|w| w.upgrade());
        if let Some(ref orch) = orchestrator {
            let req = RasRpcRequest {
                id: Some("wasm_call".to_string()),
                command: cmd.clone(),
            };
            let buf = serde_json::to_vec(&req)
                .map_err(|e| format!("Failed to serialize request: {e}"))?;
            if let Err(e) = orch.verify_rpc_exclude(&self.name, &req, &buf) {
                return Err(format!("Security verification failed: {e}"));
            }
        }

        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        if writeable {
            options.write(true).create(true);
        }
        let file = match options.open(&resolved) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to open file: {e}")),
        };

        let res = match self.table().push(crate::wasm::HostFile {
            path: resolved,
            file,
        }) {
            Ok(r) => r,
            Err(e) => return Err(e.to_string()),
        };
        Ok(res)
    }

    fn open_process(
        &mut self,
        command: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
        crate::wasm::imports_process::open_process(self, command)
    }

    fn execute_tool(
        &mut self,
        name: String,
        arguments: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
        crate::wasm::imports_tool::execute_tool(self, name, arguments)
    }

    fn open_http_stream(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        body: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostStream>, String> {
        crate::wasm::imports_http::open_http_stream(self, url, headers, body)
    }
}

pub(crate) fn resolve_and_verify_path(
    workspace: &std::path::Path,
    user_path_str: &str,
) -> Result<std::path::PathBuf, String> {
    use std::path::Path;
    let user_path = Path::new(user_path_str);
    let absolute_target = if user_path.is_relative() {
        workspace.join(user_path)
    } else {
        user_path.to_path_buf()
    };

    let canonical_workspace = workspace
        .canonicalize()
        .map_err(|e| format!("Failed to canonicalize workspace dir: {e}"))?;

    let canonical_target = match absolute_target.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            if let Some(parent) = absolute_target.parent() {
                let canonical_parent = parent
                    .canonicalize()
                    .map_err(|e| format!("Failed to resolve path: {e}"))?;
                if !canonical_parent.starts_with(&canonical_workspace) {
                    return Err("Access denied: path escapes sandbox".to_string());
                }
                if let Some(file_name) = absolute_target.file_name() {
                    return Ok(canonical_parent.join(file_name));
                }
            }
            return Err("Failed to resolve target path".to_string());
        }
    };

    if canonical_target.starts_with(&canonical_workspace) {
        Ok(canonical_target)
    } else {
        Err("Access denied: path escapes sandbox".to_string())
    }
}
