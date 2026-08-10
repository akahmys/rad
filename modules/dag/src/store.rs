//! Where the conversation lives, and when it reaches disk.
//!
//! The host used to own the graph and `src/session.rs` wrote it out after each
//! completed task. Under stage 9's decision (A) the module owns it, which moves
//! one risk with it: a module that traps has its store discarded and reloaded
//! (§3.6.6), so anything only in memory is gone.
//!
//! **The exposure is bounded, and that is why this saves where it does.**
//! Measured before deciding: the host's copy was refreshed once per completed
//! task, so a crash already cost the turn in progress. Saving on every mutation
//! here is strictly better than that, and cheap — a session file is a few
//! kilobytes and a turn writes a handful of nodes.
use crate::graph::Dag;
use std::cell::RefCell;
use std::path::PathBuf;

thread_local! {
    /// The graph, and the file it belongs to. A module's store is entered by
    /// one caller at a time, so no lock and no unsafe `Send` claim — the shape
    /// `llm-openai`, `mcp` and `agent-loop` all settled on.
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

struct State {
    dag: Dag,
    path: PathBuf,
}

/// `<base>/.rad/sessions/<session_id>.json` — where `src/session.rs` puts it,
/// so a session written by either side is readable by the other. That
/// compatibility is what lets AWU 986 route the host through this module
/// without a migration step.
///
/// **`base` is "." in the module, and it must be.** The kernel preopens the
/// workspace as the guest's `.` (`src/kernel/loader.rs`), so a *host-absolute*
/// workspace path handed in here does not name the same directory: WASI
/// resolves it under the preopen, and `/tmp/ws/.rad/...` becomes
/// `<workspace>/tmp/ws/.rad/...`. That is exactly what the first version did.
/// It went unnoticed because the module's own tests read back through the same
/// mangled path and agreed with themselves; only reading with
/// `rad::session::load_session` showed the file was not where the host looks.
/// The parameter survives for the unit tests below, which run natively where
/// no preopen is in the way.
fn session_path(base: &str, session_id: &str) -> PathBuf {
    PathBuf::from(base)
        .join(".rad")
        .join("sessions")
        .join(format!("{session_id}.json"))
}

/// Attaches to a session, loading it if the file is there.
///
/// A missing file is an empty graph, not an error: that is what starting a new
/// session looks like.
pub(crate) fn open(base: &str, session_id: &str) -> Result<(), String> {
    let path = session_path(base, session_id);
    let dag = match std::fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json)
            .map_err(|e| format!("session file '{}' is unreadable: {e}", path.display()))?,
        Err(_) => Dag::new(),
    };
    STATE.with_borrow_mut(|slot| *slot = Some(State { dag, path }));
    Ok(())
}

/// Runs `f` against the graph and saves if it changed anything.
///
/// Saving is part of the mutation rather than a separate call a caller might
/// forget: the only way to change the graph from outside this file is through
/// here.
pub(crate) fn mutate<T>(f: impl FnOnce(&mut Dag) -> Result<T, String>) -> Result<T, String> {
    STATE.with_borrow_mut(|slot| {
        let state = slot.as_mut().ok_or(NO_SESSION)?;
        let outcome = f(&mut state.dag)?;
        save(state)?;
        Ok(outcome)
    })
}

/// Reads the graph without touching disk.
pub(crate) fn read<T>(f: impl FnOnce(&Dag) -> T) -> Result<T, String> {
    STATE.with_borrow(|slot| {
        slot.as_ref()
            .map(|s| f(&s.dag))
            .ok_or(NO_SESSION.to_string())
    })
}

/// Said in full because it reaches a caller that cannot see this module's
/// state: a `dag.*` call before `dag.open` is a wiring mistake, and "not found"
/// would send someone looking at the graph instead.
const NO_SESSION: &str = "no session is open; call dag.open first";

fn save(state: &State) -> Result<(), String> {
    if let Some(dir) = state.path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("could not create {}: {e}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(&state.dag)
        .map_err(|e| format!("could not serialise the graph: {e}"))?;
    std::fs::write(&state.path, json)
        .map_err(|e| format!("could not write {}: {e}", state.path.display()))
}

#[cfg(test)]
mod tests;
