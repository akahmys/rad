// Wasm runtime lifecycle (init/lookup/clear), split out of `runner.rs` to
// stay under the 300-line file limit.
use super::Orchestrator;
use crate::ipc::RasCoreEvent;
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;

#[cfg(test)]
mod tests;

impl Orchestrator {
    /// Loads every enabled extension that is not already live, and returns the
    /// full set.
    ///
    /// # Errors
    ///
    /// Returns a message if an extension's component fails to load or
    /// instantiate. A tool provider that loads but answers `get_tools` badly is
    /// reported and skipped rather than failing the set — one broken provider
    /// must not take the others down with it.
    pub fn get_or_init_runtimes(
        self: &Arc<Self>,
        event_tx: &Sender<RasCoreEvent>,
    ) -> Result<HashMap<String, Arc<Mutex<WasmRuntime>>>, String> {
        let (extensions, hitl_enabled, llm_stream_heartbeat_ms) = {
            let config_guard = self.config.lock();
            (
                config_guard.extensions.clone(),
                config_guard.core.hitl_enabled,
                config_guard.default_timeout.llm_stream_heartbeat_ms,
            )
        };

        for ext in &extensions {
            if !ext.enabled {
                continue;
            }
            {
                let guard = self.wasm_runtime.lock();
                if guard.contains_key(&ext.name) {
                    continue;
                }
            }

            let permissions = ext.permissions.clone().unwrap_or_default();
            let wasm_path_buf = crate::config::expand_tilde(&ext.source);
            let wasm_path = wasm_path_buf.as_path();
            if wasm_path.exists() {
                let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl {
                    dag: self.dag.clone(),
                });
                let network_subsystem = Arc::new(crate::http::HttpManager);
                let mut runtime = WasmRuntime::new(
                    ext.name.clone(),
                    wasm_path,
                    ext.role.clone(),
                    permissions.clone(),
                    self.sandbox.clone() as Arc<dyn crate::subsystems::FsSubsystem>,
                    self.process_manager.clone() as Arc<dyn crate::subsystems::ProcessSubsystem>,
                    dag_subsystem.clone(),
                    network_subsystem.clone(),
                    self.active_processes.clone(),
                    event_tx.clone(),
                    Some(Arc::downgrade(self)),
                    hitl_enabled,
                    llm_stream_heartbeat_ms,
                )?;

                if runtime.tool_provider.is_some() {
                    match runtime.get_tools() {
                        Ok(json_str) => {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str)
                                && let Some(arr) = val.as_array()
                            {
                                if arr.is_empty() {
                                    if ext.name == "mcp-tool-provider" {
                                        println!(
                                            "\x1b[31m[FAILED] Extension '{}' initialized with 0 tools. See [MCP Diagnostic] lines above for the actual cause.\x1b[0m",
                                            ext.name
                                        );
                                    } else {
                                        println!(
                                            "\x1b[32m[OK] Extension '{}' initialized with 0 tools\x1b[0m",
                                            ext.name
                                        );
                                    }
                                } else {
                                    println!(
                                        "\x1b[32m[OK] Verified {} tools from extension '{}'\x1b[0m",
                                        arr.len(),
                                        ext.name
                                    );
                                }
                            } else {
                                println!(
                                    "\x1b[31m[FAILED] Extension '{}' returned invalid JSON from get_tools: {}\x1b[0m",
                                    ext.name, json_str
                                );
                            }
                        }
                        Err(e) => {
                            println!(
                                "\x1b[31m[FAILED] Extension '{}' get_tools error: {e}\x1b[0m",
                                ext.name
                            );
                        }
                    }
                }

                let mut guard = self.wasm_runtime.lock();
                guard.insert(ext.name.clone(), Arc::new(Mutex::new(runtime)));
            }
        }

        let guard = self.wasm_runtime.lock();
        Ok(guard.clone())
    }

    /// Drops every live runtime so the next `get_or_init_runtimes` rebuilds
    /// them. The recovery half of the crash loop; it cannot fail.
    pub(crate) fn clear_runtimes(&self) {
        self.wasm_runtime.lock().clear();
    }

    /// Resolves the extension currently registered for `role` (e.g.
    /// `"context-tools"`) to its live `WasmRuntime`, via the static
    /// `role` each extension declares in `~/.rad/config.json` — avoids
    /// locking every candidate runtime just to inspect its own `.role`
    /// field while holding the outer `wasm_runtime` lock, the nested-lock
    /// pattern AWU 900 fixed away from. `None` if no enabled extension
    /// declares that role, or it hasn't been initialized yet.
    pub(crate) fn find_extension_arc_by_role(&self, role: &str) -> Option<Arc<Mutex<WasmRuntime>>> {
        let resolved_name = self
            .config
            .lock()
            .extensions
            .iter()
            .find(|e| e.role == role)
            .map(|e| e.name.clone())?;
        self.wasm_runtime.lock().get(&resolved_name).cloned()
    }
}
