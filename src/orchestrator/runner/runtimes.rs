// Wasm runtime lifecycle (init/lookup/clear), split out of `runner.rs` to
// stay under the 300-line file limit.
use super::Orchestrator;
use crate::ipc::RasCoreEvent;
use crate::wasm::WasmRuntime;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Sender;

#[cfg(test)]
mod tests;

/// Settings shared by every runtime, read once so the config lock is not
/// retaken per extension.
struct RuntimeSettings {
    hitl_enabled: bool,
    llm_stream_heartbeat_ms: u64,
}

impl RuntimeSettings {
    fn read(config: &crate::config::Config) -> Self {
        Self {
            hitl_enabled: config.core.hitl_enabled,
            llm_stream_heartbeat_ms: config.default_timeout.llm_stream_heartbeat_ms,
        }
    }
}

/// Prints what a freshly loaded tool provider actually offers.
///
/// Reporting only. A provider that answers badly is described and kept — the
/// caller's contract is that one broken provider must not take the others down
/// with it, so nothing here returns a failure.
fn report_tool_inventory(name: &str, runtime: &mut WasmRuntime) {
    let json_str = match runtime.get_tools() {
        Ok(json_str) => json_str,
        Err(e) => {
            println!("\x1b[31m[FAILED] Extension '{name}' get_tools error: {e}\x1b[0m");
            return;
        }
    };

    // A parse failure and a non-array are the same answer to the only question
    // being asked — how many tools are there — so they report identically.
    let count = serde_json::from_str::<serde_json::Value>(&json_str)
        .ok()
        .and_then(|val| val.as_array().map(Vec::len));
    let Some(count) = count else {
        println!(
            "\x1b[31m[FAILED] Extension '{name}' returned invalid JSON from get_tools: {json_str}\x1b[0m"
        );
        return;
    };

    if count > 0 {
        println!("\x1b[32m[OK] Verified {count} tools from extension '{name}'\x1b[0m");
    } else if name == "mcp-tool-provider" {
        println!(
            "\x1b[31m[FAILED] Extension '{name}' initialized with 0 tools. See [MCP Diagnostic] lines above for the actual cause.\x1b[0m"
        );
    } else {
        println!("\x1b[32m[OK] Extension '{name}' initialized with 0 tools\x1b[0m");
    }
}

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
        let (extensions, settings) = {
            let config = self.config.lock();
            (config.extensions.clone(), RuntimeSettings::read(&config))
        };

        for ext in &extensions {
            if !ext.enabled {
                continue;
            }
            if self.wasm_runtime.lock().contains_key(&ext.name) {
                continue;
            }
            let wasm_path = crate::config::expand_tilde(&ext.source);
            // A configured extension whose file is absent is skipped silently,
            // as it always has been: `~/.rad/config.json` outlives any one
            // build, and a missing component is a build that has not run yet.
            if !wasm_path.exists() {
                continue;
            }

            let mut runtime = self.load_runtime(ext, &wasm_path, event_tx, &settings)?;
            if runtime.tool_provider.is_some() {
                report_tool_inventory(&ext.name, &mut runtime);
            }
            self.wasm_runtime
                .lock()
                .insert(ext.name.clone(), Arc::new(Mutex::new(runtime)));
        }

        let guard = self.wasm_runtime.lock();
        Ok(guard.clone())
    }

    /// Instantiates one extension's component with the host subsystems behind
    /// it.
    fn load_runtime(
        self: &Arc<Self>,
        ext: &crate::config::ExtensionConfig,
        wasm_path: &Path,
        event_tx: &Sender<RasCoreEvent>,
        settings: &RuntimeSettings,
    ) -> Result<WasmRuntime, String> {
        let dag_subsystem = Arc::new(crate::dag::DagSubsystemImpl {
            dag: self.dag.clone(),
            kernel: self.kernel.lock().clone(),
        });
        WasmRuntime::new(
            ext.name.clone(),
            wasm_path,
            ext.role.clone(),
            ext.permissions.clone().unwrap_or_default(),
            self.sandbox.clone() as Arc<dyn crate::subsystems::FsSubsystem>,
            self.process_manager.clone() as Arc<dyn crate::subsystems::ProcessSubsystem>,
            dag_subsystem,
            Arc::new(crate::http::HttpManager),
            self.active_processes.clone(),
            event_tx.clone(),
            Some(Arc::downgrade(self)),
            settings.hitl_enabled,
            settings.llm_stream_heartbeat_ms,
        )
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
