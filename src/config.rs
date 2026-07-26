use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// JSONC parsing / value-merging and config discovery / layered loading live
// in sibling files to keep this one under the 300-line limit.
mod load;
mod merge;
pub use load::load_config;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreConfig {
    #[serde(rename = "workspace_dir", default = "default_workspace_dir")]
    pub workspace: String,
    #[serde(rename = "snapshot_dir", default = "default_snapshot_dir")]
    pub snapshot: String,
    #[serde(rename = "log_dir", default = "default_log_dir")]
    pub log: String,
    #[serde(default)]
    pub hitl_enabled: bool,
    #[serde(default)]
    pub verification_command: Option<String>,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            workspace: default_workspace_dir(),
            snapshot: default_snapshot_dir(),
            log: default_log_dir(),
            hitl_enabled: false,
            verification_command: None,
        }
    }
}

fn default_workspace_dir() -> String {
    ".".to_string()
}

fn default_snapshot_dir() -> String {
    ".rad/snapshots".to_string()
}

fn default_log_dir() -> String {
    ".rad/logs".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DefaultTimeoutConfig {
    #[serde(default = "default_llm_stream_heartbeat_ms")]
    pub llm_stream_heartbeat_ms: u64,
    #[serde(default = "default_process_silent_timeout_ms")]
    pub process_silent_timeout_ms: u64,
}

impl Default for DefaultTimeoutConfig {
    fn default() -> Self {
        Self {
            llm_stream_heartbeat_ms: default_llm_stream_heartbeat_ms(),
            process_silent_timeout_ms: default_process_silent_timeout_ms(),
        }
    }
}

fn default_llm_stream_heartbeat_ms() -> u64 {
    15000
}

fn default_process_silent_timeout_ms() -> u64 {
    60000
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub allow_bash: bool,
    #[serde(default)]
    pub allow_commands: Vec<String>,
    #[serde(default)]
    pub block_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct NetworkConfig {
    #[serde(default)]
    pub allow_network: bool,
    #[serde(default)]
    pub allow_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PermissionConfig {
    #[serde(default)]
    pub fs_read_allow: Vec<String>,
    #[serde(default)]
    pub fs_write_allow: Vec<String>,
    #[serde(default)]
    pub execution: Option<ExecutionConfig>,
    #[serde(default)]
    pub network: Option<NetworkConfig>,
    #[serde(default)]
    pub allowed_mcp_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExtensionConfig {
    pub name: String,
    #[serde(default)]
    pub source: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_role")]
    pub role: String,
    pub permissions: Option<PermissionConfig>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

fn default_role() -> String {
    "orchestrator".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LlmEndpointProfile {
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

impl LlmEndpointProfile {
    #[must_use]
    pub fn resolved_api_key(&self) -> Option<String> {
        self.api_key.as_ref().and_then(|k| {
            if let Some(var_name) = k.strip_prefix("env:") {
                std::env::var(var_name).ok()
            } else {
                Some(k.clone())
            }
        })
    }
}

#[must_use]
pub fn expand_tilde(path_str: &str) -> std::path::PathBuf {
    if let Some(stripped) = path_str.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(stripped);
    }
    std::path::PathBuf::from(path_str)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LlmConfig {
    #[serde(default)]
    pub active: Option<String>,
    #[serde(default)]
    pub endpoints: HashMap<String, LlmEndpointProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub default_timeout: DefaultTimeoutConfig,
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub extensions: Vec<ExtensionConfig>,
}

