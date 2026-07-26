// MCP server config discovery/parsing, split out of `client.rs` to stay
// under the 300-line file limit.
use crate::radcomp::extension::types as wit;
use std::collections::HashMap;

#[derive(serde::Deserialize)]
pub struct McpProviderConfig {
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
}

#[derive(serde::Deserialize, Clone)]
pub struct McpServerConfig {
    pub command: String,
    pub args: Vec<String>,
}

fn strip_json_comments(json_str: &str) -> String {
    let mut result = String::new();
    let mut in_string = false;
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut chars = json_str.chars().peekable();

    while let Some(c) = chars.next() {
        if in_single_line_comment {
            if c == '\n' {
                in_single_line_comment = false;
                result.push(c);
            }
            continue;
        }
        if in_multi_line_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_multi_line_comment = false;
            }
            continue;
        }

        if in_string {
            result.push(c);
            if c == '\\' {
                if let Some(&next_c) = chars.peek() {
                    result.push(next_c);
                    chars.next();
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            result.push(c);
            continue;
        }

        if c == '/' {
            if chars.peek() == Some(&'/') {
                chars.next();
                in_single_line_comment = true;
                continue;
            }
            if chars.peek() == Some(&'*') {
                chars.next();
                in_multi_line_comment = true;
                continue;
            }
        }

        result.push(c);
    }

    result
}

fn read_config_file(path: &str) -> Option<String> {
    let cmd = wit::RasRpcCommand::FileRead(path.to_string());
    if let Ok(res_str) = crate::host_rpc(&cmd) {
        if !res_str.is_empty() && res_str != "null" {
            // 1. Try deserializing directly as String (if host returned JSON string)
            if let Ok(s) = serde_json::from_str::<String>(&res_str) {
                let cleaned = strip_json_comments(&s);
                if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                    return Some(cleaned);
                }
            }
            // 2. Try deserializing as byte array (serde_bytes::Bytes)
            if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&res_str) {
                if let Ok(s) = String::from_utf8(bytes) {
                    let cleaned = strip_json_comments(&s);
                    if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                        return Some(cleaned);
                    }
                }
            }
            // 3. Try deserializing as generic Value
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&res_str) {
                if let Some(s) = val.as_str() {
                    let cleaned = strip_json_comments(s);
                    if serde_json::from_str::<serde_json::Value>(&cleaned).is_ok() {
                        return Some(cleaned);
                    }
                }
                if val.is_object() {
                    let cleaned = strip_json_comments(&res_str);
                    return Some(cleaned);
                }
            }
        }
    }
    None
}

pub fn load_mcp_config() -> Option<McpProviderConfig> {
    if std::env::var("RAD_TEST_PORT").is_ok() {
        return Some(McpProviderConfig {
            mcp_servers: Some(HashMap::new()),
        });
    }

    let mut merged_servers: HashMap<String, McpServerConfig> = HashMap::new();
    let mut found_any = false;

    let mut paths = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        paths.push(format!("{home}/.rad/config.json"));
    }
    paths.push("~/.rad/config.json".to_string());
    paths.push(".rad/rad.json".to_string());
    paths.push("config.json".to_string());
    paths.push(".rad/config.json".to_string());
    paths.push("rad.json".to_string());

    for p in &paths {
        if let Some(c) = read_config_file(p) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                if let Some(exts) = v.get("extensions").and_then(|e| e.as_array()) {
                    for ext in exts {
                        if ext.get("name").and_then(|n| n.as_str()) == Some("mcp-tool-provider") {
                            if let Some(cfg_val) = ext.get("config") {
                                if let Ok(parsed_cfg) = serde_json::from_value::<McpProviderConfig>(cfg_val.clone()) {
                                    if let Some(servers) = parsed_cfg.mcp_servers {
                                        for (srv_name, srv_cfg) in servers {
                                            merged_servers.insert(srv_name, srv_cfg);
                                            found_any = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if found_any {
        Some(McpProviderConfig {
            mcp_servers: Some(merged_servers),
        })
    } else {
        None
    }
}
