// Trait-delegation boilerplate and the `context-tools` shell-command host
// bridge, split out of `imports_rpc.rs` to stay under the 300-line file
// limit.
use crate::wasm::{WasmState, bindings};

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
