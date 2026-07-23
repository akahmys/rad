use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

/// Recursively merges `b` into `a`.
/// For the "extensions" key, it performs key-based merging using the "name" field.
fn merge_json_value(a: &mut serde_json::Value, b: serde_json::Value) {
    match (a, b) {
        (serde_json::Value::Object(a_map), serde_json::Value::Object(b_map)) => {
            for (k, v) in b_map {
                if k == "extensions" {
                    if let Some(serde_json::Value::Array(a_exts)) = a_map.get_mut("extensions") {
                        if let serde_json::Value::Array(b_exts) = v {
                            merge_extensions_array(a_exts, b_exts);
                        }
                    } else {
                        a_map.insert(k, v);
                    }
                } else {
                    let entry = a_map.entry(k).or_insert(serde_json::Value::Null);
                    merge_json_value(entry, v);
                }
            }
        }
        (a, b) => {
            *a = b;
        }
    }
}

fn merge_extensions_array(a_exts: &mut Vec<serde_json::Value>, b_exts: Vec<serde_json::Value>) {
    for b_ext in b_exts {
        if let Some(b_name) = b_ext.get("name").and_then(serde_json::Value::as_str) {
            let mut found = false;
            for a_ext in a_exts.iter_mut() {
                if a_ext.get("name").and_then(serde_json::Value::as_str) == Some(b_name) {
                    merge_json_value(a_ext, b_ext.clone());
                    found = true;
                    break;
                }
            }
            if !found {
                a_exts.push(b_ext);
            }
        } else {
            a_exts.push(b_ext);
        }
    }
}

/// Parses a JSONC string into `serde_json::Value`.
fn parse_jsonc(content: &str) -> Result<serde_json::Value, crate::error::UnifiedError> {
    jsonc_parser::parse_to_serde_value(content, &jsonc_parser::ParseOptions::default())
        .map_err(|e| crate::error::UnifiedError::l1(format!("JSONC parse error: {e:?}"), "Config"))?
        .ok_or_else(|| crate::error::UnifiedError::l1("JSONC parsed to empty value", "Config"))
}

impl Config {
    /// Applies environment variable overrides following the precedence hierarchy.
    pub fn apply_env_overrides(&mut self) {
        if let Ok(ws) = std::env::var("RAD_WORKSPACE") {
            self.core.workspace = ws;
        }

        let active_name = self.llm.active.clone().unwrap_or_else(|| "default".to_string());
        let profile = self.llm.endpoints.entry(active_name.clone()).or_default();

        if let Ok(url) = std::env::var("RAD_BASE_URL").or_else(|_| std::env::var("LLM_BASE_URL")) {
            profile.base_url = url;
        }
        if let Ok(key) = std::env::var("RAD_API_KEY").or_else(|_| std::env::var("LLM_API_KEY")) {
            profile.api_key = Some(key);
        }
        if let Ok(model) = std::env::var("RAD_MODEL").or_else(|_| std::env::var("LLM_MODEL")) {
            profile.model = Some(model);
        }
    }
}

/// Discovers the config files in clean order of preference.
fn discover_config_path(explicit_path: Option<&str>) -> Option<PathBuf> {
    if let Some(path_str) = explicit_path {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
        return None;
    }

    // 1. Project Local (Root or .rad/)
    let p_root = PathBuf::from("rad.json");
    if p_root.exists() {
        return Some(p_root);
    }
    let p_rad = PathBuf::from(".rad/config.json");
    if p_rad.exists() {
        return Some(p_rad);
    }

    // 2. User Global (~/.rad/config.json)
    if let Some(home_dir) = dirs::home_dir() {
        let rad_home_config = home_dir.join(".rad/config.json");
        if rad_home_config.exists() {
            return Some(rad_home_config);
        }
    }

    None
}

/// Load configuration by merging base config with local config if it exists and applying env overrides.
pub fn load_config(explicit_path: Option<&str>) -> Result<Config, crate::error::UnifiedError> {
    let config_path = discover_config_path(explicit_path);

    let base_val = if let Some(path) = config_path {
        let content = fs::read_to_string(&path).map_err(|e| {
            crate::error::UnifiedError::l1(format!("Failed to read base config: {e}"), "Config")
        })?;
        let mut base_val = parse_jsonc(&content)?;

        // Try loading rad.local.json or config.local.json in the same directory
        if let Some(parent) = path.parent() {
            let local_path = parent.join("rad.local.json");
            let local_path_alt = parent.join("config.local.json");
            let target_local = if local_path.exists() {
                Some(local_path)
            } else if local_path_alt.exists() {
                Some(local_path_alt)
            } else {
                None
            };

            if let Some(lp) = target_local {
                let local_content = fs::read_to_string(&lp).map_err(|e| {
                    crate::error::UnifiedError::l1(
                        format!("Failed to read local config: {e}"),
                        "Config",
                    )
                })?;
                let local_val = parse_jsonc(&local_content)?;
                merge_json_value(&mut base_val, local_val);
            }
        }
        base_val
    } else {
        // If no config file found, return default Config
        let mut default_cfg = Config::default();
        default_cfg.apply_env_overrides();
        return Ok(default_cfg);
    };

    let mut config: Config = serde_json::from_value(base_val).map_err(|e| {
        crate::error::UnifiedError::l1(format!("Failed to deserialize final config: {e}"), "Config")
    })?;

    config.apply_env_overrides();
    Ok(config)
}
