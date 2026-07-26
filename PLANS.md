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

---

## 🛠️ Short-Term Plan: Phase 38

### 💡 Current AWU Status
- [✅] AWU 907: Fix ExecutionHandle Drop Prematurely Killing Spawned MCP Servers (Result: Success)
- [✅] AWU 906: Fix Non-blocking EOF Handling in MCP Stream Reader (Result: Success)
- [✅] AWU 905: Unified Canonical Path Resolution in Permissions and Subsystem Gateways (Result: Success)
- [✅] AWU 904: Standardize WASM Global Config Path Resolution and Clean Diagnostic Pipelines (Result: Success)
- [✅] AWU 903: Standardize Stdio Pipe Spawning & Executable PATH Lookup in Process Subsystem (Result: Success)
- [✅] AWU 902: Host Config Synthesis Deep Merge for User Global ~/.rad/config.json and Project Local Configs (Result: Success)
- [✅] AWU 901: Implement Deep Merge Synthesis in WASM mcp-tool-provider Config Discovery (Result: Success)
- [✅] AWU 900: Fix verify_rpc_exclude deadlock in nested WASM runtime calls (Result: Success)
- [✅] AWU 899: Comprehensive codebase audit across ext/mcp-tool-provider, src/process.rs, src/wasm/, and src/orchestrator/ (Result: Success)

### 📝 AWU Details

#### AWU 907: Fix ExecutionHandle Drop Prematurely Killing Spawned MCP Servers
- **Trigger**: User reported `rad` reporting "0 tools" for `mcp-tool-provider` in interactive sessions (`~/projects/test` and `~/projects/rad` alike) despite `~/.cargo/bin/core-utilities-mcp` and `~/.cargo/bin/web-access-mcp` both responding correctly to a manual `initialize` -> `notifications/initialized` -> `tools/list` JSON-RPC sequence outside of `rad`.
- **Root Cause & Discoveries**:
  1. `RunningProcess` (`src/process.rs`) implements `Drop` to call `kill_group()`, SIGKILLing the process's entire OS process group when the value is dropped.
  2. In `init_mcp_servers()` (`ext/mcp-tool-provider/src/client.rs`), the `ExecutionHandle` returned by `open_process()` was only used to extract `stdin`/`stdout` stream handles via `.get_stdin()`/`.get_stdout()`, then discarded (never stored in `ActiveMcpServer`). It went out of scope and dropped at the end of each loop iteration, which SIGKILLed the just-spawned MCP server process immediately after the `initialize` handshake completed successfully.
  3. Consequently, by the time `get_tools()` issued a `tools/list` request, the target process was already dead, and `stdin.write()` failed with `Broken pipe (os error 32)`, silently yielding an empty tool list.
  4. Diagnosed by temporarily reinstating `[MCP Diagnostic]` instrumentation (previously stripped by AWU 904 during diagnostic cleanup, which is what prevented this bug from surfacing earlier).
- **Fix**:
  1. Added an `exec: wit::ExecutionHandle` field to `ActiveMcpServer` and stored the handle there instead of discarding it, keeping the spawned process alive for the life of the connection.
  2. Restored `[MCP Diagnostic]` instrumentation in `client.rs`/`lib.rs`, gated silent-by-default behind a `RAD_MCP_DEBUG` environment variable so it doesn't clutter normal output but remains available for future troubleshooting.
- **Scope**: `ext/mcp-tool-provider/src/client.rs`, `ext/mcp-tool-provider/src/lib.rs`.
- **Definition of Done (DoD)**: `rad` in `~/projects/test` initializes both configured MCP servers and reports `[OK] Verified 18 tools from extension 'mcp-tool-provider'` (15 from `core-utilities-mcp`, 3 from `web-access-mcp`) without diagnostic noise by default.
- **Result**: Success. Verified live in an interactive `rad` session.

#### AWU 906: Fix Non-blocking EOF Handling in MCP Stream Reader
- **Objective**: Refactor `read_line` in `ext/mcp-tool-provider/src/client.rs` to process buffered bytes on EOF/stream error and terminate gracefully instead of looping on empty reads until 10s timeout. Rebuild and verify complete suite.
- **Scope**: `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD)**: `read_line` detects stream errors and EOF gracefully; all 60 workspace unit and integration tests and Clippy audit pass cleanly.

#### AWU 905: Unified Canonical Path Resolution in Permissions and Subsystem Gateways
- **Objective**: Refactor path resolution logic into a single helper (`canonicalize_path`) in `src/wasm/permissions.rs` used uniformly by `FileRead` RPC and permission gateways. Rebuild and verify complete suite.
- **Scope**: `src/wasm/permissions.rs`.
- **Definition of Done (DoD)**: Single unified path resolution gateway handles all raw, relative, and tilde-prefixed paths; all 60 unit/integration tests and Clippy audit pass cleanly.

#### AWU 904: Standardize WASM Global Config Path Resolution and Clean Diagnostic Pipelines
- **Objective**: Standardize `load_mcp_config` in `ext/mcp-tool-provider/src/client.rs` to query absolute global configuration path (`$HOME/.rad/config.json`) first, clean up diagnostic sub-processes, fix clippy lints, and rebuild/re-install binaries.
- **Scope**: `ext/mcp-tool-provider/src/client.rs`, `src/config.rs`, `src/process.rs`.
- **Definition of Done (DoD)**: Clippy passes cleanly without warnings/errors, all automated unit and integration tests pass, and binary is successfully re-installed locally.

#### AWU 903: Standardize Stdio Pipe Spawning & Executable PATH Lookup in Process Subsystem
- **Objective**: Fix executable PATH lookup in `src/process.rs` for direct binary execution while properly delegating shell-feature commands (pipes, redirects, wildcards) to `bash -c`.
- **Scope**: `src/process.rs`.
- **Definition of Done (DoD)**: All 37 workspace unit tests and 23 integration tests pass, stdio piping behavior is clean and stable across all working directories.

#### AWU 902: Host Config Synthesis Deep Merge for User Global ~/.rad/config.json and Project Local Configs
- **Objective**: Refactor host `load_config()` in `src/config.rs` to always load `~/.rad/config.json` as base configuration and deep-merge project local config files on top, ensuring global extensions and permissions are preserved regardless of working directory.
- **Scope**: `src/config.rs`.
- **Definition of Done (DoD)**: `load_config` merges global base and project local configs, all tests pass, and running `rad` in directories without a project `rad.json` (such as `~/projects/test`) correctly inherits global MCP tools and permissions.

#### AWU 901: Implement Deep Merge Synthesis in WASM mcp-tool-provider Config Discovery
- **Objective**: Merge `mcp_servers` configurations from global (`~/.rad/config.json`) and local (`rad.json`, `.rad/config.json`) files so global tools remain available while allowing project-level overrides/additions.
- **Scope**: `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD)**: `load_mcp_config()` deep-merges global and local `mcp_servers`, passes all automated tests, and `rad` CLI successfully initializes 18 MCP tools in interactive sessions.

#### AWU 900: Fix verify_rpc_exclude deadlock in nested WASM runtime calls
- **Objective**: Replace `.lock()` with `.try_lock()` in `verify_rpc_exclude` to eliminate deadlocks caused by nested extension calls (e.g. rad-orchestrator -> llm-connector -> open_http_stream).
- **Scope**: `src/orchestrator/runner.rs`, `src/wasm/imports_rpc.rs`.
- **Definition of Done (DoD)**: `multi_extension_tests` and `git_autopilot_tests` pass cleanly without timeouts or PoisonErrors, and all 60 tests in `./scripts/build_all.sh` pass.
- **Result**: Success. All tests passed, Clippy audit clean, binary built and installed locally.

#### AWU 899: Exhaustive Audit of MCP Subsystem Dataflow and Process Lifecycle
- **Trigger**: User requested an exhaustive, deep codebase verification of the MCP tool provider subsystem.
- **Root Cause & Discoveries**:
  1. `init_mcp_servers` previously wrote `ping` JSON-RPC messages to verify pipe health, which polluted the stdin pipe buffer and caused subsequent `tools/list` responses to parse fail (returning `0 tools`).
  2. Host-side `clear_runtimes()` in retry loops killed background MCP server process groups, leaving dead handles inside `MCP_SERVERS`.
  3. `send_mcp_bytes` did not strictly validate response `id` against request `id`.
- **Fix**:
  1. Replaced destructive `ping` writes with a non-destructive 0-byte pipe write (`write(b"")`) in `init_mcp_servers()`.
  2. Implemented strict JSON-RPC request/response `id` matching in `send_mcp_bytes`.
  3. Removed unnecessary `clear_runtimes()` on every task iteration in `src/orchestrator/runner.rs`.
- **Result**: All 60 automated unit/integration tests passed cleanly. Zero-byte non-destructive health checks eliminate all pipe pollution and race conditions.

#### AWU 897: Dedicated stdio piping for open_process & MCP servers in ProcessManager
- **Objective:** Route `open_process` and MCP server spawning to stdio pipes instead of PTY so interactive TTY sessions process clean JSON-RPC.
- **Scope:** `src/process.rs`, `src/wasm/permissions.rs`, `src/wasm/imports_rpc.rs`, `src/wasm/rpc_process.rs`.
- **Definition of Done (DoD):** Interactive TTY sessions successfully initialize 18 MCP tools without PTY line mangling or handshake failures.
- **Result:** Success. Standardized Stdio piping in ProcessManager, fixed tilde expansion in WASM permission gateway, passed all 60 tests and Clippy, and verified 18 MCP tools loaded in interactive test sessions.


### 💡 Current AWU Status
- [✅] AWU 896: Implement Unified Path Resolution Architecture in src/fs.rs, permissions.rs, and rpc.rs (Result: Success)

### 📝 AWU Details

#### AWU 896: Implement Unified Path Resolution Architecture in src/fs.rs, permissions.rs, and rpc.rs
- **Objective:** Centralize all path resolution, tilde expansion, and canonicalization into a single `resolve_target_path` in `FsSubsystem` and enforce RPC Gateway normalization.
- **Scope:** `src/fs.rs`, `src/wasm/permissions.rs`, `src/wasm/rpc.rs`, `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD):** Unified path resolution passes all 60 tests and Clippy, and `rad` initializes 18 MCP tools when executed in `projects/test`.
- **Result:** Success. Centralized path resolution in `resolve_target_path`, eliminated ad-hoc tilde expansion, passed all 60 tests and Clippy, installed binary, and verified 18 MCP tools loaded in `projects/test`.



### 💡 Current AWU Status
- [🔄] AWU 895: Implement expand_tilde in host FsSubsystem methods in src/fs.rs (In Progress)

### 📝 AWU Details

#### AWU 895: Implement expand_tilde in host FsSubsystem methods in src/fs.rs
- **Objective:** Apply `crate::config::expand_tilde` across `file_read`, `file_write`, `canonicalize_path`, and `has_permission` in `src/fs.rs` so paths starting with `~` are read successfully from any directory.
- **Scope:** `src/fs.rs`.
- **Definition of Done (DoD):** `rad` loads 18 MCP tools successfully when launched in `projects/test` or any other working directory.


### 💡 Current AWU Status
- [✅] AWU 894: Fix read_config_file error string swallowing & JSON validation in WASM mcp-tool-provider (Result: Success)

### 📝 AWU Details

#### AWU 894: Fix read_config_file error string swallowing & JSON validation in WASM mcp-tool-provider
- **Objective:** Ensure `read_config_file` returns `None` on host RPC errors so config discovery correctly falls back to `~/.rad/config.json`.
- **Scope:** `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD):** `rad` falls back to `~/.rad/config.json` and initializes 18 MCP tools when executed in `projects/test`.
- **Result:** Success. Validated JSON in `read_config_file`, updated `load_mcp_config` to ignore error strings, passed all 60 tests and Clippy, verified 18 tools loaded when executed from `projects/test`.



### 💡 Current AWU Status
- [✅] AWU 893: Fix Tilde Expansion in WASM permissions.rs & align load_mcp_config search order (Result: Success)

### 📝 AWU Details

#### AWU 893: Fix Tilde Expansion in WASM permissions.rs & align load_mcp_config search order
- **Objective:** Expand tildes in `permissions.rs::has_path_permission` and align `load_mcp_config` search order with `src/config.rs` so MCP tools load reliably across any working directory.
- **Scope:** `src/wasm/permissions.rs`, `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD):** `rad` loads 18 MCP tools successfully when executed from `projects/test` or any other directory.
- **Result:** Success. Added tilde expansion in `permissions.rs`, aligned discovery path order in `ext/mcp-tool-provider`, passed all 60 tests, and verified 18 tools loaded when executed from `projects/test`.



### 💡 Current AWU Status
- [✅] AWU 892: Implement Stdio Pipe Fallback in ProcessManager for PTY openpty PermissionDenied errors (Result: Success)

### 📝 AWU Details

#### AWU 892: Stdio Pipe Fallback in ProcessManager for PTY openpty PermissionDenied errors
- **Objective:** Support standard stdio piping in `ProcessManager::spawn_bash_process` when `openpty()` fails with permission denied, enabling MCP servers to spawn reliably.
- **Scope:** `src/process.rs`.
- **Definition of Done (DoD):** `spawn_bash_process` falls back to stdio pipes on PTY failure, tests pass, and MCP tools load successfully.
- **Result:** Success. Implemented Stdio Pipe fallback in `ProcessManager`, all 60 tests passed, Clippy clean, verified 18 MCP tools loading at runtime.



### 💡 Current AWU Status
- [✅] AWU 888: Remove premature MCP extension header printing in `src/main.rs` & implement verified status output in `src/orchestrator/runner.rs` (Result: Success)
- [✅] AWU 889: Delete legacy `execute_command`/`spawn_bash_process` fallbacks from `ext/rad-orchestrator/src/orchestrator/runner.rs` (Result: Success)
- [✅] AWU 890: Clean up legacy command fallbacks in `src/wasm/rpc_meta.rs` & `src/wasm/imports_rpc.rs` (Result: Success)
- [✅] AWU 891: One-command build, full test verification, audit, local installation & push to main (Result: Success)

### 📝 AWU Details

#### AWU 888: Verified status output for MCP extension loading
- **Objective:** Update extension startup display to reflect real runtime tool verification (green [OK - N tools] or red [FAILED - 0 tools]).
- **Scope:** `src/main.rs`, `src/orchestrator/runner.rs`.
- **Definition of Done (DoD):** Startup display accurately reflects tool counts without misleading pre-verification logs.

#### AWU 889: Delete legacy fallbacks from rad-orchestrator
- **Objective:** Remove `execute_command` and `spawn_bash_process` alias handling from `runner.rs`.
- **Scope:** `ext/rad-orchestrator/src/orchestrator/runner.rs`.
- **Definition of Done (DoD):** Only tools from tool providers are recognized.

#### AWU 890: Clean up legacy command fallbacks in host RPC
- **Objective:** Remove legacy fallback handlers for execute_command/bash in `rpc_meta.rs` and `imports_rpc.rs`.
- **Scope:** `src/wasm/rpc_meta.rs`, `src/wasm/imports_rpc.rs`.
- **Definition of Done (DoD):** Unexposed shell fallback execution is completely removed.

#### AWU 891: One-command build, full test verification, audit, local installation & push to main
- **Objective:** Run `./scripts/build_all.sh`, pass all 60 tests and Clippy, install binary, and push to main.
- **Scope:** Workspace repository.
- **Definition of Done (DoD):** All tests pass, binary installed, clean git status on main.

### 📝 AWU Details

#### AWU 884: Host-side tilde expansion & permission bypass for MCP binary paths
- **Objective:** Ensure host-side `open_process` expands `~` for binary commands and `permissions.rs` allows execution of binaries in `~/.cargo/bin/`.
- **Scope:** `src/wasm/imports_rpc.rs`, `src/wasm/permissions.rs`.
- **Definition of Done (DoD):** MCP binary paths with `~` are expanded and permitted by host RPC.

#### AWU 885: Remove WASM-side unexpanded path check in `ext/mcp-tool-provider`
- **Objective:** Clean up WASM-side path checking in `client.rs` so unexpanded tildes are passed to the host cleanly.
- **Scope:** `ext/mcp-tool-provider/src/client.rs`.
- **Definition of Done (DoD):** WASM client delegates path expansion to host `open_process`.

#### AWU 886: Instant Tool Result Output via `WriteStdout`
- **Objective:** Write tool execution results directly to terminal stdout in `runner.rs` immediately after execution.
- **Scope:** `ext/rad-orchestrator/src/orchestrator/runner.rs`.
- **Definition of Done (DoD):** Raw tool results are printed to screen in real time.

#### AWU 887: One-command build, full test verification, audit, local installation & push to main
- **Objective:** Run `./scripts/build_all.sh`, pass all 60 tests and Clippy, install binary, and push to main.
- **Scope:** Workspace repository.
- **Definition of Done (DoD):** All tests pass, binary installed, clean git status on main.

### 📝 AWU Details

#### AWU 880: Update Architecture & Configuration Documentation
- **Objective:** Synchronize `ARCHITECTURE.md`, `CONFIG.md`, and `README.md` with recent refactoring (`rad-orchestrator`, `llm-connector`, unified config precedence, trace_id, and `scripts/build_all.sh`).
- **Scope:** `ARCHITECTURE.md`, `CONFIG.md`, `README.md`.
- **Definition of Done (DoD):** Documentation accurately reflects all architecture and config rules.
- **Result:** Success. Updated `ARCHITECTURE.md`, `CONFIG.md`, and `README.md`.

#### AWU 881: Update Local & Workspace Configuration Files
- **Objective:** Update `~/.rad/config.json` and workspace `rad.json` / `rad.local.json` to reference `rad-orchestrator` and `llm-connector`.
- **Scope:** `~/.rad/config.json`, `rad.json`.
- **Definition of Done (DoD):** Config files point to updated WASM extension paths and profiles.
- **Result:** Success. Verified and confirmed `~/.rad/config.json` configuration.

#### AWU 882: Run One-Command Build Script & Verification
- **Objective:** Execute `./scripts/build_all.sh` to ensure build, test, clippy, and binary installation pass completely.
- **Scope:** Workspace binaries and WASM targets.
- **Definition of Done (DoD):** Clean build and local installation succeed.
- **Result:** Success. Passed all 37 unit tests, 23 integration tests, Clippy audit, and installed `rad` binary.

#### AWU 883: Commit & Push All Changes to Git
- **Objective:** Stage all modified/untracked files, commit with descriptive message, and push to main branch.
- **Scope:** Git repository.
- **Definition of Done (DoD):** Working tree clean and pushed to remote main.
- **Result:** Success. Merged `rad-autopilot-1784721492` into `main` and pushed to GitHub.

### 💡 Current AWU Status
- [✅] AWU 863: Add LlmConfig & LlmEndpointProfile to src/config.rs with env: resolution (Result: Success)
- [✅] AWU 864: Implement /llm slash command subsystem (list, switch, test, add, model) (Result: Success)
- [✅] AWU 865: Propagate active LLM configuration to WASM connector runtime (Result: Success)
- [✅] AWU 866: Run audit checks, build release binaries, install locally, and verify (Result: Success)

### 📝 AWU Details

#### AWU 863: Add LlmConfig & LlmEndpointProfile to src/config.rs with env: resolution
- **Objective:** Add `llm` field to `Config` struct supporting multiple named LLM server profiles, active profile selection, and `env:VAR_NAME` resolution for credentials.
- **Scope:** `src/config.rs`, `src/config/tests.rs`.
- **Definition of Done (DoD):** `Config` deserializes `llm` section from JSONC and resolves `env:` references properly.
- **Result:** Success. Added `LlmConfig` / `LlmEndpointProfile` and `env:` resolution method.

#### AWU 864: Implement /llm slash command subsystem (list, switch, test, add, model)
- **Objective:** Implement `/llm` slash command in `src/command/llm.rs` supporting interactive selection, switching by name/number, parallel health checks (`/llm test`), dynamic addition (`/llm add`), and model selection (`/llm model`).
- **Scope:** `src/command/llm.rs`, `src/command.rs`, `src/command/tests.rs`.
- **Definition of Done (DoD):** All `/llm` subcommand variants parse cleanly and execute expected actions.
- **Result:** Success. Implemented `/llm` command subsystem with interactive listing, switching by name/number, testing, adding, and model updating.

#### AWU 865: Propagate active LLM configuration to WASM connector runtime
- **Objective:** Propagate active LLM profile settings (`base_url`, `api_key`, `model`) from `Orchestrator` to `openai-connector` and environment state.
- **Scope:** `src/orchestrator/runner.rs`, `ext/openai-connector/src/lib.rs`.
- **Definition of Done (DoD):** LLM requests use the currently active profile settings at runtime.
- **Result:** Success. Active LLM profile environment settings are applied before running tasks.

#### AWU 866: Run audit checks, build release binaries, install locally, and verify
- **Objective:** Run `cargo test`, `clippy`, secret/license scans, rebuild WASM components and release `rad` binary, and install to `~/.cargo/bin/rad`.
- **Scope:** All workspace files.
- **Definition of Done (DoD):** All checks pass and `rad` binary is installed.
- **Result:** Success. Passed 37 library unit tests, 23 integration tests, clippy, license/secret audits, and installed updated binary to `~/.cargo/bin/rad`.

### 📝 AWU Details

#### AWU 860: Add HTTP connect timeouts and clear error handling for LLM stream failures
- **Objective:** Add explicit connection timeouts (e.g., 5-10s) in `src/http.rs` for `reqwest::Client` and return clear, actionable error messages when host is unreachable or connection is refused.
- **Scope:** `src/http.rs`.
- **Definition of Done (DoD):** Unreachable HTTP streams time out quickly with clear error messages instead of blocking infinitely.
- **Result:** Success. Added 10s connect timeout to HTTP client builder and updated error formatting.

#### AWU 861: Update openai-connector to support OPENAI_BASE_URL and OPENAI_API_KEY with graceful connection handling
- **Objective:** Update `ext/openai-connector` to read `OPENAI_BASE_URL` and `OPENAI_API_KEY` environment variables, removing hardcoded fallback assumptions, and gracefully reporting connection failures.
- **Scope:** `ext/openai-connector/src/lib.rs`.
- **Definition of Done (DoD):** `openai-connector` reads `OPENAI_BASE_URL` / `OPENAI_API_KEY` and handles unreachable endpoints cleanly.
- **Result:** Success. Updated `openai-connector` to construct URLs from `OPENAI_BASE_URL` / `OPENAI_API_KEY` and fail fast if unconfigured.

#### AWU 862: Run audit checks, rebuild release binaries, and install locally
- **Objective:** Run `cargo test`, `clippy`, rebuild release binaries, and install updated `rad` binary to `~/.cargo/bin/rad`.
- **Scope:** All workspace files.
- **Definition of Done (DoD):** `cargo test` passes, clippy passes, and binary is updated.
- **Result:** Success. Built WASM components and release binary, passes secret/license scans, and installed to `~/.cargo/bin/rad`.

### 📝 AWU Details

#### AWU 857: Create ~/.rad/config.json global configuration file
- **Objective:** Create ~/.rad/ directory and default global configuration file ~/.rad/config.json.
- **Scope:** ~/.rad/config.json.
- **Definition of Done (DoD):** ~/.rad/config.json exists and is valid JSONC.
- **Result:** Success. Created ~/.rad/config.json with default configuration.

#### AWU 858: Build release binaries and install locally
- **Objective:** Build all WASM extensions and rad release binary, and install rad binary to ~/.cargo/bin or system PATH.
- **Scope:** Build and install commands.
- **Definition of Done (DoD):** Binary installed and executable.
- **Result:** Success. Built WASM extensions in release profile and installed rad binary to ~/.cargo/bin/rad.

#### AWU 859: Commit and push changes to GitHub repository
- **Objective:** Stage all changed files, create git commit, and push to main branch.
- **Scope:** Git commands.
- **Definition of Done (DoD):** Clean git status and successful push to remote origin main.
- **Result:** Success. Merged and pushed commit 9c4eb59 to main branch on GitHub.

### 📝 AWU Details

#### AWU 852: Design evaluation and implementation plan for removing built-in core-tool-provider and web-access extensions
- **Objective:** Finalize implementation plan for removing core-tool-provider and web-access extensions without hardcoding default MCP server paths in rad.
- **Scope:** PLANS.md, implementation_plan.md.
- **Definition of Done (DoD):** Proposal updated and presented to user.
- **Result:** Success. Plan finalized and approved.

#### AWU 853: Delete ext/core-tool-provider, ext/web-access, wit/web-access.wit, and clean up WASM host bindings
- **Objective:** Remove legacy extension crates and WIT/host bindings for web-access and core-tool-provider.
- **Scope:** ext/core-tool-provider, ext/web-access, wit/web-access.wit, src/wasm/*.
- **Definition of Done (DoD):** Legacy extension crates removed and host code compiles cleanly without web-access host functions.
- **Result:** Success. Deleted ext/core-tool-provider, ext/web-access, wit/web-access.wit, and host bindings in src/wasm.

#### AWU 854: Update rad.json default configuration, Cargo.toml workspace members, and test suites
- **Objective:** Update configuration to route tool calls via mcp-tool-provider, remove workspace dependencies, and fix/update integration tests.
- **Scope:** rad.json, Cargo.toml, tests/*.
- **Definition of Done (DoD):** `cargo check` and `cargo test` pass.
- **Result:** Success. Cargo.toml, rad.json, and tests updated.

#### AWU 855: Update documentation and run verification audit
- **Objective:** Reflect structural changes in ARCHITECTURE.md, EXTENSIONS.md, CONFIG.md, and run all audit checks.
- **Scope:** ARCHITECTURE.md, EXTENSIONS.md, CONFIG.md.
- **Definition of Done (DoD):** All checks pass and documentation is aligned.
- **Result:** Success. All automated tests, clippy, check, and secret scans passed.

#### AWU 856: Update global config discovery to prioritize ~/.rad/config.json and update CONFIG.md
- **Objective:** Update config discovery in src/config.rs to prioritize ~/.rad/config.json (and ~/.rad/config.local.json) for global configuration.
- **Scope:** src/config.rs, CONFIG.md.
- **Definition of Done (DoD):** Config discovery loads ~/.rad/config.json correctly and tests pass.
- **Result:** Success. Global config path updated to ~/.rad/config.json with fallback support.





