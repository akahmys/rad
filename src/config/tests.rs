use super::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_parse_jsonc_valid() {
    let jsonc = r#"
    {
        // core settings
        "core": {
            "workspace_dir": "."
        }
    }
    "#;
    let val = parse_jsonc(jsonc).unwrap();
    assert_eq!(val["core"]["workspace_dir"], ".");
}

#[test]
fn test_parse_jsonc_invalid() {
    let jsonc = r#"
    {
        "core": {
            "workspace_dir": .
        }
    }
    "#;
    assert!(parse_jsonc(jsonc).is_err());
}

#[test]
fn test_merge_json_value() {
    let mut base = serde_json::json!({
        "core": {
            "workspace_dir": ".",
            "log_dir": ".rad/logs"
        },
        "extensions": [
            {
                "name": "ext1",
                "source": "builtin://ext1",
                "enabled": true,
                "config": {
                    "model": "gpt-4"
                }
            }
        ]
    });

    let local = serde_json::json!({
        "core": {
            "log_dir": "/custom/logs"
        },
        "extensions": [
            {
                "name": "ext1",
                "config": {
                    "api_key": "secret"
                }
            },
            {
                "name": "ext2",
                "source": "builtin://ext2",
                "enabled": false
            }
        ]
    });

    merge_json_value(&mut base, local);

    let config: Config = serde_json::from_value(base).unwrap();
    assert_eq!(config.core.workspace, ".");
    assert_eq!(config.core.log, "/custom/logs");
    assert_eq!(config.extensions.len(), 2);

    let ext1 = config.extensions.iter().find(|e| e.name == "ext1").unwrap();
    assert!(ext1.enabled);
    assert_eq!(ext1.config.get("model").unwrap().as_str().unwrap(), "gpt-4");
    assert_eq!(
        ext1.config.get("api_key").unwrap().as_str().unwrap(),
        "secret"
    );

    let ext2 = config.extensions.iter().find(|e| e.name == "ext2").unwrap();
    assert!(!ext2.enabled);
}

#[test]
fn test_load_config_default_when_no_file() {
    // Explicit path that doesn't exist returns default Config
    let config = load_config(Some("non_existent_config_file_xyz.json"));
    assert!(config.is_ok());
    let config = config.unwrap();
    assert_eq!(config.core.workspace, ".");
}

#[test]
fn test_load_config_with_local_override() {
    let test_dir = PathBuf::from("temp_test_config");
    fs::create_dir_all(&test_dir).unwrap();

    let base_path = test_dir.join("rad.json");
    let local_path = test_dir.join("rad.local.json");

    let base_content = r#"
    {
        "core": {
            "workspace_dir": "/base/workspace"
        },
        "extensions": [
            {
                "name": "ext",
                "source": "builtin://ext",
                "enabled": true
            }
        ]
    }
    "#;
    let local_content = r#"
    {
        "core": {
            "snapshot_dir": "/local/snapshots"
        },
        "extensions": [
            {
                "name": "ext",
                "config": {
                    "api_key": "local_key"
                }
            }
        ]
    }
    "#;

    fs::write(&base_path, base_content).unwrap();
    fs::write(&local_path, local_content).unwrap();

    let config_res = load_config(Some(base_path.to_str().unwrap()));

    let _ = fs::remove_file(&base_path);
    let _ = fs::remove_file(&local_path);
    let _ = fs::remove_dir(&test_dir);

    let config = config_res.unwrap();
    assert_eq!(config.core.workspace, "/base/workspace");
    assert_eq!(config.core.snapshot, "/local/snapshots");
    let ext = config.extensions.iter().find(|e| e.name == "ext").unwrap();
    assert_eq!(
        ext.config.get("api_key").unwrap().as_str().unwrap(),
        "local_key"
    );
}

#[test]
fn test_llm_config_deserialization_and_env_resolution() {
    unsafe {
        std::env::set_var("TEST_RAD_LLM_KEY", "secret_123");
    }
    let jsonc = r#"
    {
        "llm": {
            "active": "ollama",
            "endpoints": {
                "ollama": {
                    "base_url": "http://localhost:11434/v1",
                    "model": "llama3"
                },
                "openai": {
                    "base_url": "https://api.openai.com/v1",
                    "api_key": "env:TEST_RAD_LLM_KEY",
                    "model": "gpt-4o"
                }
            }
        }
    }
    "#;

    let val = parse_jsonc(jsonc).unwrap();
    let config: Config = serde_json::from_value(val).unwrap();
    assert_eq!(config.llm.active.as_deref(), Some("ollama"));
    assert_eq!(config.llm.endpoints.len(), 2);

    let openai_profile = config.llm.endpoints.get("openai").unwrap();
    assert_eq!(openai_profile.base_url, "https://api.openai.com/v1");
    assert_eq!(openai_profile.resolved_api_key().as_deref(), Some("secret_123"));

    unsafe {
        std::env::remove_var("TEST_RAD_LLM_KEY");
    }
}
