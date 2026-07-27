//! Mutating `/llm` subcommands (switch/model/add/test), split out of
//! `command/llm.rs` to stay under the 300-line file limit.
use crate::config::{Config, LlmEndpointProfile};
use crate::orchestrator::Orchestrator;
use std::fmt::Write as _;

use super::check_endpoint;

pub(super) fn switch_llm_profile(orchestrator: &Orchestrator, target: &str) -> String {
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

pub(super) fn set_active_model(orchestrator: &Orchestrator, new_model: &str) -> String {
    let (active_name, base_url) = {
        let cfg = orchestrator.config.lock();
        let Some(active_name) = cfg.llm.active.clone() else {
            return "\x1b[1;31mError: No active LLM profile selected to change model.\x1b[0m"
                .to_string();
        };
        let Some(base_url) = cfg.llm.endpoints.get(&active_name).map(|p| p.base_url.clone())
        else {
            return format!("\x1b[1;31mError: Active profile '{active_name}' not found.\x1b[0m");
        };
        (active_name, base_url)
    };

    // The model just changed, so any previously detected context window
    // belonged to the OLD model — carrying it over would silently apply
    // the wrong budget under a false-confidence "known" state. Re-check
    // for the new model, clearing to unknown on failure rather than
    // keeping the stale number.
    let status = check_endpoint(&base_url, Some(new_model));

    let mut cfg = orchestrator.config.lock();
    let Some(profile) = cfg.llm.endpoints.get_mut(&active_name) else {
        return format!("\x1b[1;31mError: Active profile '{active_name}' not found.\x1b[0m");
    };
    profile.model = Some(new_model.to_string());
    profile.context_length = status.context_length;
    save_global_config(&cfg);
    let ctx_info = match (&status.reachable, status.context_length) {
        (Err(e), _) => format!(" (server unreachable: {e})"),
        (Ok(()), Some(n)) => format!(" (context window: {n} tokens)"),
        (Ok(()), None) => " (context window: unknown)".to_string(),
    };
    format!(
        "\x1b[32mUpdated model for profile '{active_name}' to '\x1b[1m{new_model}\x1b[0;32m'{ctx_info}.\x1b[0m"
    )
}

pub(super) fn add_llm_profile(
    orchestrator: &Orchestrator,
    name: &str,
    url: &str,
    model: Option<&str>,
    api_key: Option<&str>,
) -> String {
    let status = check_endpoint(url, model);
    let mut cfg = orchestrator.config.lock();
    // If fresh detection fails, keep whatever this profile already had
    // (manually configured, or detected on a previous `/llm add`/`/llm
    // test`) instead of clobbering it with `None` just because the server
    // was briefly unreachable or is a backend detection doesn't support.
    let context_length = status
        .context_length
        .or_else(|| cfg.llm.endpoints.get(name).and_then(|p| p.context_length));
    let profile = LlmEndpointProfile {
        base_url: url.to_string(),
        api_key: api_key.map(ToString::to_string),
        model: model.map(ToString::to_string),
        context_length,
    };
    cfg.llm.endpoints.insert(name.to_string(), profile);
    if cfg.llm.active.is_none() {
        cfg.llm.active = Some(name.to_string());
    }
    save_global_config(&cfg);
    let ctx_info = match (&status.reachable, context_length) {
        (Err(e), _) => format!(" (server unreachable: {e})"),
        (Ok(()), Some(n)) => format!(" (context window: {n} tokens)"),
        (Ok(()), None) => {
            " (context window: unknown — falling back to a conservative default)".to_string()
        }
    };
    format!(
        "\x1b[32mAdded LLM profile '\x1b[1m{name}\x1b[0;32m' ({url}){ctx_info} and saved to config.json.\x1b[0m"
    )
}

pub(super) fn test_llm_profiles(orchestrator: &Orchestrator, target: Option<&str>) -> String {
    let targets: Vec<(String, LlmEndpointProfile)> = {
        let cfg = orchestrator.config.lock();
        if cfg.llm.endpoints.is_empty() {
            return "\x1b[33mNo LLM endpoints configured to test.\x1b[0m".to_string();
        }
        if let Some(t) = target {
            let Some(p) = cfg.llm.endpoints.get(t) else {
                return format!("\x1b[1;31mError: Profile '{t}' not found.\x1b[0m");
            };
            vec![(t.to_string(), p.clone())]
        } else {
            let mut list: Vec<_> = cfg
                .llm
                .endpoints
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            list.sort_by(|a, b| a.0.cmp(&b.0));
            list
        }
    };

    let mut out = String::new();
    let _ = writeln!(out, "Running LLM Server Health Checks...");

    for (name, profile) in targets {
        let start = std::time::Instant::now();
        // Re-checks context length on every test, not just `/llm add`: the
        // user may have swapped the model on an already-registered
        // endpoint without re-running `add`, and a stale context window
        // number is worse than a fresh re-probe here.
        let status = check_endpoint(&profile.base_url, profile.model.as_deref());
        let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        match status.reachable {
            Ok(()) => {
                let _ = writeln!(
                    out,
                    "  - {name} ({}) -> \x1b[32mOK\x1b[0m ({elapsed_ms}ms)",
                    profile.base_url
                );
                if let Some(n) = status.context_length {
                    let mut cfg = orchestrator.config.lock();
                    if let Some(p) = cfg.llm.endpoints.get_mut(&name) {
                        p.context_length = Some(n);
                    }
                    save_global_config(&cfg);
                    let _ = writeln!(out, "      context window: {n} tokens");
                } else if let Some(n) = profile.context_length {
                    // Detection failed this time, but a value from a
                    // previous probe (or a manual config.json edit) is
                    // still in place and untouched — say so accurately
                    // instead of implying nothing is configured.
                    let _ = writeln!(
                        out,
                        "      context window: {n} tokens (kept previous value — neither /props nor /api/show responded this time)"
                    );
                } else {
                    let _ = writeln!(
                        out,
                        "      context window: unknown (neither /props nor /api/show responded)"
                    );
                }
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

/// Manual `context_length` override for the active profile (Phase 49-1),
/// for backends `check_endpoint`'s auto-detection can't probe (no `/props`
/// or `/api/show` — e.g. a plain OpenAI-compatible proxy). Persists like
/// every other `/llm` mutation, and is overwritten again the next time
/// detection succeeds (`/llm add`/`/llm test`/`/llm model`), since a fresh
/// real reading is always preferred over a manual guess.
pub(super) fn set_manual_context_length(orchestrator: &Orchestrator, arg: &str) -> String {
    let Ok(n) = arg.trim().parse::<u32>() else {
        return "\x1b[1;31mUsage: /llm context <n>\x1b[0m".to_string();
    };
    let mut cfg = orchestrator.config.lock();
    let Some(active_name) = cfg.llm.active.clone() else {
        return "\x1b[1;31mError: No active LLM profile selected.\x1b[0m".to_string();
    };
    let Some(profile) = cfg.llm.endpoints.get_mut(&active_name) else {
        return format!("\x1b[1;31mError: Active profile '{active_name}' not found.\x1b[0m");
    };
    profile.context_length = Some(n);
    save_global_config(&cfg);
    format!(
        "\x1b[32mSet context window for '\x1b[1m{active_name}\x1b[0;32m' to {n} tokens (manual override).\x1b[0m"
    )
}

pub(super) fn delete_llm_profile(orchestrator: &Orchestrator, name: &str) -> String {
    let mut cfg = orchestrator.config.lock();
    if cfg.llm.endpoints.remove(name).is_none() {
        return format!("\x1b[1;31mError: LLM profile '{name}' not found.\x1b[0m");
    }
    if cfg.llm.active.as_deref() == Some(name) {
        // Fall back to another remaining profile (arbitrary but
        // deterministic-per-run choice), or `None` if none are left.
        cfg.llm.active = cfg.llm.endpoints.keys().next().cloned();
    }
    save_global_config(&cfg);
    format!("\x1b[32mRemoved LLM profile '{name}'.\x1b[0m")
}

fn save_global_config(config: &Config) {
    // `crate::config::global_config_path` is the single shared resolver for
    // this path (honors `RAD_TEST_CONFIG_HOME`) — `load_config`'s read side
    // uses the same one, so tests and production agree on where this file
    // lives. See that function's docs / AWU 918.
    let Some(config_path) = crate::config::global_config_path() else {
        return;
    };
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json_str) = serde_json::to_string_pretty(config) {
        let _ = std::fs::write(config_path, json_str);
    }
}
