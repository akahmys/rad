// Config/session loading and the startup LLM endpoint health check, split
// out of `main.rs` to stay under the 300-line file limit.
use crate::Args;

pub(crate) fn load_config_and_session(
    args: &Args,
) -> Result<
    (
        rad::config::Config,
        String,
        std::sync::Arc<parking_lot::Mutex<rad::dag::Dag>>,
    ),
    String,
> {
    let mut cfg = rad::config::load_config(args.config.as_deref())
        .map_err(|e| format!("Error loading configuration: {e}"))?;

    // Apply CLI overrides (Tier 1 Priority)
    if let Some(ref ws) = args.workspace {
        cfg.core.workspace.clone_from(ws);
    }
    let active_name = cfg
        .llm
        .active
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let profile = cfg.llm.endpoints.entry(active_name).or_default();
    if let Some(ref url) = args.base_url {
        profile.base_url.clone_from(url);
    }
    if let Some(ref key) = args.api_key {
        profile.api_key = Some(key.clone());
    }
    if let Some(ref model) = args.model {
        profile.model = Some(model.clone());
    }

    println!("\x1b[32mConfiguration loaded successfully!\x1b[0m");
    check_active_llm_endpoint(&mut cfg);
    println!("Workspace Dir: {}", cfg.core.workspace);
    println!("Snapshot Dir: {}", cfg.core.snapshot);
    println!("Log Dir: {}", cfg.core.log);
    let enabled_exts: Vec<_> = cfg
        .extensions
        .iter()
        .filter(|ext| ext.enabled && rad::config::expand_tilde(&ext.source).exists())
        .collect();
    println!("Extensions loaded ({}):", enabled_exts.len());
    for ext in &enabled_exts {
        println!("  - {}", ext.name);
    }

    let session_id = args.session.clone().unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs())
            .to_string()
    });

    let dag = if let Ok(loaded) = rad::session::load_session(&cfg.core.workspace, &session_id) {
        println!("\x1b[36mResumed session: {session_id}\x1b[0m");
        loaded
    } else {
        println!("\x1b[36mStarted new session: {session_id}\x1b[0m");
        rad::dag::Dag::new()
    };

    rad::session::prune_sessions(&cfg.core.workspace, cfg.core.max_sessions, &session_id);

    Ok((
        cfg,
        session_id,
        std::sync::Arc::new(parking_lot::Mutex::new(dag)),
    ))
}

/// Probes the active LLM endpoint at startup and prints the result. This is
/// best-effort: a warning here doesn't block the shell from starting (the
/// server may just not be up yet), but it surfaces connectivity/hang issues
/// immediately instead of leaving the user staring at a silent prompt after
/// their first message.
fn check_active_llm_endpoint(cfg: &mut rad::config::Config) {
    use std::io::Write as _;

    let active_name = cfg
        .llm
        .active
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let (base_url, model) = {
        let Some(profile) = cfg.llm.endpoints.get(&active_name) else {
            return;
        };
        if profile.base_url.trim().is_empty() {
            return;
        }
        (profile.base_url.clone(), profile.model.clone())
    };

    print!("Checking LLM endpoint ({base_url})... ");
    let _ = std::io::stdout().flush();

    let start = std::time::Instant::now();
    let status = rad::command::llm::check_endpoint(&base_url, model.as_deref());
    match status.reachable {
        Ok(()) => {
            let elapsed_ms = start.elapsed().as_millis();
            let ctx_info = status
                .context_length
                .map_or_else(String::new, |n| format!(", context window: {n} tokens"));
            println!("\x1b[32mOK\x1b[0m ({elapsed_ms}ms{ctx_info})");
            if let Some(n) = status.context_length
                && let Some(profile) = cfg.llm.endpoints.get_mut(&active_name)
            {
                profile.context_length = Some(n);
            }
        }
        Err(e) => {
            println!(
                "\x1b[33mWARNING\x1b[0m ({e}). The server may be unreachable or unresponsive — \
                 requests could hang until it recovers. Run `/llm test` to re-check."
            );
        }
    }
}
