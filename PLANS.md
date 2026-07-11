# Project Work Plan (PLANS.md)

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 1: Framework Setup (Implement 6-file governance system in root)
- [🔄] Phase 2: Technical Standardization (Define Architecture & Coding standards)
- [ ] Phase 3: Operational Transition (Standardize all tasks into AWUs)

---

## 🛠️ Short-Term Plan: Phase 2 (Technical Standardization)

### 💡 Current AWU Status
- [✅] AWU-1: [Completed] Phase 1: Framework Setup (Governance Hub Implementation)
- [🔄] AWU-2: [In Progress] Audit and refine existing Technical Policies (`ARCHITECTURE.md`, `CODING.md`)
- [✅] AWU-2.1: [Completed] Define testing standards via `TESTING.md`
- [ ] AWU-3: [Todo] Standardize project structure and dependency usage
- [✅] AWU-2.7: [Completed] Fix compilation errors in WASM/RPC layers (enum mismatch). Result: Resolved non-exhaustive pattern matches in bindings.rs.

### 📝 AWU Details

#### AWU-1: Phase 1 Completion
- **Objective:** Phase 1 (Framework Setup) のすべての要素を完了させ、Phase 2 へ移行する。
- **Scope:** `PLANS.md`, `AGENTS.md`
- **Definition of Done (DoD):** Phase 1 のすべての AWU が `[✅]` となり、`AGENTS.md` が実装されていること。
- **Result:** Phase 1 の全タスクが完了。

#### AWU-2: Audit and refine existing Technical Policies
- **Objective:** `ARCHITECTURE.md` および `CODING.md` が現在のプロジェクトの実態と完全に一致しているか監査し、必要に応じて洗練させる。
- **Scope:** `ARCHITECTURE.md`, `CODING.md`
- **Definition of Done (DoD):** 両ファイルが現在のプロジェクト構造・規約を正確に反映しており、矛盾がないこと。
- **Result:**
    - **Audit Findings (AWU-2.2):**
        - **Violations of `CODING.md` (Function Length):** `src/wasm/rpc.rs:execute_rpc_command` exceeds 40 lines.
        - **Violations of `CODING.md` (No unwrap/expect):**
            - `src/wasm/rpc.rs` (Mutex locks on MCP servers)
            - `src/terminal.rs` (Mutex locks on state/buffer)
            - `src/fs.rs` (Mutex locks on allow-lists)
            - `src/orchestrator/runner.rs` (Mutex locks on config/session)
            - `src/repo_map.rs` (Parser usage)
            - `src/git.rs` (Various operations)
        - **Violations of `ARCHITECTURE.md`:** Inconsistencies between `ARCHITECTURE.md` and `wit/rad.wit` / `rad.json` identified.
    - **Remediation Roadmap:**
        - [🔄] AWU-2.3.1: Split `execute_rpc_command` in `src/wasm/rpc.rs` (Incremental Refactoring)
            - [✅] 2.3.1.1: Refactor `execute_rpc_command` into a thin dispatcher pattern. Result: Split into `RpcContext` + 6 handler fns (`handle_fs`, `handle_dag`, `handle_process`, `handle_io`, `handle_set_timeout`, `handle_meta`). All functions ≤40 lines. MCP `unwrap()` replaced with proper error handling.
            - [ ] 2.3.1.2: Extract File System operations (FileRead, FileWrite, etc.)
            - [ ] 2.3.1.3: Extract DAG operations (CreateNode, DeleteNode, etc.)
            - [ ] 2.3.1.4: Extract Process & MCP operations (SpawnBash, MCP, etc.)
            - [ ] 2.3.1.5: Extract Network & Terminal operations (OpenHttp, WriteStdout, etc.)
            - [ ] 2.3.1.6: Finalize metadata/orchestration handlers and audit.
        - [🔄] AWU-2.3.2: Replace `unwrap`/`expect` in `src/wasm/rpc.rs`.
        - [ ] AWU-2.4.1: Refactor `src/git.rs` (Fix unwrap).
        - [ ] AWU-2.4.2: Refactor `src/repo_map.rs` (Fix unwrap).
        - [ ] AWU-2.5.1: Refactor `src/terminal.rs` (Fix mutex unwrap).
        - [ ] AWU-2.5.2: Refactor `src/fs.rs` (Fix mutex unwrap).
        - [ ] AWU-2.5.3: Refactor `src/orchestrator/runner.rs` (Fix mutex unwrap).
        - [ ] AWU-2.6: Update `ARCHITECTURE.md` (Align with WIT and `rad.json` specifications).
        - [✅] AWU-2.7: Fix compilation errors in `src/wasm/rpc.rs` and `src/wasm/bindings.rs` (RasRpcCommand mismatch).



#### AWU-2.1: Define testing standards via TESTING.md
- **Objective:** テストの階層、モックの方針、テストデータ管理、カバレッジ目標を定義する。
- **Scope:** `TESTING.md`
- **Definition of Done (DoD):** `TESTING.md` が作成され、提案された4つの基準が詳細に記述されていること。
- **Result:** `TESTING.md` を新規作成し、テストの階層化、モック、データ管理、カバレッジ目標を定義した。

#### AWU-2.7: Fix compilation errors in WASM/RPC layers
- **Objective**: Fix compilation errors in `src/wasm/rpc.rs` and `src/wasm/bindings.rs` caused by non-exhaustive pattern matching for `RasRpcCommand`.
- **Scope**: `src/wasm/rpc.rs`, `src/wasm/bindings.rs`, `models/src/lib.rs`
- **DoD**: `cargo check` passes for the project.

---
*Note: This file is the single source of truth for the project status.*
