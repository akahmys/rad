use crate::config::Config;
use crate::orchestrator::Orchestrator;
use std::fmt::Write as _;

mod context_length;
mod profile_admin;
pub use context_length::check_endpoint;
use profile_admin::{
    add_llm_profile, delete_llm_profile, set_active_model, set_manual_context_length,
    switch_llm_profile, test_llm_profiles,
};

/// One `/llm` subcommand: name, one-line usage (doubles as help text), and
/// handler. Mirrors the top-level `CommandSpec` registry in
/// `src/command.rs` (AWU 917) so `/llm`'s own subcommands aren't a second,
/// hand-matched parser living one level down — `execute_llm` and
/// `render_llm_profiles`' help footer both read from this same table.
pub struct LlmSubcommandSpec {
    pub name: &'static str,
    pub usage: &'static str,
    pub handler: fn(&str, &Orchestrator) -> String,
}

#[must_use]
pub fn llm_subcommand_specs() -> &'static [LlmSubcommandSpec] {
    &[
        LlmSubcommandSpec {
            name: "list",
            usage: "/llm list",
            handler: |_, orch| render_llm_profiles(&orch.config.lock()),
        },
        LlmSubcommandSpec {
            name: "switch",
            usage: "/llm switch <name|index>",
            handler: |args, orch| switch_llm_profile(orch, args.trim()),
        },
        LlmSubcommandSpec {
            name: "test",
            usage: "/llm test [name]",
            handler: cmd_test,
        },
        LlmSubcommandSpec {
            name: "add",
            usage: "/llm add <name> <url> [--model <model>] [--api-key <key>]",
            handler: cmd_add,
        },
        LlmSubcommandSpec {
            name: "model",
            usage: "/llm model <model>",
            handler: |args, orch| set_active_model(orch, args.trim()),
        },
        LlmSubcommandSpec {
            name: "delete",
            usage: "/llm delete <name>",
            handler: cmd_delete,
        },
        LlmSubcommandSpec {
            name: "context",
            usage: "/llm context <n>   manually set context window (tokens) when auto-detection can't reach it",
            handler: |args, orch| set_manual_context_length(orch, args),
        },
    ]
}

fn cmd_test(args: &str, orchestrator: &Orchestrator) -> String {
    let target = args.trim();
    test_llm_profiles(orchestrator, if target.is_empty() { None } else { Some(target) })
}

/// Flag-based, not positional: `/llm add <name> <url> [--model <m>] [--api-key <k>]`.
/// Replaces the old strict `<name> <url> <model> <api_key>` positional form
/// (which made it impossible to set `api_key` without also supplying
/// `model`) — a deliberate breaking change, acceptable pre-1.0.
fn cmd_add(args: &str, orchestrator: &Orchestrator) -> String {
    const USAGE: &str =
        "\x1b[1;31mUsage: /llm add <name> <url> [--model <model>] [--api-key <key>]\x1b[0m";

    let tokens: Vec<&str> = args.split_whitespace().collect();
    if tokens.len() < 2 {
        return USAGE.to_string();
    }
    let name = tokens[0];
    let url = tokens[1];
    let mut model = None;
    let mut api_key = None;
    let mut i = 2;
    while i < tokens.len() {
        match tokens[i] {
            "--model" if i + 1 < tokens.len() => {
                model = Some(tokens[i + 1].to_string());
                i += 2;
            }
            "--api-key" if i + 1 < tokens.len() => {
                api_key = Some(tokens[i + 1].to_string());
                i += 2;
            }
            other => {
                return format!("\x1b[1;31mUnrecognized argument '{other}'.\x1b[0m\n{USAGE}");
            }
        }
    }
    add_llm_profile(orchestrator, name, url, model.as_deref(), api_key.as_deref())
}

fn cmd_delete(args: &str, orchestrator: &Orchestrator) -> String {
    let name = args.trim();
    if name.is_empty() {
        return "\x1b[1;31mUsage: /llm delete <name>\x1b[0m".to_string();
    }
    delete_llm_profile(orchestrator, name)
}

/// Parses and executes a `/llm` invocation given the raw text that followed
/// `/llm` (e.g. `"add ollama http://localhost:11434"`).
///
/// Registered profile names take priority over reserved subcommand
/// keywords, so a profile literally named `test`/`add`/`model`/etc. can
/// still be switched to by typing its name directly — `/llm switch <name>`
/// remains an unambiguous escape hatch either way.
#[must_use]
pub fn execute_llm(args: &str, orchestrator: &Orchestrator) -> String {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return render_llm_profiles(&orchestrator.config.lock());
    }
    let (word, rest) = trimmed.split_once(char::is_whitespace).unwrap_or((trimmed, ""));

    let is_known_profile = orchestrator.config.lock().llm.endpoints.contains_key(word);
    if is_known_profile || word.parse::<usize>().is_ok() {
        return switch_llm_profile(orchestrator, word);
    }

    match llm_subcommand_specs().iter().find(|spec| spec.name == word) {
        Some(spec) => (spec.handler)(rest.trim(), orchestrator),
        // Not a known subcommand or profile name either — let
        // `switch_llm_profile` produce the "not found" error, rather than
        // silently falling through to being sent to the LLM as a task.
        None => switch_llm_profile(orchestrator, word),
    }
}

#[must_use]
pub fn render_llm_profiles(config: &Config) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Configured LLM Server Endpoints:");
    if config.llm.endpoints.is_empty() {
        let _ = writeln!(out, "  (No LLM endpoints configured in config.json)");
    } else {
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
            let ctx_info = profile
                .context_length
                .map_or_else(String::new, |n| format!(" [ctx: {n} tok]"));

            let _ = writeln!(
                out,
                "  [{num}] {name}{active_mark}: {}{model_info}{key_info}{ctx_info}",
                profile.base_url
            );
        }
    }

    let _ = writeln!(out, "\nSubcommands:");
    for spec in llm_subcommand_specs() {
        let _ = writeln!(out, "  {}", spec.usage);
    }
    let _ = writeln!(out, "  /llm <name_or_index>   switch directly by name or list position");
    out
}
