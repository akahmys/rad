//! `/compact`: manually triggers the same compaction policy that already
//! runs automatically every turn (`context-tools.optimize`, called by
//! `rad-orchestrator`), but *persists* the result into the DAG via
//! `merge_nodes` instead of applying it ephemerally to a single outgoing
//! request. Implemented entirely host-side (no new WIT surface): the host
//! already has native access to both the DAG and the registered
//! `WasmRuntime`s, so it can call `context-tools` the same way
//! `rad-orchestrator` does (`call_extension_method`) without needing
//! `rad-orchestrator` to expose a new callable entry point of its own —
//! its `on-event`-only world has no generic "invoke me now" surface (see
//! `wit/rad.wit`'s `rad-orchestrator` world).
use crate::orchestrator::Orchestrator;
use std::collections::HashSet;
use std::sync::Arc;

/// Matches rad-orchestrator's own fallback default
/// (`crate::orchestrator::STATE`'s `max_history_messages`) — the host has
/// no channel to read that WASM-extension-internal state, so this is a
/// deliberate, documented duplication of the one constant rather than new
/// plumbing to share it.
const DEFAULT_MAX_HISTORY: u32 = 30;

#[derive(serde::Serialize)]
struct CompactMessage {
    node_id: Option<String>,
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct CompactRequest {
    messages: Vec<CompactMessage>,
    max_history: Option<u32>,
    max_content_chars: Option<u32>,
}

#[derive(serde::Deserialize)]
struct CompactMessageResp {
    node_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct CompactResponse {
    optimized_messages: Vec<CompactMessageResp>,
    summary: String,
}

pub fn run_compact(orchestrator: &Arc<Orchestrator>) -> String {
    let (tx, _rx) = std::sync::mpsc::channel();
    let _ = orchestrator.get_or_init_runtimes(&tx);

    let candidates = collect_candidates(orchestrator);
    if candidates.len() < 2 {
        return "\x1b[33mNothing to compact yet.\x1b[0m".to_string();
    }

    let Some(runtime_arc) = orchestrator.find_extension_arc_by_role("context-tools") else {
        return "\x1b[1;31mNo extension registered for role \"context-tools\" — nothing to compact against.\x1b[0m".to_string();
    };

    let max_content_chars =
        active_llm_context_length(orchestrator).map(rad_models::budget_chars_from_context_length);
    let req = CompactRequest {
        messages: candidates
            .iter()
            .map(|(id, role, content)| CompactMessage {
                node_id: Some(id.clone()),
                role: role.clone(),
                content: content.clone(),
            })
            .collect(),
        max_history: Some(DEFAULT_MAX_HISTORY),
        max_content_chars,
    };

    let Ok(req_json) = serde_json::to_string(&req) else {
        return "\x1b[1;31mFailed to build compaction request.\x1b[0m".to_string();
    };

    let call_result = {
        let Some(mut runtime) = runtime_arc.try_lock() else {
            return "\x1b[1;31mcontext-tools is busy handling another call — try again shortly.\x1b[0m"
                .to_string();
        };
        runtime.call_extension_method("optimize", &req_json)
    };

    let res_str = match call_result {
        Ok(s) => s,
        Err(e) => return format!("\x1b[1;31mCompaction failed: {e}\x1b[0m"),
    };
    let Ok(resp) = serde_json::from_str::<CompactResponse>(&res_str) else {
        return "\x1b[1;31mCompaction failed: could not parse context-tools' response.\x1b[0m"
            .to_string();
    };

    let surviving_ids: HashSet<String> = resp
        .optimized_messages
        .into_iter()
        .filter_map(|m| m.node_id)
        .collect();

    persist_compaction(orchestrator, &candidates, &surviving_ids, &resp.summary)
}

/// Walks the current DAG chain from `current_node_id` back to the root
/// (mirroring `rad-orchestrator`'s own `traverse_dag_messages`, but
/// host-native and only as detailed as `context-tools.optimize` actually
/// needs — no `tool_calls` parsing, no orphan filtering, since this step
/// only decides which DAG nodes to merge, not what to send an LLM).
/// `"merge"`-typed nodes (from a previous compaction) are treated as role
/// `"system"` so a summary node still counts as real conversation content
/// instead of silently disappearing from future compaction passes.
fn collect_candidates(orchestrator: &Arc<Orchestrator>) -> Vec<(String, String, String)> {
    let dag = orchestrator.dag.lock();
    let mut candidates = Vec::new();
    let mut current_id = dag.current_node_id.clone();

    while let Some(id) = current_id {
        let Some(node) = dag.nodes.get(&id) else {
            break;
        };
        let role = match node.node_type.as_str() {
            "user" | "assistant" | "tool" | "system" => Some(node.node_type.clone()),
            "merge" => Some("system".to_string()),
            _ => None,
        };
        if let Some(role) = role
            && !node.text.is_empty()
        {
            candidates.push((id.clone(), role, node.text.clone()));
        }
        current_id = node.parent_ids.first().cloned();
    }

    candidates.reverse();
    candidates
}

/// Asks the active LLM profile's detected context window directly from
/// config (host-native equivalent of `rpc_meta::active_llm_profile_json`
/// — no RPC needed, the host already holds the config it would otherwise
/// be asking itself for).
fn active_llm_context_length(orchestrator: &Arc<Orchestrator>) -> Option<u32> {
    let cfg = orchestrator.config.lock();
    cfg.llm
        .active
        .as_deref()
        .and_then(|name| cfg.llm.endpoints.get(name))
        .and_then(|p| p.context_length)
}

/// Splits the dropped (non-surviving) candidates into contiguous runs
/// (by position in the chronological `candidates` list) and merges each
/// run independently via `merge_nodes`. Relevance retention (Phase 51-2)
/// can make the dropped set non-contiguous — a single earlier turn might
/// survive between two dropped stretches — so each contiguous run is
/// merged on its own rather than passed to `merge_nodes` as one
/// non-contiguous list, which `merge_nodes` isn't designed to collapse
/// correctly. Runs of a single node are left untouched (not worth
/// "merging" one node into itself).
fn persist_compaction(
    orchestrator: &Arc<Orchestrator>,
    candidates: &[(String, String, String)],
    surviving_ids: &HashSet<String>,
    summary: &str,
) -> String {
    let mut runs: Vec<Vec<String>> = Vec::new();
    let mut current_run: Vec<String> = Vec::new();
    for (id, _, _) in candidates {
        if surviving_ids.contains(id) {
            if !current_run.is_empty() {
                runs.push(std::mem::take(&mut current_run));
            }
        } else {
            current_run.push(id.clone());
        }
    }
    if !current_run.is_empty() {
        runs.push(current_run);
    }

    let mergeable_runs: Vec<Vec<String>> = runs.into_iter().filter(|r| r.len() >= 2).collect();
    if mergeable_runs.is_empty() {
        return format!("\x1b[33mNothing worth compacting: {summary}\x1b[0m");
    }

    let summary_text = format!("[Compacted summary] {summary}");
    let mut merged_count = 0usize;
    let mut dag = orchestrator.dag.lock();
    for run in &mergeable_runs {
        match dag.merge_nodes(run, &summary_text) {
            Ok(_) => merged_count += run.len(),
            Err(e) => return format!("\x1b[1;31mFailed to persist compaction: {e}\x1b[0m"),
        }
    }

    format!(
        "\x1b[32mCompacted {merged_count} message(s) into {} summary node(s). {summary}\x1b[0m",
        mergeable_runs.len()
    )
}

#[cfg(test)]
mod tests;
