# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-18

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 10: Codebase Refactoring & Rule Alignment (v0.15.0)
- [✅] Phase 11: Unified Error Handling Mechanism (v0.16.0)

---

## 🛠️ Short-Term Plan: Phase 11 (Unified Error Handling Mechanism)

### 💡 Current AWU Status
- [✅] AWU 605: Propose Unified Error Handling Design (Result: Design proposal approved and reflected in ARCHITECTURE.md and README.md)
- [✅] AWU 606: Implement Core UnifiedError Types & dependency (Result: thiserror/anyhow dependencies added, src/error.rs created)
- [✅] AWU 607: Migrate Subsystems to UnifiedError (Result: Config, FS, Git, HTTP subsystems migrated, tests updated and passing)
- [✅] AWU 608: Implement Rollback & Dual-Track Integration in Orchestrator (Result: JSON error serialization in Wasm boundary, snapshot/rollback, and dual-track user notification implemented and verified)

### 📝 AWU Details

#### AWU 605: Propose Unified Error Handling Design
- **Objective:** Design a unified error handling mechanism for `rad` core and extensions, using `thiserror` and `anyhow` as per CODING.md, and document the proposal in `implementation_plan.md`.
- **Scope:** `implementation_plan.md`
- **Definition of Done (DoD):** Implementation plan with proposed error types and conversion architecture created and submitted for user feedback.

#### AWU 606: Implement Core UnifiedError Types & dependency
- **Objective:** Add `thiserror` and `anyhow` dependencies to Cargo.toml, and create `src/error.rs` defining the `UnifiedError` structure and its JSON serialization.
- **Scope:** `Cargo.toml`, `src/error.rs`, `src/lib.rs`
- **Definition of Done (DoD):** `src/error.rs` compiles with zero warnings under clippy.

#### AWU 607: Migrate Subsystems to UnifiedError
- **Objective:** Refactor key host subsystems (Config, FS, Git, HTTP) to return `Result<T, UnifiedError>` instead of `Result<T, String>`.
- **Scope:** `src/config.rs`, `src/fs.rs`, `src/git.rs`, `src/http.rs`
- **Definition of Done (DoD):** Host library compiles and existing unit tests pass.

#### AWU 608: Implement Rollback & Dual-Track Integration in Orchestrator
- **Objective:** Update the Wasm host RPC bridge and orchestrator extension loop to process the JSON-serialized error payloads, trigger file rollback on L2, and print clean semantic output to the user.
- **Scope:** `src/wasm/rpc.rs`, `ext/openai-orchestrator/src/orchestrator.rs`
- **Definition of Done (DoD):** Full integration test flow compiles and passes.
