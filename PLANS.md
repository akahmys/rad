# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-23

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
- [🔄] Phase 28: Documentation Update, Config Deployment & Git Main Release (v0.33.0)

---

## 🛠️ Short-Term Plan: Phase 28

### 💡 Current AWU Status
- [ ] AWU 880: Update Architecture & Configuration Documentation (ARCHITECTURE.md, CONFIG.md, README.md)
- [ ] AWU 881: Update Local & Workspace Configuration Files (~/.rad/config.json, ~/projects/rad/rad.json)
- [ ] AWU 882: Run One-Command Build Script (scripts/build_all.sh) & Verification
- [ ] AWU 883: Commit & Push All Changes to Git (main Branch)

### 📝 AWU Details

#### AWU 880: Update Architecture & Configuration Documentation
- **Objective:** Synchronize `ARCHITECTURE.md`, `CONFIG.md`, and `README.md` with recent refactoring (`rad-orchestrator`, `llm-connector`, unified config precedence, trace_id, and `scripts/build_all.sh`).
- **Scope:** `ARCHITECTURE.md`, `CONFIG.md`, `README.md`.
- **Definition of Done (DoD):** Documentation accurately reflects all architecture and config rules.

#### AWU 881: Update Local & Workspace Configuration Files
- **Objective:** Update `~/.rad/config.json` and workspace `rad.json` / `rad.local.json` to reference `rad-orchestrator` and `llm-connector`.
- **Scope:** `~/.rad/config.json`, `rad.json`.
- **Definition of Done (DoD):** Config files point to updated WASM extension paths and profiles.

#### AWU 882: Run One-Command Build Script & Verification
- **Objective:** Execute `./scripts/build_all.sh` to ensure build, test, clippy, and binary installation pass completely.
- **Scope:** Workspace binaries and WASM targets.
- **Definition of Done (DoD):** Clean build and local installation succeed.

#### AWU 883: Commit & Push All Changes to Git
- **Objective:** Stage all modified/untracked files, commit with descriptive message, and push to main branch.
- **Scope:** Git repository.
- **Definition of Done (DoD):** Working tree clean and pushed to remote main.

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





