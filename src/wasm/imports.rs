use crate::wasm::format_wasm_error;
use crate::wasm::{WasmState, bindings, permissions, rpc};
use futures_util::StreamExt;
use parking_lot::Mutex;
use wasmtime_wasi::WasiView;

use crate::ipc::RasRpcRequest;

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
            Err(e) => Err(format!("RPC command execution failed: {e}")),
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
        let orchestrator = self
            .orchestrator
            .as_ref()
            .and_then(|w| w.upgrade())
            .ok_or_else(|| "Orchestrator unavailable".to_string())?;

        // Find the tool provider runtime
        let runtimes = orchestrator.wasm_runtime.lock();
        let mut provider_opt = None;
        for (_id, runtime_arc) in runtimes.iter() {
            let runtime = runtime_arc.lock();
            if runtime.tool_provider.is_some() {
                provider_opt = Some(runtime_arc.clone());
                break;
            }
        }
        drop(runtimes);

        let provider_arc = provider_opt.ok_or_else(|| "No Tool Provider found".to_string())?;
        let mut provider = provider_arc.lock();

        let provider_ref = &mut *provider;
        // Call execute_tool on the provider. It returns a Resource<HostExecution> in the provider's store.
        let ext_name = provider_ref.store.data().name.clone();
        let provider_res = {
            let prov = provider_ref.tool_provider.as_ref().unwrap();
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

impl bindings::rad_orchestrator::RadOrchestratorImports for WasmState {
    fn host_rpc(&mut self, command: bindings::wit::RasRpcCommand) -> Result<String, String> {
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

impl bindings::rad_security_guard::RadSecurityGuardImports for WasmState {
    fn host_rpc(&mut self, command: bindings::wit::RasRpcCommand) -> Result<String, String> {
        bindings::RadExtensionImports::host_rpc(self, command)
    }
}

impl bindings::rad_tool_provider::RadToolProviderImports for WasmState {
    fn host_rpc(&mut self, command: bindings::wit::RasRpcCommand) -> Result<String, String> {
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

impl bindings::wit::HostStreamHandle for WasmState {
    fn read(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        max_bytes: u32,
    ) -> Result<Vec<u8>, String> {
        use std::io::Read;
        let stream = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        match stream {
            crate::wasm::HostStream::File(file) => {
                let mut buf = vec![0u8; max_bytes as usize];
                match file.read(&mut buf) {
                    Ok(n) => {
                        buf.truncate(n);
                        Ok(buf)
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            crate::wasm::HostStream::PipeReader(rx_mutex) => {
                let rx = rx_mutex.lock();
                match rx.try_recv() {
                    Ok(data) => Ok(data),
                    Err(std::sync::mpsc::TryRecvError::Empty) => Ok(vec![]),
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Ok(vec![]),
                }
            }
            crate::wasm::HostStream::PipeWriter(_) => {
                Err("Cannot read from a write-only stream".to_string())
            }
            crate::wasm::HostStream::Closed => Ok(vec![]),
        }
    }

    fn write(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostStream>,
        data: Vec<u8>,
    ) -> Result<(), String> {
        use std::io::Write;
        let stream = self.table().get_mut(&self_).map_err(|e| e.to_string())?;
        match stream {
            crate::wasm::HostStream::File(file) => match file.write_all(&data) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.to_string()),
            },
            crate::wasm::HostStream::PipeWriter(stdin_mutex) => {
                let mut stdin = stdin_mutex.lock();
                match stdin.write_all(&data) {
                    Ok(_) => {
                        let _ = stdin.flush();
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            crate::wasm::HostStream::PipeReader(_) => {
                Err("Cannot write to a read-only stream".to_string())
            }
            crate::wasm::HostStream::Closed => Err("Stream is closed".to_string()),
        }
    }

    fn close(&mut self, self_: wasmtime::component::Resource<crate::wasm::HostStream>) {
        if let Ok(stream) = self.table().get_mut(&self_) {
            *stream = crate::wasm::HostStream::Closed;
        }
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostStream>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}

impl bindings::wit::HostFileHandle for WasmState {
    fn read_at(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
        offset: u64,
        len: u32,
    ) -> Result<Vec<u8>, String> {
        use std::os::unix::fs::FileExt;
        let file_ref = self.table().get(&self_).map_err(|e| e.to_string())?;
        let file = &file_ref.file;
        let mut buf = vec![0u8; len as usize];
        match file.read_exact_at(&mut buf, offset) {
            Ok(_) => Ok(buf),
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                let mut partial_buf = vec![0u8; len as usize];
                let n = file.read_at(&mut partial_buf, offset).unwrap_or(0);
                partial_buf.truncate(n);
                Ok(partial_buf)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn write_at(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
        offset: u64,
        data: Vec<u8>,
    ) -> Result<(), String> {
        use std::os::unix::fs::FileExt;
        let file_ref = self.table().get(&self_).map_err(|e| e.to_string())?;
        let file = &file_ref.file;
        match file.write_all_at(&data, offset) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn get_stream(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostFile>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let file_ref = self.table().get(&self_).unwrap();
        let file_dup = file_ref.file.try_clone().unwrap();
        self.table()
            .push(crate::wasm::HostStream::File(file_dup))
            .unwrap()
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostFile>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}

impl bindings::wit::HostExecutionHandle for WasmState {
    fn get_stdout(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let rx_opt = {
            let exec = self.table().get_mut(&self_).unwrap();
            exec.stdout.lock().take()
        };
        if let Some(rx) = rx_opt {
            self.table()
                .push(crate::wasm::HostStream::PipeReader(Mutex::new(rx)))
                .unwrap()
        } else {
            panic!("Stdout stream already acquired or unavailable")
        }
    }

    fn get_stderr(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let rx_opt = {
            let exec = self.table().get_mut(&self_).unwrap();
            exec.stderr.lock().take()
        };
        if let Some(rx) = rx_opt {
            self.table()
                .push(crate::wasm::HostStream::PipeReader(Mutex::new(rx)))
                .unwrap()
        } else {
            panic!("Stderr stream already acquired or unavailable")
        }
    }

    fn get_stdin(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> wasmtime::component::Resource<crate::wasm::HostStream> {
        let stdin_opt = {
            let exec = self.table().get_mut(&self_).unwrap();
            exec.stdin.lock().take()
        };
        if let Some(stdin) = stdin_opt {
            self.table()
                .push(crate::wasm::HostStream::PipeWriter(Mutex::new(stdin)))
                .unwrap()
        } else {
            panic!("Stdin stream already acquired or unavailable")
        }
    }

    fn wait(
        &mut self,
        self_: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> Result<i32, String> {
        let exec = self.table().get_mut(&self_).unwrap();
        let mut running = exec.running.lock();
        match running.child.wait() {
            Ok(status) => {
                running.unregister_pgid();
                Ok(status.exit_code() as i32)
            }
            Err(e) => Err(e.to_string()),
        }
    }

    fn kill(&mut self, self_: wasmtime::component::Resource<crate::wasm::HostExecution>) {
        let exec = self.table().get_mut(&self_).unwrap();
        let mut running = exec.running.lock();
        running.kill_group();
    }

    fn drop(
        &mut self,
        rep: wasmtime::component::Resource<crate::wasm::HostExecution>,
    ) -> Result<(), wasmtime::Error> {
        self.table().delete(rep)?;
        Ok(())
    }
}
