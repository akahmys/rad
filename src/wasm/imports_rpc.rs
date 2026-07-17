/// Host-side RPC import implementations for all Wasm extension roles.
///
/// Contains the `host_rpc`, `open_file`, `open_process`, `execute_tool`, and
/// `open_http_stream` trait implementations delegated from each WIT world.
use crate::ipc::RasRpcRequest;
use crate::wasm::format_wasm_error;
use crate::wasm::{WasmState, bindings, permissions, rpc};
use futures_util::StreamExt;
use parking_lot::Mutex;
use wasmtime_wasi::WasiView;

struct MutexWriteWrapper(parking_lot::Mutex<Box<dyn std::io::Write + Send>>);
impl std::io::Write for MutexWriteWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.lock().flush()
    }
}

impl bindings::wit::Host for WasmState {}

impl bindings::RadExtensionImports for WasmState {
    fn host_rpc(&mut self, command: bindings::wit::RasRpcCommand) -> Result<String, String> {
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
            &self.active_mcp_servers,
            &self.event_tx,
            &self.llm_timeout_policy,
            orchestrator.as_ref(),
            "wasm_call".to_string(),
            self.hitl_enabled,
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
        // Validate command via security guard check
        let cmd = rad_models::RasRpcCommand::SpawnBashProcess {
            command: command.clone(),
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

        // HITL check
        if self.hitl_enabled {
            let approved = crate::wasm::rpc_process::ask_human_approval_internal(&format!(
                "Spawn bash process: {command}"
            ))?;
            if !approved {
                return Err("User rejected execution of tool spawn_bash_process".to_string());
            }
        }

        // Spawn process
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let call_id = format!("wasm_proc_{ts}");
        let mut running = self.process_manager.spawn_bash_process(
            &command,
            Some(self.sandbox.workspace_dir()),
            call_id,
            "spawn_bash_process".to_string(),
            format!("{{\"command\":\"{command}\"}}"),
        )?;

        // Extract stdin, stdout, stderr channels
        let stdout_rx = running.stdout_rx.take();
        let stderr_rx = running.stderr_rx.take();
        let stdin_writer = if let Some(stdin_tx) = running.stdin_tx.take() {
            let wrapper = MutexWriteWrapper(stdin_tx);
            Some(Box::new(wrapper) as Box<dyn std::io::Write + Send>)
        } else {
            None
        };

        // Insert into Host ResourceTable
        let host_exec = crate::wasm::HostExecution {
            running: Mutex::new(running),
            stdout: Mutex::new(stdout_rx),
            stderr: Mutex::new(stderr_rx),
            stdin: Mutex::new(stdin_writer),
        };
        let res = self.table().push(host_exec).map_err(|e| e.to_string())?;
        Ok(res)
    }

    fn execute_tool(
        &mut self,
        name: String,
        arguments: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
        crate::log_host!(
            "[HOST] RadExtensionImports::execute_tool called: name = '{}', args = '{}'",
            name,
            arguments
        );
        let mut provider_opt = None;

        if let Some(orchestrator) = self.orchestrator.as_ref().and_then(|w| w.upgrade()) {
            // Find the tool provider runtime
            let runtimes = orchestrator.wasm_runtime.lock();
            for (_id, runtime_arc) in runtimes.iter() {
                let Some(runtime) = runtime_arc.try_lock() else {
                    continue;
                };
                if runtime.tool_provider.is_some() {
                    provider_opt = Some(runtime_arc.clone());
                    break;
                }
            }
        }

        let provider_arc = match provider_opt {
            Some(arc) => arc,
            None => {
                return self.execute_core_tool_fallback(&name, &arguments);
            }
        };
        let mut provider = provider_arc.lock();

        let provider_ref = &mut *provider;
        let ext_name = provider_ref.store.data().name.clone();
        let provider_res = {
            let prov = provider_ref
                .tool_provider
                .as_ref()
                .ok_or_else(|| "Tool provider bindings missing".to_string())?;
            prov.call_execute_tool(&mut provider_ref.store, &name, &arguments)
                .map_err(|e| format_wasm_error(&ext_name, "execute_tool", &e))??
        };

        // Extract the HostExecution from the provider's table
        let provider_state = provider.store.data_mut();
        let host_exec = provider_state
            .table()
            .delete(provider_res)
            .map_err(|e| format!("Failed to extract resource from provider table: {e}"))?;

        // Push it into our (the caller/orchestrator) table
        let our_res = self
            .table()
            .push(host_exec)
            .map_err(|e| format!("Failed to insert resource into caller table: {e}"))?;

        Ok(our_res)
    }

    fn open_http_stream(
        &mut self,
        url: String,
        headers: Vec<(String, String)>,
        body: String,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostStream>, String> {
        let cmd = rad_models::RasRpcCommand::OpenHttpStream {
            url: url.clone(),
            headers: std::collections::HashMap::new(),
            body: body.clone(),
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

        let (tx, rx) = std::sync::mpsc::channel();

        // Convert headers to HashMap
        let mut header_map = std::collections::HashMap::new();
        for (k, v) in headers {
            header_map.insert(k, v);
        }

        // We can just use tokio to fetch and stream bytes
        std::thread::spawn(move || {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    let _ = tx.send(format!("Error creating runtime: {e}").into_bytes());
                    return;
                }
            };

            let url_owned = url.clone();
            let body_owned = body.clone();

            rt.block_on(async {
                let client = reqwest::Client::new();
                let mut req_headers = reqwest::header::HeaderMap::new();
                for (k, v) in header_map {
                    if let (Ok(name), Ok(value)) = (
                        reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                        reqwest::header::HeaderValue::from_str(&v),
                    ) {
                        req_headers.insert(name, value);
                    }
                }

                let response_res = client
                    .post(&url_owned)
                    .headers(req_headers)
                    .body(body_owned)
                    .send()
                    .await;

                let response = match response_res {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = tx.send(format!("HTTP request failed: {e}").into_bytes());
                        return;
                    }
                };

                if !response.status().is_success() {
                    let _ =
                        tx.send(format!("HTTP status error: {}", response.status()).into_bytes());
                    return;
                }

                let mut stream = response.bytes_stream();
                while let Some(chunk_res) = stream.next().await {
                    match chunk_res {
                        Ok(bytes) => {
                            if tx.send(bytes.to_vec()).is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(format!("Stream read error: {e}").into_bytes());
                            break;
                        }
                    }
                }
            });
        });

        let res = self
            .table()
            .push(crate::wasm::HostStream::PipeReader(Mutex::new(rx)))
            .map_err(|e| e.to_string())?;
        Ok(res)
    }
}

/// Delegation macro: generates trait impls that forward all methods to
/// `RadExtensionImports`, eliminating boilerplate for each WIT world.
macro_rules! delegate_extension_imports {
    ($trait_path:path) => {
        impl $trait_path for WasmState {
            fn host_rpc(
                &mut self,
                command: bindings::wit::RasRpcCommand,
            ) -> Result<String, String> {
                bindings::RadExtensionImports::host_rpc(self, command)
            }

            fn open_file(
                &mut self,
                path: String,
                writeable: bool,
            ) -> Result<wasmtime::component::Resource<crate::wasm::HostFile>, String> {
                bindings::RadExtensionImports::open_file(self, path, writeable)
            }

            fn open_process(
                &mut self,
                command: String,
            ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
                bindings::RadExtensionImports::open_process(self, command)
            }

            fn execute_tool(
                &mut self,
                name: String,
                arguments: String,
            ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
                bindings::RadExtensionImports::execute_tool(self, name, arguments)
            }

            fn open_http_stream(
                &mut self,
                url: String,
                headers: Vec<(String, String)>,
                body: String,
            ) -> Result<wasmtime::component::Resource<crate::wasm::HostStream>, String> {
                bindings::RadExtensionImports::open_http_stream(self, url, headers, body)
            }
        }
    };
    // Variant for security guard (host_rpc only)
    ($trait_path:path, rpc_only) => {
        impl $trait_path for WasmState {
            fn host_rpc(
                &mut self,
                command: bindings::wit::RasRpcCommand,
            ) -> Result<String, String> {
                bindings::RadExtensionImports::host_rpc(self, command)
            }
        }
    };
}

delegate_extension_imports!(bindings::rad_orchestrator::RadOrchestratorImports);
delegate_extension_imports!(
    bindings::rad_security_guard::RadSecurityGuardImports,
    rpc_only
);
delegate_extension_imports!(bindings::rad_tool_provider::RadToolProviderImports);

impl bindings::rad_context_tools::radcomp::context_tools::types::Host for WasmState {}

impl bindings::rad_context_tools::radcomp::context_tools::host_rpc::Host for WasmState {
    fn call(
        &mut self,
        command: bindings::rad_context_tools::radcomp::context_tools::types::RasRpcCommand,
    ) -> Result<String, String> {
        let bindings::rad_context_tools::radcomp::context_tools::types::RasRpcCommand::Command(
            cmd_str,
        ) = command;

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .current_dir(self.sandbox.workspace_dir())
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let stdout_str = String::from_utf8_lossy(&out.stdout).into_owned();
                    Ok(stdout_str)
                } else {
                    let stderr_str = String::from_utf8_lossy(&out.stderr).into_owned();
                    Err(format!(
                        "Command failed with status {}: {stderr_str}",
                        out.status
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute command: {e}")),
        }
    }
}

impl bindings::rad_web_access::radcomp::web_access::types::Host for WasmState {}

impl bindings::rad_web_access::radcomp::web_access::host_rpc::Host for WasmState {
    fn call(
        &mut self,
        command: bindings::rad_web_access::radcomp::web_access::types::RasRpcCommand,
    ) -> Result<String, String> {
        let bindings::rad_web_access::radcomp::web_access::types::RasRpcCommand::Command(cmd_str) =
            command;

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .current_dir(self.sandbox.workspace_dir())
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let stdout_str = String::from_utf8_lossy(&out.stdout).into_owned();
                    Ok(stdout_str)
                } else {
                    let stderr_str = String::from_utf8_lossy(&out.stderr).into_owned();
                    Err(format!(
                        "Command failed with status {}: {stderr_str}",
                        out.status
                    ))
                }
            }
            Err(e) => Err(format!("Failed to execute command: {e}")),
        }
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

impl WasmState {
    fn execute_core_tool_fallback(
        &mut self,
        name: &str,
        arguments: &str,
    ) -> Result<wasmtime::component::Resource<crate::wasm::HostExecution>, String> {
        crate::log_host!(
            "[HOST] WIT Core Tool Fallback: executing '{}' with args '{}'",
            name,
            arguments
        );

        // Reconstruct RasRpcCommand to perform security check
        crate::log_host!("[HOST] Fallback: parsing arguments and resolving path");
        let rpc_cmd = match name {
            "read" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                }
                let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
                let resolved = resolve_and_verify_path(self.sandbox.workspace_dir(), &args.path)?;
                rad_models::RasRpcCommand::FileRead { path: resolved }
            }
            "write" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                    content: String,
                }
                let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
                let resolved = resolve_and_verify_path(self.sandbox.workspace_dir(), &args.path)?;
                rad_models::RasRpcCommand::FileWrite {
                    path: resolved,
                    data: args.content.into_bytes(),
                }
            }
            "edit" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                    diff: String,
                }
                let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
                let resolved = resolve_and_verify_path(self.sandbox.workspace_dir(), &args.path)?;
                rad_models::RasRpcCommand::FileEditPatch {
                    path: resolved,
                    diff: args.diff,
                }
            }
            "bash" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    command: String,
                }
                let args: Args = serde_json::from_str(arguments).map_err(|e| e.to_string())?;
                rad_models::RasRpcCommand::SpawnBashProcess {
                    command: args.command,
                }
            }
            other => return Err(format!("Unknown core tool: {other}")),
        };

        permissions::check_permissions(&rpc_cmd, &self.permissions, self.sandbox.workspace_dir())
            .map_err(|e| format!("Permission denied in extension '{}': {e}", self.name))?;

        crate::log_host!("[HOST] Fallback: parsed command, fetching orchestrator");
        let orchestrator = self.orchestrator.as_ref().and_then(|w| w.upgrade());
        if let Some(ref orch) = orchestrator {
            let req = RasRpcRequest {
                id: Some("wasm_call".to_string()),
                command: rpc_cmd.clone(),
            };
            let buf = serde_json::to_vec(&req)
                .map_err(|e| format!("Failed to serialize request: {e}"))?;
            crate::log_host!("[HOST] Fallback: calling verify_rpc_exclude");
            if let Err(e) = orch.verify_rpc_exclude(&self.name, &req, &buf) {
                crate::log_host!("[HOST] Fallback: verify_rpc_exclude rejected request");
                return Err(format!("Security verification failed: {e}"));
            }
            crate::log_host!("[HOST] Fallback: verify_rpc_exclude accepted request");
        }

        let command_to_run = match name {
            "read" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                }
                let args: Args = serde_json::from_str(arguments)
                    .map_err(|e| format!("Failed to parse read args: {e}"))?;
                format!("cat '{}'", args.path)
            }
            "write" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                    content: String,
                }
                let args: Args = serde_json::from_str(arguments)
                    .map_err(|e| format!("Failed to parse write args: {e}"))?;

                let resolved = resolve_and_verify_path(self.sandbox.workspace_dir(), &args.path)?;
                self.sandbox
                    .file_write(&resolved, args.content.as_bytes())?;
                "echo 'File written successfully.'".to_string()
            }
            "edit" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    path: String,
                    diff: String,
                }
                let args: Args = serde_json::from_str(arguments)
                    .map_err(|e| format!("Failed to parse edit args: {e}"))?;

                let resolved = resolve_and_verify_path(self.sandbox.workspace_dir(), &args.path)?;
                self.sandbox.file_edit_patch(&resolved, &args.diff)?;
                "echo 'Patch applied successfully.'".to_string()
            }
            "bash" => {
                #[derive(serde::Deserialize)]
                struct Args {
                    command: String,
                }
                let args: Args = serde_json::from_str(arguments)
                    .map_err(|e| format!("Failed to parse bash args: {e}"))?;
                args.command
            }
            _ => unreachable!(),
        };

        bindings::RadExtensionImports::open_process(self, command_to_run)
    }
}
