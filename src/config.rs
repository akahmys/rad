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
pub struct Config {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub default_timeout: DefaultTimeoutConfig,
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
fn parse_jsonc(content: &str) -> Result<serde_json::Value, String> {
    jsonc_parser::parse_to_serde_value(content, &jsonc_parser::ParseOptions::default())
        .map_err(|e| format!("JSONC parse error: {e:?}"))?
        .ok_or_else(|| "JSONC parsed to empty value".to_string())
}

/// Discovers the config files in order of preference.
fn discover_config_path(explicit_path: Option<&str>) -> Option<PathBuf> {
    if let Some(path_str) = explicit_path {
        let p = PathBuf::from(path_str);
        if p.exists() {
            return Some(p);
        }
        return None;
    }

    // 2. Project Local (Root)
    let p_root = PathBuf::from("rad.json");
    if p_root.exists() {
        return Some(p_root);
    }

    // 3. Project Local (.rad/)
    let p_rad = PathBuf::from(".rad/rad.json");
    if p_rad.exists() {
        return Some(p_rad);
    }

    // 4. User Global
    if let Some(mut config_dir) = dirs::config_dir() {
        config_dir.push("rad/rad.json");
        if config_dir.exists() {
            return Some(config_dir);
        }
    }

    None
}

/// Load configuration by merging base config with local config if it exists.
pub fn load_config(explicit_path: Option<&str>) -> Result<Config, String> {
    let config_path = discover_config_path(explicit_path);

    let base_val = if let Some(path) = config_path {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read base config: {e}"))?;
        let mut base_val = parse_jsonc(&content)?;

        // Try loading rad.local.json in the same directory
        if let Some(parent) = path.parent() {
            let local_path = parent.join("rad.local.json");
            if local_path.exists() {
                let local_content = fs::read_to_string(&local_path)
                    .map_err(|e| format!("Failed to read local config: {e}"))?;
                let local_val = parse_jsonc(&local_content)?;
                merge_json_value(&mut base_val, local_val);
            }
        }
        base_val
    } else {
        // If no config file found, return default Config
        return Ok(Config::default());
    };

    let config: Config = serde_json::from_value(base_val)
        .map_err(|e| format!("Failed to deserialize final config: {e}"))?;
    Ok(config)
}
