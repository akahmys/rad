//! The blocklist itself. Ported from `ext/security-guard/src/policy.rs`, with
//! the config arriving from `kernel.config` instead of `GetExtensionConfig`
//! and the `RasRpcCommand` match gone.
//!
//! **Only `block_command_patterns` survives the port.** The extension also
//! carried `block_path_patterns`, matched against `RasRpcCommand::FileWrite`.
//! Nothing reached that branch: `modules/mcp` serves the `write` tool through
//! `bash`, not through `FileWrite`, and `rad-orchestrator` issues no
//! `FileWrite` at all. Measured rather than reasoned — removing the key from
//! all three end-to-end test configs left them green and still blocking, and
//! removing `block_command_patterns` as well made them fail. The one place it
//! still had an effect was a unit test calling `verify_rpc` directly, with no
//! production path behind it.
use std::cell::RefCell;

/// Patterns are substrings, not globs or regexes — the same matching the
/// extension did. An empty or absent list blocks nothing: the policy is
/// opt-in, and a `policy` module with no config is a no-op rather than a
/// lockout.
#[derive(Default, Clone)]
pub(crate) struct Rules {
    block_command_patterns: Vec<String>,
}

thread_local! {
    static RULES: RefCell<Option<Rules>> = const { RefCell::new(None) };
}

impl Rules {
    /// Reads `kernel.config` once per component instance and caches it.
    ///
    /// A kernel that cannot answer yields an empty policy rather than a
    /// refusal to run: this module denying every tool call because its own
    /// config lookup failed would be a far worse failure than blocking
    /// nothing, and `mcp` treats a *call* failure as a denial already.
    fn fetch() -> Self {
        let Ok(raw) = crate::dispatch::call("kernel", "kernel.config", "{}") else {
            return Self::default();
        };
        let Ok(config) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return Self::default();
        };
        Self {
            block_command_patterns: string_array(&config, "block_command_patterns"),
        }
    }

    /// `None` means allow. `Some(reason)` is the pattern that matched, which
    /// travels back to the model as text — a refusal it cannot read is a
    /// refusal it will retry.
    fn refuse(&self, arguments: &str) -> Option<String> {
        self.block_command_patterns
            .iter()
            .find(|p| arguments.contains(p.as_str()))
            .map(|p| format!("blocked by policy pattern '{p}'"))
    }
}

fn string_array(config: &serde_json::Value, key: &str) -> Vec<String> {
    config
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Evaluates one tool call against the cached policy.
///
/// Matches on `arguments` only, exactly as the extension's `ExecuteTool` arm
/// did. The tool `name` is deliberately not matched: doing so would be a new
/// behaviour, and stage 7 is a port.
pub(crate) fn check(arguments: &str) -> Option<String> {
    RULES.with(|cell| {
        let mut slot = cell.borrow_mut();
        slot.get_or_insert_with(Rules::fetch).refuse(arguments)
    })
}

#[cfg(test)]
mod tests;
