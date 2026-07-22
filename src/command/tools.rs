use crate::orchestrator::Orchestrator;
use std::fmt::Write as _;

/// Renders the configured permissions and queries the active tools.
#[must_use]
pub fn render_tools_and_permissions(orchestrator: &Orchestrator) -> String {
    let mut output = String::new();
    let config = orchestrator.config.lock();
    output.push_str(&render_config_permissions(&config));
    output.push('\n');
    output.push_str(&render_wasm_tools(&orchestrator.wasm_runtime));
    output
}

fn render_config_permissions(config: &crate::config::Config) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "Active Permissions:");
    for ext in &config.extensions {
        if !ext.enabled {
            continue;
        }
        let _ = writeln!(output, "  Extension: {} (Role: {})", ext.name, ext.role);
        if let Some(ref perm) = ext.permissions {
            let _ = writeln!(output, "    Read Allowed Paths: {:?}", perm.fs_read_allow);
            let _ = writeln!(output, "    Write Allowed Paths: {:?}", perm.fs_write_allow);
            render_sub_permissions(&mut output, perm);
        } else {
            let _ = writeln!(output, "    Permissions: Default");
        }
    }
    output
}

fn render_sub_permissions(output: &mut String, perm: &crate::config::PermissionConfig) {
    if let Some(ref exec) = perm.execution {
        let _ = writeln!(
            output,
            "    Execution: Bash allowed: {}, Allowed commands: {:?}, Blocked: {:?}",
            exec.allow_bash, exec.allow_commands, exec.block_commands
        );
    }
    if let Some(ref net) = perm.network {
        let _ = writeln!(
            output,
            "    Network: Allowed: {}, Domains: {:?}",
            net.allow_network, net.allow_domains
        );
    }
    let _ = writeln!(
        output,
        "    Allowed MCP Servers: {:?}",
        perm.allowed_mcp_servers
    );
}

fn render_wasm_tools(
    wasm_runtime: &parking_lot::Mutex<
        std::collections::HashMap<
            String,
            std::sync::Arc<parking_lot::Mutex<crate::wasm::WasmRuntime>>,
        >,
    >,
) -> String {
    let mut output = String::new();
    let _ = writeln!(output, "Available Tools (from Wasm tool providers):");
    let runtimes = wasm_runtime.lock();
    let mut found_provider = false;
    for (name, runtime_arc) in runtimes.iter() {
        let mut runtime = runtime_arc.lock();
        if runtime.tool_provider.is_some() {
            found_provider = true;
            let _ = writeln!(output, "  Extension [{name}]:");
            match runtime.get_tools() {
                Ok(tools_json) => {
                    render_tools_list(&mut output, &tools_json);
                }
                Err(e) => {
                    let _ = writeln!(output, "    Error fetching tools: {e}");
                }
            }
        }
    }

    if !found_provider {
        let _ = writeln!(output, "  No active tool provider extension found.");
    }
    output
}

fn render_tools_list(output: &mut String, tools_json: &str) {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(tools_json) {
        if let Some(tools_list) = val.as_array() {
            for tool in tools_list {
                let name = tool
                    .get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| tool.get("name").and_then(serde_json::Value::as_str))
                    .unwrap_or("unknown");
                let desc = tool
                    .get("function")
                    .and_then(|f| f.get("description"))
                    .and_then(serde_json::Value::as_str)
                    .or_else(|| tool.get("description").and_then(serde_json::Value::as_str))
                    .unwrap_or("");
                let _ = writeln!(output, "  - {name}: {desc}");
            }
        } else {
            let _ = writeln!(output, "  {tools_json}");
        }
    } else {
        let _ = writeln!(output, "  {tools_json}");
    }
}
