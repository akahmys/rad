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
