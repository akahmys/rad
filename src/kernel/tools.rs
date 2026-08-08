//! Bridges the host's tool path to kernel modules.
//!
//! The registry maps one method to exactly one module, so tool providers cannot
//! all claim a bare `tools.list` — the second one to register would be a
//! startup error. Each module therefore namespaces its own
//! (`skills.tools.list`, `skills.tools.call`, following the same
//! `<module>.<method>` rule as every other module method), and the host walks
//! the modules to build the aggregate. That inverts where the loop lives
//! relative to `dispatch.call`, which is the point: aggregation is the host's
//! job, and no module needs to know another exists.
//!
//! A module that provides neither method is skipped, so this is a no-op until
//! a tool-providing module is configured.

use crate::kernel::shared::KernelShared;
use serde_json::Value;

/// `<module>.tools.list` — takes no payload, returns `{"tools": [...]}` in the
/// OpenAI function-tool shape the LLM connector already expects.
fn list_method(module: &str) -> String {
    format!("{module}.tools.list")
}

/// `<module>.tools.call` — takes `{"name": ..., "arguments": ...}`, returns
/// `{"content": ...}`.
fn call_method(module: &str) -> String {
    format!("{module}.tools.call")
}

/// Asks one module for its tools. `None` means "does not provide tools", which
/// is not an error; `Some(vec![])` means it provides them and currently has
/// none.
fn tools_of(kernel: &KernelShared, module: &str) -> Option<Vec<Value>> {
    let method = list_method(module);
    // Routed by method, not by target: `resolve` answers `Some` for any live
    // module name, which would say yes for modules that provide no tools.
    if kernel.provider_of(&method).as_deref() != Some(module) {
        return None;
    }
    let reply = match kernel.call("host", module, &method, "{}") {
        Ok(reply) => reply,
        Err(e) => {
            crate::log_host!("[KERNEL] {method} failed: {e}");
            return Some(Vec::new());
        }
    };
    match serde_json::from_str::<Value>(&reply) {
        Ok(Value::Object(mut obj)) => match obj.remove("tools") {
            Some(Value::Array(arr)) => Some(arr),
            _ => {
                crate::log_host!("[KERNEL] {method} returned no 'tools' array");
                Some(Vec::new())
            }
        },
        _ => {
            crate::log_host!("[KERNEL] {method} returned a non-object reply");
            Some(Vec::new())
        }
    }
}

/// Every tool offered by every loaded module, in module-registration order.
#[must_use]
pub fn list(kernel: &KernelShared) -> Vec<Value> {
    let mut all = Vec::new();
    for module in kernel.modules() {
        if let Some(tools) = tools_of(kernel, &module) {
            all.extend(tools);
        }
    }
    all
}

/// The module offering `name`, if any.
fn owner(kernel: &KernelShared, name: &str) -> Option<String> {
    kernel.modules().into_iter().find(|module| {
        tools_of(kernel, module).is_some_and(|tools| {
            tools.iter().any(|t| {
                t.get("function")
                    .and_then(|f| f.get("name"))
                    .and_then(Value::as_str)
                    == Some(name)
            })
        })
    })
}

/// Runs a tool if a module owns it.
///
/// `None` means no module claims the tool, which is the signal to fall through
/// to the extension path — during migration both are live at once, and a
/// missing module must not turn into a failed tool call.
#[must_use]
pub fn execute(
    kernel: &KernelShared,
    name: &str,
    arguments: &str,
) -> Option<Result<String, String>> {
    let module = owner(kernel, name)?;
    let payload = serde_json::json!({ "name": name, "arguments": arguments }).to_string();
    Some(
        kernel
            .call("host", &module, &call_method(&module), &payload)
            .and_then(|reply| {
                serde_json::from_str::<Value>(&reply)
                    .ok()
                    .and_then(|v| {
                        v.get("content")
                            .and_then(Value::as_str)
                            .map(ToString::to_string)
                    })
                    // A module that answers something other than
                    // `{"content": ...}` has its raw reply passed through
                    // rather than swallowed — the model sees what happened.
                    .ok_or(reply)
                    .or_else(Ok::<String, String>)
            }),
    )
}
