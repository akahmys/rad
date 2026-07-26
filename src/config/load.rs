// Config file discovery, layered loading/merging, and env-var overrides,
// split out of `config.rs` to stay under the 300-line file limit.
use super::Config;
use super::merge::{merge_json_value, parse_jsonc};
use std::fs;
use std::path::PathBuf;

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
    let mut base_val = serde_json::Value::Object(serde_json::Map::new());

    // 1. Always load User Global (~/.rad/config.json) as the base if it exists
    if let Some(home_dir) = dirs::home_dir() {
        let rad_home_config = home_dir.join(".rad/config.json");
        if rad_home_config.exists()
            && let Ok(content) = fs::read_to_string(&rad_home_config)
            && let Ok(global_val) = parse_jsonc(&content)
        {
            merge_json_value(&mut base_val, global_val);
        }
    }

    // 2. Discover and merge Project Local / Explicit config over the global base
    let config_path = discover_config_path(explicit_path);
    if let Some(path) = config_path {
        // If config_path is the same as global config, skip duplicate read
        let is_global = dirs::home_dir()
            .map(|h| h.join(".rad/config.json") == path)
            .unwrap_or(false);

        if !is_global {
            let content = fs::read_to_string(&path).map_err(|e| {
                crate::error::UnifiedError::l1(format!("Failed to read project config: {e}"), "Config")
            })?;
            let local_project_val = parse_jsonc(&content)?;
            merge_json_value(&mut base_val, local_project_val);
        }

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
    }

    let mut config: Config = serde_json::from_value(base_val).map_err(|e| {
        crate::error::UnifiedError::l1(format!("Failed to deserialize final config: {e}"), "Config")
    })?;

    config.apply_env_overrides();
    Ok(config)
}
