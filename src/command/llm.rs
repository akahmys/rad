use crate::config::{Config, LlmEndpointProfile};
use crate::orchestrator::Orchestrator;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmSubcommand {
    List,
    Switch(String),
    Test(Option<String>),
    Add {
        name: String,
        url: String,
        model: Option<String>,
        api_key: Option<String>,
    },
    Model(String),
}

#[must_use]
pub fn parse_llm_command(args: &[&str]) -> LlmSubcommand {
    if args.is_empty() {
        return LlmSubcommand::List;
    }

    match args[0] {
        "list" => LlmSubcommand::List,
        "test" => {
            let target = if args.len() > 1 {
                Some(args[1].to_string())
            } else {
                None
            };
            LlmSubcommand::Test(target)
        }
        "add" => {
            if args.len() >= 3 {
                let name = args[1].to_string();
                let url = args[2].to_string();
                let model = if args.len() > 3 {
                    Some(args[3].to_string())
                } else {
                    None
                };
                let api_key = if args.len() > 4 {
                    Some(args[4].to_string())
                } else {
                    None
                };
                LlmSubcommand::Add {
                    name,
                    url,
                    model,
                    api_key,
                }
            } else {
                LlmSubcommand::List
            }
        }
        "model" => {
            if args.len() > 1 {
                LlmSubcommand::Model(args[1].to_string())
            } else {
                LlmSubcommand::List
            }
        }
        "switch" => {
            if args.len() > 1 {
                LlmSubcommand::Switch(args[1].to_string())
            } else {
                LlmSubcommand::List
            }
        }
        other => LlmSubcommand::Switch(other.to_string()),
    }
}

#[must_use]
pub fn execute_llm_command(subcmd: &LlmSubcommand, orchestrator: &Orchestrator) -> String {
    match subcmd {
        LlmSubcommand::List => render_llm_profiles(&orchestrator.config.lock()),
        LlmSubcommand::Switch(target) => switch_llm_profile(orchestrator, target),
        LlmSubcommand::Test(target) => test_llm_profiles(orchestrator, target.as_deref()),
        LlmSubcommand::Add {
            name,
            url,
            model,
            api_key,
        } => add_llm_profile(orchestrator, name, url, model.as_deref(), api_key.as_deref()),
        LlmSubcommand::Model(new_model) => set_active_model(orchestrator, new_model),
    }
}

#[must_use]
pub fn render_llm_profiles(config: &Config) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Configured LLM Server Endpoints:");
    if config.llm.endpoints.is_empty() {
        let _ = writeln!(out, "  (No LLM endpoints configured in config.json)");
        let _ = writeln!(
            out,
            "  Use `/llm add <name> <url> [model] [api_key]` to register an endpoint."
        );
        return out;
    }

    let mut sorted_keys: Vec<_> = config.llm.endpoints.keys().collect();
    sorted_keys.sort();

    for (idx, name) in sorted_keys.iter().enumerate() {
        let num = idx + 1;
        let profile = &config.llm.endpoints[*name];
        let is_active = config.llm.active.as_deref() == Some(*name);
        let active_mark = if is_active { " \x1b[1;32m(active)\x1b[0m" } else { "" };
        let model_info = profile
            .model
            .as_deref()
            .map_or_else(String::new, |m| format!(" [model: {m}]"));
        let key_info = if profile.api_key.is_some() { " [auth: yes]" } else { "" };

        let _ = writeln!(
            out,
            "  [{num}] {name}{active_mark}: {}{model_info}{key_info}",
            profile.base_url
        );
    }
    let _ = writeln!(
        out,
        "\nType `/llm <name_or_number>` to switch endpoints, or `/llm test` for health checks."
    );
    out
}

fn switch_llm_profile(orchestrator: &Orchestrator, target: &str) -> String {
    let mut cfg = orchestrator.config.lock();
    let matched_name = if let Ok(num) = target.parse::<usize>() {
        let mut sorted_keys: Vec<_> = cfg.llm.endpoints.keys().cloned().collect();
        sorted_keys.sort();
        if num > 0 && num <= sorted_keys.len() {
            Some(sorted_keys[num - 1].clone())
        } else {
            None
        }
    } else if cfg.llm.endpoints.contains_key(target) {
        Some(target.to_string())
    } else {
        None
    };

    let Some(profile_name) = matched_name else {
        return format!("\x1b[1;31mError: LLM profile '{target}' not found.\x1b[0m\nUse `/llm` to list available profiles.");
    };

    cfg.llm.active = Some(profile_name.clone());
    save_global_config(&cfg);
    format!("\x1b[32mSwitched active LLM server profile to '\x1b[1m{profile_name}\x1b[0;32m'.\x1b[0m")
}

fn set_active_model(orchestrator: &Orchestrator, new_model: &str) -> String {
    let mut cfg = orchestrator.config.lock();
    let Some(active_name) = cfg.llm.active.clone() else {
        return "\x1b[1;31mError: No active LLM profile selected to change model.\x1b[0m".to_string();
    };

    if let Some(profile) = cfg.llm.endpoints.get_mut(&active_name) {
        profile.model = Some(new_model.to_string());
        save_global_config(&cfg);
        format!("\x1b[32mUpdated model for profile '{active_name}' to '\x1b[1m{new_model}\x1b[0;32m'.\x1b[0m")
    } else {
        format!("\x1b[1;31mError: Active profile '{active_name}' not found.\x1b[0m")
    }
}

fn add_llm_profile(
    orchestrator: &Orchestrator,
    name: &str,
    url: &str,
    model: Option<&str>,
    api_key: Option<&str>,
) -> String {
    let mut cfg = orchestrator.config.lock();
    let profile = LlmEndpointProfile {
        base_url: url.to_string(),
        api_key: api_key.map(ToString::to_string),
        model: model.map(ToString::to_string),
    };
    cfg.llm.endpoints.insert(name.to_string(), profile);
    if cfg.llm.active.is_none() {
        cfg.llm.active = Some(name.to_string());
    }
    save_global_config(&cfg);
    format!("\x1b[32mAdded LLM profile '\x1b[1m{name}\x1b[0;32m' ({url}) and saved to config.json.\x1b[0m")
}

fn test_llm_profiles(orchestrator: &Orchestrator, target: Option<&str>) -> String {
    let cfg = orchestrator.config.lock();
    if cfg.llm.endpoints.is_empty() {
        return "\x1b[33mNo LLM endpoints configured to test.\x1b[0m".to_string();
    }

    let mut out = String::new();
    let _ = writeln!(out, "Running LLM Server Health Checks...");

    let targets: Vec<(String, LlmEndpointProfile)> = if let Some(t) = target {
        if let Some(p) = cfg.llm.endpoints.get(t) {
            vec![(t.to_string(), p.clone())]
        } else {
            return format!("\x1b[1;31mError: Profile '{t}' not found.\x1b[0m");
        }
    } else {
        let mut list: Vec<_> = cfg
            .llm
            .endpoints
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    };

    for (name, profile) in targets {
        let start = std::time::Instant::now();
        let res = probe_endpoint(&profile.base_url);
        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match res {
            Ok(()) => {
                let _ = writeln!(
                    out,
                    "  - {name} ({}) -> \x1b[32mOK\x1b[0m ({elapsed_ms}ms)",
                    profile.base_url
                );
            }
            Err(e) => {
                let _ = writeln!(
                    out,
                    "  - {name} ({}) -> \x1b[31mFAILED ({e})\x1b[0m",
                    profile.base_url
                );
            }
        }
    }

    out
}

fn probe_endpoint(url_str: &str) -> Result<(), String> {
    let stripped = url_str
        .strip_prefix("http://")
        .or_else(|| url_str.strip_prefix("https://"))
        .unwrap_or(url_str);
    let host_port_path = stripped.split('/').next().unwrap_or(stripped);
    let mut parts = host_port_path.split(':');
    let host = parts.next().unwrap_or("localhost");
    let default_port = if url_str.starts_with("https://") { 443 } else { 80 };
    let port = parts
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(default_port);

    let socket_addr = format!("{host}:{port}");
    let addrs: Vec<std::net::SocketAddr> = std::net::ToSocketAddrs::to_socket_addrs(&socket_addr)
        .map_err(|e| format!("DNS resolution error: {e}"))?
        .collect();

    if addrs.is_empty() {
        return Err("Could not resolve host address".to_string());
    }

    let timeout = std::time::Duration::from_secs(3);
    for addr in addrs {
        if std::net::TcpStream::connect_timeout(&addr, timeout).is_ok() {
            return Ok(());
        }
    }
    Err("Connection refused".to_string())
}

fn save_global_config(config: &Config) {
    if let Some(home_dir) = dirs::home_dir() {
        let config_path = home_dir.join(".rad/config.json");
        if let Ok(json_str) = serde_json::to_string_pretty(config) {
            let _ = std::fs::write(config_path, json_str);
        }
    }
}
