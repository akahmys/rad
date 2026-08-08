// Trait-delegation boilerplate, split out of `imports_rpc.rs` to stay under
// the 300-line file limit.
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

            fn execute_tool_text(
                &mut self,
                name: String,
                arguments: String,
            ) -> Result<String, String> {
                bindings::RadExtensionImports::execute_tool_text(self, name, arguments)
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
