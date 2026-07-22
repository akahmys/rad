use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use wasmtime::component::{Component, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

use super::{WasmRuntime, WasmState, bindings};
use crate::config::PermissionConfig;
use crate::ipc::RasCoreEvent;
use crate::process::RunningProcess;
use crate::subsystems::{DagSubsystem, FsSubsystem, NetworkSubsystem, ProcessSubsystem};

impl WasmRuntime {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        wasm_path: &Path,
        role: String,
        permissions: PermissionConfig,
        sandbox: Arc<dyn FsSubsystem>,
        process_manager: Arc<dyn ProcessSubsystem>,
        dag: Arc<dyn DagSubsystem>,
        network: Arc<dyn NetworkSubsystem>,
        active_processes: Arc<Mutex<HashMap<String, RunningProcess>>>,
        event_tx: std::sync::mpsc::Sender<RasCoreEvent>,
        orchestrator: Option<std::sync::Weak<crate::orchestrator::Orchestrator>>,
        hitl_enabled: bool,
    ) -> Result<Self, String> {
        let mut config = wasmtime::Config::new();
        config.wasm_multi_memory(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(|e| format!("Failed to create Engine: {e}"))?;
        let component = Component::from_file(&engine, wasm_path)
            .map_err(|e| format!("Failed to load Wasm component: {e}"))?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker)
            .map_err(|e| format!("Linker error WASI: {e}"))?;

        match role.as_str() {
            "orchestrator" => {
                bindings::rad_orchestrator::RadOrchestrator::add_to_linker(&mut linker, |s| s)
                    .map_err(|e| format!("Linker error RadOrchestrator: {e}"))?
            }
            "security" => {
                bindings::rad_security_guard::RadSecurityGuard::add_to_linker(&mut linker, |s| s)
                    .map_err(|e| format!("Linker error RadSecurityGuard: {e}"))?
            }
            "tool-provider" => {
                bindings::rad_tool_provider::RadToolProvider::add_to_linker(&mut linker, |s| s)
                    .map_err(|e| format!("Linker error RadToolProvider: {e}"))?
            }
            "llm-connector" => {
                bindings::rad_llm_connector::LlmConnector::add_to_linker(&mut linker, |s| s)
                    .map_err(|e| format!("Linker error LlmConnector: {e}"))?
            }
            "context-tools" => bindings::rad_context_tools::ContextToolsExtension::add_to_linker(
                &mut linker,
                |s| s,
            )
            .map_err(|e| format!("Linker error ContextToolsExtension: {e}"))?,
            _ => bindings::RadExtension::add_to_linker(&mut linker, |s| s)
                .map_err(|e| format!("Linker error RadExtension: {e}"))?,
        }

        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .inherit_env()
            .build();
        let resource_table = ResourceTable::new();

        let state = WasmState {
            name,
            sandbox,
            process_manager,
            dag,
            network,
            permissions,
            active_processes,
            active_mcp_servers: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
            llm_timeout_policy: Arc::new(Mutex::new(crate::ipc::TimeoutPolicy::Infinite)),
            orchestrator,
            hitl_enabled,
            wasi,
            resource_table,
        };

        let mut store = Store::new(&engine, state);
        let instance = linker
            .instantiate(&mut store, &component)
            .map_err(|e| format!("Failed to instantiate component: {e}"))?;

        let mut extension = None;
        let mut orchestrator = None;
        let mut security_guard = None;
        let mut tool_provider = None;
        let mut llm_connector = None;
        let mut context_tools = None;

        match role.as_str() {
            "orchestrator" => {
                orchestrator = Some(
                    bindings::rad_orchestrator::RadOrchestrator::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create orchestrator bindings: {e}"))?,
                )
            }
            "security" => {
                security_guard = Some(
                    bindings::rad_security_guard::RadSecurityGuard::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create security bindings: {e}"))?,
                )
            }
            "tool-provider" => {
                tool_provider = Some(
                    bindings::rad_tool_provider::RadToolProvider::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create tool-provider bindings: {e}"))?,
                )
            }
            "llm-connector" => {
                llm_connector = Some(
                    bindings::rad_llm_connector::LlmConnector::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create llm-connector bindings: {e}"))?,
                )
            }
            "context-tools" => {
                context_tools = Some(
                    bindings::rad_context_tools::ContextToolsExtension::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create context-tools bindings: {e}"))?,
                )
            }
            _ => {
                extension = Some(
                    bindings::RadExtension::new(&mut store, &instance)
                        .map_err(|e| format!("Failed to create legacy bindings: {e}"))?,
                )
            }
        }

        Ok(Self {
            store,
            extension,
            orchestrator,
            security_guard,
            tool_provider,
            llm_connector,
            context_tools,
            instance,
            role,
        })
    }
}
