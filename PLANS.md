# Project Work Plan (PLANS.md)
**Last Updated**: 2026-07-18

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 10: Codebase Refactoring & Rule Alignment (v0.15.0)
- [✅] Phase 11: Unified Error Handling Mechanism (v0.16.0)
- [✅] Phase 12: Codebase Verification & Integrity Audit (v0.17.0)
- [🔄] Phase 13: Release Build, Local Installation & Push (v0.18.0)

---

## 🛠️ Short-Term Plan: Phase 13 (Release Build, Local Installation & Push)

### 💡 Current AWU Status
- [🔄] AWU 612: Execute Release Build and Local Installation
- [ ] AWU 613: Push commits to remote git repository

### 📝 AWU Details

#### AWU 612: Execute Release Build and Local Installation
- **Objective:** Build the rad runtime binary in release mode and install it to the local system.
- **Scope:** Command line / terminal
- **Definition of Done (DoD):** Release binary successfully compiled and `rad` installed locally (verifiable via `which rad` or similar).

#### AWU 613: Push commits to remote git repository
- **Objective:** Stage, commit doc/format adjustments, and push changes to the remote git repository.
- **Scope:** git repository
- **Definition of Done (DoD):** `git push` runs successfully with zero errors.
- **Result:** Reviewed README.md and identified missing slash commands and quick start execution commands. Updated documentation to list all 10 interactive slash commands, added binary execution command to Quick Start, and linked CONFIG.md for advanced parameters and credential handling.
- **Result:** Checked configuration schemas and RPC commands. Discovered minor discrepancies in `RasCoreEvent`'s fields (e.g. `error` vs `message`, `i32` vs `String` for `pgid`) within `ARCHITECTURE.md` and successfully aligned it with `models/src/lib.rs` and `wit/rad.wit` definitions.

