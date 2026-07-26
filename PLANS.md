# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-26

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 10: Codebase Refactoring & Rule Alignment (v0.15.0)
- [✅] Phase 11: Unified Error Handling Mechanism (v0.16.0)
- [✅] Phase 12: Codebase Verification & Integrity Audit (v0.17.0)
- [✅] Phase 14: Esc Key Task Abort Polish & Robustness (v0.19.0)
- [✅] Phase 15: Release Build, Local Installation & Push (v0.20.0)
- [✅] Phase 16: Separate Core and MCP Tool Providers (v0.21.0)
- [✅] Phase 17: Release Build, Local Installation & Push to Main (v0.22.0)
- [✅] Phase 18: Add Timeouts to Web Access and Fix Hangs (v0.23.0)
- [✅] Phase 19: Remove Built-in Core Tools and Web Access Extensions from rad (v0.24.0)
- [✅] Phase 20: Standardize Global Configuration Directory to ~/.rad/config.json (v0.25.0)
- [✅] Phase 21: Release Build, Local Installation, ~/.rad/config.json Creation & Push to GitHub (v0.26.0)
- [✅] Phase 22: Fix LLM Connection Hangs & Add Graceful Error Handling (v0.27.0)
- [✅] Phase 23: Configurable LLM Endpoints & /llm Slash Command Management (v0.28.0)
- [✅] Phase 24: Model-Agnostic Architecture Refactoring (v0.29.0)
- [✅] Phase 25: Unified Event Tracing & Distributed Logging (v0.30.0)
- [✅] Phase 26: WASM I/O & Streaming Performance Optimization (v0.31.0)
- [✅] Phase 27: One-Command Build & Deployment Automation (v0.32.0)
- [✅] Phase 28: Documentation Update, Config Deployment & Git Main Release (v0.33.0)
- [✅] Phase 29: MCP Host Tilde Expansion & Instant Tool Result Visibility (v0.34.0)
- [✅] Phase 30: Complete Removal of Built-in Shell Fallbacks & Accurate MCP Startup Verification Output (v0.35.0)
- [✅] Phase 31: Stdio Pipe Fallback for PTY Permission Errors in Process Manager (v0.36.0)
- [✅] Phase 32: Fix Tilde Expansion in WASM Permissions & Align MCP Config Discovery (v0.37.0)
- [✅] Phase 33: Fix `read_config_file` Error String Swallowing in WASM MCP Provider (v0.38.0)
- [✅] Phase 34: Fix Tilde Expansion in Host Fs Subsystem (`src/fs.rs`) (v0.39.0)
- [✅] Phase 35: Unified Path Resolution Architecture & Gateway Normalization (v0.40.0)
- [✅] Phase 36: Standardize Stdio Piping for MCP Process Spawning & Fix PTY JSON-RPC Corruptions (v0.41.0)
- [✅] Phase 37: Fast Direct Binary Execution & Robust Read Loop for MCP Server Spawning (v0.42.0)
- [✅] Phase 38: Deep End-to-End Codebase Audit & Verification of MCP Subsystem (v0.43.0)
- [✅] Phase 39: Consolidate Context Compaction Logic into `context-tools` Extension (v0.44.0)
- [✅] Phase 40: Post-Consolidation Audit Fixes — Orphan-Filter Ordering & CallExtension Deadlock Risk (v0.45.0)
- [✅] Phase 41: Second Audit Pass — Unsafe Tool-Call Squashing, File-Size Limit, Production Unwrap (v0.46.0)
- [✅] Phase 42: Full File-Size Limit Compliance — Split All Remaining 300+ Line Files (v0.47.0)
- [✅] Phase 43: Extension Crates Clippy Audit & Hand-Written Allow Cleanup (v0.48.0)
- [✅] Phase 44: Release Build, Verification Audit & Push to Main (v0.49.0)

---

## 🛠️ Short-Term Plan: Phase 44

### 💡 Current AWU Status
- [✅] AWU 913: Commit and Push Phase 43 Extension Crates Cleanups to Main (Result: Success)

### 📝 AWU Details

#### AWU 913: Commit and Push Phase 43 Extension Crates Cleanups to Main
- **Trigger**: User requested pushing completed Phase 43 changes to `main`.
- **Scope**: All modified workspace files in `ext/` and `PLANS.md`.
- **Definition of Done (DoD)**: All changes committed cleanly with descriptive commit message and pushed to `origin/main`.
- **Result**: Success. Committed (`4963118`) and pushed to `origin/main` on 2026-07-26. All pre-commit secret/path scanning checks passed.

---

## 🛠️ Short-Term Plan: Phase 43

### 💡 Current AWU Status
- [✅] AWU 912: Remove Hand-Written Clippy Allows in Extension Crates & Fix Warnings (Result: Success)

### 📝 AWU Details

#### AWU 912: Remove Hand-Written Clippy Allows in Extension Crates & Fix Warnings
- **Trigger**: User requested completely eliminating crate-level `#![allow(...)]` attributes by scoping necessary macro-generated allowances strictly to generated modules.
- **Changes**:
  1. **Zero Crate-Level Allow Directives**: Removed all crate-wide `#![allow(...)]` attributes from `lib.rs` across all 5 extension crates (`ext/context-tools`, `ext/llm-connector`, `ext/mcp-tool-provider`, `ext/rad-orchestrator`, `ext/security-guard`), leaving only `#![deny(clippy::pedantic)]` at crate root.
  2. **Scoped Macro Encapsulation**: Encapsulated `wit_bindgen::generate!` and `export!(...)` inside a dedicated `mod bindings` module annotated with `#[allow(unsafe_op_in_unsafe_fn, clippy::same_length_and_capacity, clippy::pedantic)]`, re-exporting via `pub use bindings::*;`. This ensures `wit_bindgen` generated code warnings are isolated to the generated module alone, while all hand-written code is strictly checked under `#![deny(clippy::pedantic)]`.
  3. **Code Fixes**: Resolved all hand-written Clippy warnings across extension crates (collapsible_if, uninlined_format_args, manual_strip, cast_possible_truncation, too_many_lines, single_match_else, manual_assert, collapsible_match).
- **Scope**: `ext/llm-connector/src/{lib.rs, connector.rs, event_stream.rs}`, `ext/mcp-tool-provider/src/{lib.rs, client.rs, conv.rs, mcp_config.rs, mcp_transport.rs}`, `ext/rad-orchestrator/src/{lib.rs, conv/rpc.rs, llm.rs, orchestrator.rs, tool.rs, orchestrator/reasoning.rs, orchestrator/runner/done.rs}`, `ext/security-guard/src/lib.rs`, `ext/context-tools/src/lib.rs`.
- **Definition of Done (DoD)**: Zero `#![allow(...)]` attributes at crate root; generated code warnings scoped strictly to `mod bindings`; all extension crates build cleanly under `wasm32-wasip1` with zero Clippy warnings; all workspace unit/integration tests and `./scripts/build_all.sh` pass cleanly.
- **Result**: Success. Verified via `cargo clippy --target wasm32-wasip1 -p <ext>` for all 5 extension crates, `cargo clippy --workspace --all-targets` (0 warnings), and full `./scripts/build_all.sh` run.

---

## 🛠️ Short-Term Plan: Phase 42

### 💡 Current AWU Status
- [✅] AWU 911: Split All Remaining CODING.md 300-Line Violations (Result: Success)

### 📝 AWU Details

#### AWU 911: Split All Remaining CODING.md 300-Line Violations
- **Trigger**: User asked for a recap of remaining known issues after AWU 910 shipped; AWU 910 had only fixed one of several files already over CODING.md's 300-line limit (`ext/rad-orchestrator/src/orchestrator.rs`, folded into that AWU because it was directly touched by the squashing fix). Nine more files were still over the limit. User then explicitly asked to address the 300-line limit ("300行制限に対応").
- **Files split** (each into 2–5 sibling files by concern, with the original either becoming a thin dispatcher or keeping the "core"/most-referenced piece and re-exporting where call sites needed an unchanged path):
  1. `src/wasm/imports_rpc.rs` (695→181 lines) → `imports_process.rs`, `imports_tool.rs`, `imports_http.rs`, `imports_delegate.rs` (process/tool/http import handlers + the trait-delegation macro & context-tools host bridge).
  2. `src/wasm/rpc_meta.rs` (603→166 lines) → `rpc_meta_fallback.rs` (built-in tool fallback), `rpc_meta_llm_connector.rs` (Wasm-connector LLM stream path), `rpc_meta_llm_fallback.rs` (raw-HTTP-SSE LLM stream path).
  3. `ext/mcp-tool-provider/src/client.rs` (387→132 lines) → `mcp_config.rs` (config discovery/parsing), `mcp_transport.rs` (JSON-RPC line transport; re-exports `mcp_request` so `lib.rs`'s import path is unchanged).
  4. `src/wasm/bindings.rs` (381→142 lines) → `bindings/rpc_command.rs` (the two large `RasRpcCommand`↔WIT `From` impls).
  5. `src/orchestrator/runner.rs` (366→202 lines) → `runner/runtimes.rs` (Wasm runtime init/lookup/clear), `runner/events.rs` (RPC verification fan-out + event-dispatch loop); both as sibling inherent-`impl Orchestrator` blocks.
  6. `src/wasm/imports_resources.rs` (351→114 lines) → `imports_resources_file.rs` (`file-handle` WIT impl), `imports_resources_exec.rs` (`execution-handle` WIT impl + llm-connector stream glue); shared `push_closed_fallback` helper made `pub(crate)`.
  7. `src/config.rs` (346→181 lines) → `config/merge.rs` (JSONC parse + recursive value merge), `config/load.rs` (config discovery/layered loading, `Config::apply_env_overrides`); `load_config` re-exported so `crate::config::load_config` call sites are unchanged.
  8. `ext/llm-connector/src/lib.rs` (337→28 lines) → `serialize_types.rs` (wire structs), `event_stream.rs` (SSE parsing + `GuestEventStream` impl), `connector.rs` (`ConnectorImpl`/`Guest` impl).
  9. `src/process.rs` (315→158 lines) → `process_child.rs` (`StdioChild`/`ChildKiller` glue + reader-thread helper), `process_running.rs` (`RunningProcess`, re-exported so `crate::process::RunningProcess` is unchanged).
  10. `ext/rad-orchestrator/src/orchestrator/runner.rs` (315→59 lines) → `runner/done.rs` (`handle_done`, the Done-event finalizer), `runner/inline_tool_calls.rs` (plain-text tool-call fallback parser); `handle_done` re-exported so `orchestrator.rs`'s import is unchanged.
- **Bugs hit and fixed during the split** (all caught by the user's `build_all.sh` runs, none reached `main`):
  - `wit_bindgen::generate!`'s `export!` macro only accepts a plain identifier, not a path — `export!(connector::ConnectorImpl)` failed to compile; fixed by `use connector::ConnectorImpl;` then `export!(ConnectorImpl);`.
  - `src/process/tests.rs` used `thread::sleep` via `use super::*`, relying on `process.rs`'s top-level `use std::thread;` which moved to `process_running.rs` in the split; added an explicit `use std::thread;` to the test file.
  - `src/config.rs`'s re-export of `merge_json_value`/`parse_jsonc` (added only so `config/tests.rs`'s `use super::*` could reach them) was flagged as an unused import in non-test builds, since nothing outside `config/tests.rs` used the re-exported path; removed the re-export and had the test file import directly from `super::merge::{..}` instead.
  - Clippy's `empty_line_after_doc_comments` (denied via `-D warnings`) fired on `config/merge.rs`, whose file-header note used `///` (a doc comment) followed by a blank line before the next doc comment block. The same header-comment pattern (`/// ... split out of ... 300-line limit.`) existed verbatim at the top of all 20 new/touched files from this AWU; rather than fix one and wait for the next to surface on a future build, converted all 20 headers from `///` to plain `//` in one pass, since they document the file's split-provenance for humans, not a specific downstream item for rustdoc.
- **Scope**: 10 original files edited (trimmed to dispatchers or "core" pieces), 22 new files created, plus `src/wasm.rs`, `src/lib.rs`, `ext/mcp-tool-provider/src/lib.rs`, `ext/llm-connector/src/lib.rs` updated with the corresponding `mod`/re-export declarations, and `src/config/tests.rs`/`src/process/tests.rs` fixed for the import fallout above.
- **Definition of Done (DoD)**: Every `.rs` file in the workspace (including test files) at or under 300 lines; all tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success. Verified via `find src ext -name "*.rs" | xargs wc -l` that no file exceeds 300 lines. User confirmed a clean `./scripts/build_all.sh` run (build, all unit/integration tests, Clippy audit, binary reinstall) after the three follow-up fixes above.

---

## 🛠️ Short-Term Plan: Phase 41

### 💡 Current AWU Status
- [✅] AWU 910: Remove Unsafe Tool-Call Squashing, Split orchestrator.rs, Remove Production Unwrap (Result: Success)

### 📝 AWU Details

#### AWU 910: Remove Unsafe Tool-Call Squashing, Split orchestrator.rs, Remove Production Unwrap
- **Trigger**: User asked for further improvement points after AWU 909; a second read-through of `ext/context-tools`, `ext/rad-orchestrator`, and `src/wasm/rpc_meta.rs` surfaced one more real bug plus the housekeeping items AWU 909 had left as "known outstanding issues." User said to fix all of it.
- **Issue 1 (real bug, pre-existing, not introduced by AWU 908/909)**: `context-tools`' role-based squashing (`compress_messages`) collapsed consecutive non-`user`/`assistant` messages into the last one. Since `system` is stripped out before `optimize()` sees the list, and the DAG only ever produces `user`/`assistant`/`tool`/`system` nodes, this logic only ever fires on runs of `tool` messages. `ext/rad-orchestrator/src/orchestrator/runner.rs` confirms parallel `tool_calls` (multiple tool calls in one `assistant` turn) are fully supported, each producing its own consecutive `tool` DAG node — so a 2+ tool-call turn would get squashed down to one `tool` message while the preceding `assistant` message's `tool_calls` array still listed all of them, producing an API-invalid request (same failure class as AWU 78/909, but reachable in a single turn, not just long sessions). There is no role in this system for which the squash is ever safe.
- **Fix 1**: Removed `compress_messages` and the squashing pass entirely from `ext/context-tools/src/lib.rs` rather than special-casing `tool` (nothing left for the generic mechanism to safely apply to). `optimize()` is now count-based windowing only. Updated `ARCHITECTURE.md`'s Context Compactor description and the L3-reset strategy line to match. Rewrote/renamed the two tests that exercised squashing to instead assert parallel tool-call pairs survive both un-windowed and windowed optimization.
- **Fix 2 (file-size limit)**: `ext/rad-orchestrator/src/orchestrator.rs` had grown to 318 lines (over CODING.md's 300-line limit) via AWU 909's earlier `RAD_DEBUG`-unification commit. Split the reasoning/thought-stream formatting helpers (`debug_enabled`, `RawEvent`/`ToolCallChunkEvent`/`CompletionUsageEvent`, `handle_content_token`, `handle_thought_start_tag`, `handle_reasoning_text`) into a new companion module `ext/rad-orchestrator/src/orchestrator/reasoning.rs` (128 lines). `orchestrator.rs` is now 194 lines.
- **Fix 3 (production unwrap)**: `src/wasm/rpc_meta.rs`'s background LLM-stream polling thread used `connector_ref.llm_connector.as_ref().unwrap()` every loop iteration. Replaced with a `let...else` that sends an `error`-type `LlmConnectorEvent` and breaks the loop instead of panicking the thread if the connector is ever gone.
- **Fix 4 (docs)**: Added a superseded-by-`PLANS.md` note to `TASKS.md` and marked its one abandoned unfinished item (AWU 205) rather than guessing at its original scope and leaving it silently stale.
- **Not fixed (deliberate)**: Crate-level `#![allow(clippy::...)]` in every extension crate. `unsafe_op_in_unsafe_fn` and `clippy::same_length_and_capacity` appear in literally every `wit_bindgen::generate!`-using crate — a generated-code artifact, not something hand-written code can clean up without patching `wit_bindgen` itself. The remaining crate-specific allows (`needless_pass_by_value`, `collapsible_if`, `uninlined_format_args`, `cast_possible_truncation`, `manual_strip`, `too_many_lines`, `collapsible_match`) look fixable but touching 5 crates' worth of lints with no local compiler to verify against was judged too risky for this round; left as-is.
- **Scope**: `ext/context-tools/src/lib.rs`, `ARCHITECTURE.md`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/reasoning.rs` (new), `src/wasm/rpc_meta.rs`, `TASKS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success.

---

## 🛠️ Short-Term Plan: Phase 40

### 💡 Current AWU Status
- [✅] AWU 909: Fix Orphan Tool-Message Filter Ordering & CallExtension Blocking Lock (Result: Success)

### 📝 AWU Details

#### AWU 909: Fix Orphan Tool-Message Filter Ordering & CallExtension Blocking Lock
- **Trigger**: User asked for a general audit of the current code after AWU 908 shipped. Two concrete issues surfaced from reading `ext/rad-orchestrator/src/llm.rs` and `src/wasm/rpc_meta.rs` against the codebase's own documented history (AUDITING.md, CODING.md, PLANS.md archive).
- **Issue 1 (regression from AWU 908)**: The orphan-`tool`-message filter in `load_messages_from_dag` used to run *after* count-based history windowing, so any orphan the windowing created (by slicing between an `assistant`/`tool_calls` message and its `tool` reply) got cleaned up before the request reached the LLM. AWU 908 moved windowing into `context-tools` and left the filter running *before* it, on the unwindowed list — meaning `context-tools`' purely positional windowing could reintroduce an orphaned `tool` message that never gets filtered out again, recreating the class of bug AWU 78 ("Filter Out Isolated Tool Messages", a 400 Bad Request fix) originally addressed.
- **Issue 2**: `src/wasm/rpc_meta.rs`'s `CallExtension` RPC handler (the exact path AWU 908's `context-tools` calls go through every turn) held the outer `orch.wasm_runtime` lock for the full duration of the nested extension call and used a blocking `.lock()` on the target runtime, instead of the clone-Arc-then-`try_lock()` pattern used everywhere else in the same file (`GetTools`, `ExecuteTool`) since AWU 900's deadlock fix. Not a confirmed live deadlock, but the same risk class on a hot path.
- **Fix**:
  1. `ext/rad-orchestrator/src/llm.rs`: extracted the filter into `filter_orphaned_tool_messages()` and apply it twice — once after DAG reconstruction, once again after parsing `context-tools`' optimized response, before it's trusted.
  2. `src/wasm/rpc_meta.rs`: `CallExtension` now clones the target `Arc` out of a short-lived `wasm_runtime` lock, then `try_lock()`s the runtime, returning an error instead of blocking if it's busy.
- **Scope**: `ext/rad-orchestrator/src/llm.rs`, `src/wasm/rpc_meta.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success. One Clippy pedantic fixup needed mid-flight (`clippy::doc_markdown` — backticks around `` `assistant`/`tool_call` `` in a doc comment). Final run: all 5 `context-tools` unit tests + 37 `rad` lib unit tests + 23 integration tests pass, Clippy clean, binary reinstalled.

---

## 🛠️ Short-Term Plan: Phase 39

### 💡 Current AWU Status
- [✅] AWU 908: Consolidate Context Compaction Logic into `context-tools` Extension (Result: Success)

### 📝 AWU Details

#### AWU 908: Consolidate Context Compaction Logic into `context-tools` Extension
- **Trigger**: User-led review of the context management mechanism (2026-07-26) surfaced that compaction logic is split across two places: count-based `max_history_messages` windowing lives in `ext/rad-orchestrator/src/llm.rs::load_messages_from_dag`, while role-based squashing of consecutive non-user/assistant messages lives in `ext/context-tools/src/lib.rs::optimize`. Same responsibility ("shrink history to fit"), two owners, one stateful (Orchestrator's global `OrchestratorState`) and one stateless.
- **Decision & Rationale**: Draw the boundary between "mechanical, DAG-coupled reconstruction" and "judgment-based compaction policy" rather than the doc's previous "construction vs. compaction" wording alone. DAG traversal (walking `parent_ids`, deserializing node text into `Message`, filtering orphaned `tool_call_id`s, assembling the system prompt) stays in the Orchestrator — it's a pure, deterministic transform tightly bound to `rad_models::Dag`, already shared as a library crate rather than crossed via RPC, and putting it behind a `context-tools` RPC call would add a hop on the hot path and turn an optional optimization into a single point of failure for message delivery. Compaction (what to keep/discard/summarize under a size budget) is genuine policy that benefits from being isolated, pluggable, and testable in one place — so both existing strategies (count windowing + role squashing) consolidate into `context-tools`.
- **Objective**: Extend the `context-tools` WIT `optimization-request` (`wit/context-tools.wit`) with a history-length threshold parameter. Move the count-based windowing logic (first-message-preserving trim, currently in `load_messages_from_dag`) into `MyContextTools::optimize` in `ext/context-tools/src/lib.rs`, applied ahead of or alongside the existing role-based squashing pass. Remove the now-duplicate windowing logic from `ext/rad-orchestrator/src/llm.rs`, and stop reading `max_history_messages` out of the Orchestrator's global `STATE` for that purpose (pass it as a request parameter instead).
- **Scope**: `wit/context-tools.wit`, `ext/context-tools/src/lib.rs`, `ext/rad-orchestrator/src/llm.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/types.rs` (state plumbing), unit tests in `ext/context-tools/src/lib.rs` and `ext/rad-orchestrator/src/orchestrator/tests.rs`.
- **Non-Goals**: DAG traversal, system-prompt assembly, and orphaned-tool-call filtering in `load_messages_from_dag` are explicitly out of scope and remain in the Orchestrator.
- **Definition of Done (DoD)**: `context-tools::optimize` accepts a history-length threshold and applies unified count- and role-based trimming in a single pass; `load_messages_from_dag` retains only DAG traversal/parsing/system-prompt assembly with no ad-hoc `max_history` windowing; all workspace unit/integration tests and Clippy audit pass cleanly.
- **Result**: Success. Added `max-history: option<u32>` to `optimization-request` in `wit/context-tools.wit`. `ext/context-tools/src/lib.rs` now applies `apply_history_window` (ported first+recent trim) ahead of the existing `compress_messages` role-squash pass inside `optimize`, with combined summary text and 5 unit tests (incl. windowing-only, under-limit no-op, and windowing+compression interaction). `ext/rad-orchestrator/src/llm.rs::load_messages_from_dag` no longer performs count-based trimming itself — it only reconstructs messages from the DAG, then forwards `max_history_messages` as the new `max-history` request field to `context-tools`. Used `try_from`/`unwrap_or` instead of `as` casts at the `usize`/`u32` boundary to stay clean under `clippy::pedantic`. Verified via `./scripts/build_all.sh` on 2026-07-26: all 5 new `context-tools` unit tests pass, all 37 `rad` lib unit tests + 23 integration tests pass, Clippy audit (`-D warnings`) clean, `rad` binary rebuilt and reinstalled to `~/.cargo/bin/rad`.

---

## 📜 Completed Work Archive (Phases 19–38)

Condensed one-line-per-AWU log. Full narrative detail (trigger/root-cause/fix breakdowns) has been trimmed; scope and outcome are preserved. Older phases (1–18) predate this log's granularity and are tracked only in the Roadmap above.

**Phase 38 (v0.43.0) — Deep End-to-End Codebase Audit & Verification of MCP Subsystem**
- AWU 907: Fixed `ExecutionHandle` being dropped (and SIGKILLing) spawned MCP servers right after handshake, by retaining the handle in `ActiveMcpServer`. — `ext/mcp-tool-provider`
- AWU 906: Non-blocking EOF handling in the MCP stream reader instead of looping to a 10s timeout. — `ext/mcp-tool-provider/src/client.rs`
- AWU 905: Unified canonical path resolution across permission checks and RPC gateways. — `src/wasm/permissions.rs`
- AWU 904: Standardized global config path resolution; cleaned diagnostic pipelines behind `RAD_MCP_DEBUG`. — `ext/mcp-tool-provider`, `src/config.rs`, `src/process.rs`
- AWU 903: Standardized stdio pipe spawning & executable PATH lookup, delegating shell features to `bash -c`. — `src/process.rs`
- AWU 902: Host-side deep-merge of global (`~/.rad/config.json`) and project configs in `load_config()`. — `src/config.rs`
- AWU 901: WASM-side deep-merge of `mcp_servers` configs (global + local). — `ext/mcp-tool-provider/src/client.rs`
- AWU 900: Fixed `verify_rpc_exclude` deadlock on nested extension calls via `try_lock()`. — `src/wasm/imports_rpc.rs`
- AWU 899: Exhaustive MCP subsystem audit — fixed stdin pipe pollution from `ping` health checks, dead handle cleanup, strict JSON-RPC id matching.
- AWU 897: Dedicated stdio piping for `open_process`/MCP server spawning, replacing PTY to avoid line mangling. — `src/process.rs`

**Phase 35 (v0.40.0) — Unified Path Resolution Architecture & Gateway Normalization**
- AWU 896: Centralized path resolution/tilde expansion/canonicalization into a single `resolve_target_path`. — `src/fs.rs`, `src/wasm/permissions.rs`, `src/wasm/rpc.rs`

**Phase 34 (v0.39.0) — Fix Tilde Expansion in Host Fs Subsystem**
- AWU 895: Applied `expand_tilde` across `file_read`/`file_write`/`canonicalize_path`/`has_permission`. — `src/fs.rs`

**Phase 33 (v0.38.0) — Fix read_config_file Error String Swallowing**
- AWU 894: `read_config_file` returns `None` on host RPC errors so config discovery falls back to `~/.rad/config.json` correctly. — `ext/mcp-tool-provider/src/client.rs`

**Phase 32 (v0.37.0) — Fix Tilde Expansion in WASM Permissions & Align MCP Config Discovery**
- AWU 893: Tilde expansion in `permissions.rs::has_path_permission`; aligned `load_mcp_config` search order with `src/config.rs`.

**Phase 31 (v0.36.0) — Stdio Pipe Fallback for PTY Permission Errors**
- AWU 892: Stdio pipe fallback in `ProcessManager::spawn_bash_process` when `openpty()` fails with permission denied. — `src/process.rs`

**Phase 30 (v0.35.0) — Complete Removal of Built-in Shell Fallbacks & Accurate MCP Startup Verification**
- AWU 888: Startup display reflects real runtime tool verification instead of pre-verification logs. — `src/main.rs`, `src/orchestrator/runner.rs`
- AWU 889: Removed legacy `execute_command`/`spawn_bash_process` fallbacks from the orchestrator. — `ext/rad-orchestrator/src/orchestrator/runner.rs`
- AWU 890: Removed legacy command fallbacks from host RPC. — `src/wasm/rpc_meta.rs`, `src/wasm/imports_rpc.rs`
- AWU 891: Build/test/audit/install/push cycle.

**Phase 29 (v0.34.0) — MCP Host Tilde Expansion & Instant Tool Result Visibility**
- AWU 884: Host-side tilde expansion & permission bypass for MCP binary paths (e.g. `~/.cargo/bin/`). — `src/wasm/imports_rpc.rs`, `src/wasm/permissions.rs`
- AWU 885: Removed redundant WASM-side unexpanded-path checks. — `ext/mcp-tool-provider/src/client.rs`
- AWU 886: Instant tool result output to terminal via `WriteStdout`. — `ext/rad-orchestrator/src/orchestrator/runner.rs`
- AWU 887: Build/test/audit/install/push cycle.

**Phase 28 (v0.33.0) — Documentation Update, Config Deployment & Git Main Release**
- AWU 880: Synced `ARCHITECTURE.md`/`CONFIG.md`/`README.md` with the `rad-orchestrator`/`llm-connector` refactor.
- AWU 881: Updated global/workspace config files to reference `rad-orchestrator`/`llm-connector`.
- AWU 882: Build/test/verification cycle (37 unit + 23 integration tests, Clippy).
- AWU 883: Commit & push to main.

**Phase 23 (v0.28.0) — Configurable LLM Endpoints & /llm Slash Command Management**
- AWU 863: Added `LlmConfig`/`LlmEndpointProfile` with `env:VAR_NAME` credential resolution. — `src/config.rs`
- AWU 864: Implemented `/llm` slash command subsystem (list/switch/test/add/model). — `src/command/llm.rs`
- AWU 865: Propagated active LLM profile settings to the WASM connector runtime. — `ext/openai-connector/src/lib.rs`
- AWU 866: Audit/build/install/verify cycle.

**Phase 22 (v0.27.0) — Fix LLM Connection Hangs & Add Graceful Error Handling**
- AWU 860: Added HTTP connect timeouts (10s) and clearer error messages for unreachable hosts. — `src/http.rs`
- AWU 861: `openai-connector` reads `OPENAI_BASE_URL`/`OPENAI_API_KEY`, fails fast when unreachable/unconfigured.
- AWU 862: Audit/build/install cycle.

**Phase 21 (v0.26.0) — Release Build, Local Installation, ~/.rad/config.json Creation & Push to GitHub**
- AWU 857: Created default `~/.rad/config.json`.
- AWU 858: Built & installed release binaries to `~/.cargo/bin`.
- AWU 859: Commit & push to GitHub.

**Phase 19–20 (v0.24.0–v0.25.0) — Remove Built-in Extensions & Standardize Global Config Directory**
- AWU 852: Finalized removal plan for `core-tool-provider`/`web-access` extensions.
- AWU 853: Deleted legacy extension crates and WIT/host bindings. — `ext/core-tool-provider`, `ext/web-access`, `src/wasm/*`
- AWU 854: Routed tool calls via `mcp-tool-provider`; updated `rad.json`/`Cargo.toml`/tests.
- AWU 855: Synced `ARCHITECTURE.md`/`EXTENSIONS.md`/`CONFIG.md`; ran full verification audit.
- AWU 856: Prioritized `~/.rad/config.json` in global config discovery. — `src/config.rs`
