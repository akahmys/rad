//! Config-driven blocklist policy: patterns come from this extension's own
//! `ExtensionConfig.config` (`block_path_patterns`/`block_command_patterns`
//! JSON string arrays), fetched via `GetExtensionConfig` and cached for the
//! lifetime of this component instance — replaces the hardcoded
//! `"blocked.txt"`/`"blocked_command"` literals this crate used to carry.
//! Empty/missing config means "block nothing": the policy is opt-in.
use std::cell::RefCell;

use rad_models::RasRpcCommand as CoreRpcCommand;

#[derive(Default, Clone)]
struct Policy {
    block_path_patterns: Vec<String>,
    block_command_patterns: Vec<String>,
}

thread_local! {
    static POLICY: RefCell<Option<Policy>> = const { RefCell::new(None) };
}

impl Policy {
    fn fetch() -> Self {
        let Ok(config) = crate::call_host(CoreRpcCommand::GetExtensionConfig) else {
            return Self::default();
        };
        Self {
            block_path_patterns: string_array(&config, "block_path_patterns"),
            block_command_patterns: string_array(&config, "block_command_patterns"),
        }
    }
}

fn string_array(config: &serde_json::Value, key: &str) -> Vec<String> {
    config
        .get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Evaluates `cmd` against the cached policy, fetching it from the host on
/// first use in this component instance. Returns `false` (deny) if any
/// configured pattern matches; `true` (allow) otherwise, including when no
/// config is present at all.
pub(crate) fn evaluate(cmd: &CoreRpcCommand) -> bool {
    POLICY.with(|cell| {
        let mut slot = cell.borrow_mut();
        let policy = slot.get_or_insert_with(Policy::fetch);
        match cmd {
            CoreRpcCommand::FileWrite { path, .. } => {
                let path_str = path.to_string_lossy();
                !policy.block_path_patterns.iter().any(|p| path_str.contains(p.as_str()))
            }
            CoreRpcCommand::SpawnBashProcess { command } => {
                !policy.block_command_patterns.iter().any(|p| command.contains(p.as_str()))
            }
            CoreRpcCommand::ExecuteTool { arguments, .. } => {
                !policy.block_command_patterns.iter().any(|p| arguments.contains(p.as_str()))
            }
            _ => true,
        }
    })
}
