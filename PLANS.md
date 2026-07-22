# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-18

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

---

## 🛠️ Short-Term Plan: Phase 21

### 💡 Current AWU Status
- [✅] AWU 857: Create ~/.rad/config.json global configuration file (Result: Success)
- [✅] AWU 858: Build release binaries and install locally (Result: Success)
- [✅] AWU 859: Commit and push changes to GitHub repository (Result: Success)

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





