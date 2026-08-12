# Project Work Plan (PLANS.md)
**Last Updated**: 2026-08-09

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
- [✅] Phase 45: Fix Hardcoded Model Name & Local-LLM Context-Overflow Prevention (v0.50.0)
- [✅] Phase 46: RPC Contract Consolidation & `context-tools` WIT Unification (v0.51.0)
- [✅] Phase 47: Slash Command Registry Redesign (v0.52.0)
- [✅] Phase 48: Security Policy & Extension Config Hygiene (v0.53.0)
- [✅] Phase 49: Context-Overflow Feature Completion (Manual Override & Exact Tokenization) (v0.54.0)
- [✅] Phase 50: Session Storage Operational Hygiene (v0.55.0)
- [✅] Phase 51: Advanced Context Compression Techniques (v0.56.0)
- [✅] Phase 52: Slash Command Composition Review, Role-Based Extension Lookup & Manual Compaction (v0.57.0)
- [✅] Phase 53: File-Size Limit Compliance Cleanup (v0.58.0)
- [✅] Phase 54: Real-World Dogfooding Fixes — MCP/LLM Onboarding, Doc Consolidation & Eager-Load Env-Var Bug (v0.59.0)
- [✅] Phase 55: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider` (v0.60.0)
- [✅] Phase 56: Performance Audit — wasmtime Compile Cache & MCP Tool-List Caching (v0.61.0)
- [✅] Phase 57: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem (v0.62.0)
- [✅] Phase 58: Skills — Autonomously-Discoverable Markdown Tool Definitions (v0.63.0)
- [✅] Phase 59: Documentation Audit & Codebase Synchronization (v0.64.0)
- [✅] Phase 60: Architecture Design & Codebase Alignment Re-verification (v0.65.0)
- [✅] Phase 61: Fix False-Positive [FAILED] Tool Initialization Status for Non-MCP Extensions (v0.66.0)
- [✅] Phase 62: Make LLM Thinking Process Display Independent of RAD_DEBUG (v0.67.0)
- [✅] Phase 63: Release Build, Local Installation & Push to Main (v0.68.0)
- [✅] Phase 64: Edit-Failure & Infinite-Loop Countermeasures — Shell-Quoting Corruption Fix, Consecutive-Failure Circuit Breaker & Content-Addressed `edit_file` (v0.69.0)
- [✅] Phase 64: Fix & Update scripts/build_all.sh Pipeline (v0.69.0)
- [✅] Phase 65: Comprehensive Workspace Test Suite Audit (v0.70.0)
- [✅] Phase 66: Rebuild WASM Components & Local Installation of rad Binary (v0.71.0)
- [✅] Phase 67: Spec-First Architecture Reconciliation — ARCHITECTURE.md & README.md Realigned, L3 Recovery Implemented, FS Watcher Deleted (v0.72.0)
- [✅] Phase 68: Repository Hygiene & Convention Audit — Authorship Rewrite, CI Workspace Fix, Rule Documents Corrected (v0.73.0)
- [✅] Phase 69: Microkernel Migration — Preparation & Stage 0 (v0.74.0)
- [✅] Phase 70: Microkernel Migration — Kernel Surface Alongside the Existing One (v0.75.0) (`ARCHITECTURE-NEXT.md` §9 stages 1–2)
- [🔄] Phase 71: Microkernel Migration — Extensions to Modules, One at a Time (§9 stages 3–8; stages 3–6 done)
- [ ] Phase 73: Windows Support — post-migration, once `proc-spawn` is the single point of process supervision (`ARCHITECTURE-NEXT.md` §8)
- [ ] Phase 72: Microkernel Migration — DAG/UI Extraction and Author Tooling (§9 stages 9–10)

---

## 🛠️ Short-Term Plan: Phase 71 (Extensions to Modules, One at a Time)

### 📍 Where this is

**Stage 9 is in progress**, with one unit left. Stage 8 is **paused, not
finished** — three of `rad-orchestrator`'s files cannot move until stage 9
takes the DAG and the terminal, which is a dependency running the opposite way
from §9.3's ordering.

Current shape: **1 extension** (`rad-orchestrator`) + **8 modules** (`context`,
`skills`, `mcp`, `llm-openai`, `policy`, `agent-loop`, `dag`, `ui`).

### 🗂️ What each AWU did

Completed units are indexed rather than retold. The commit message is the
record — `git show <hash>` — and every one of them carries the reasoning, the
measurements and the negative controls in full. What stays in this file is only
what a commit cannot hold: what is still open, and what the next session needs
before it reads any code.

**Stage 7 — `security-guard` → `policy` (complete)**

| AWU | Commit | What |
|---|---|---|
| 970 | `169c919` | refactor(wasm): delete the three verification sites no guest reaches |
| 971 | `5cfbfd0` | feat(modules): policy — the blocklist as a module |
| 972 | `fe0db44` | feat(modules): mcp asks policy before running a tool |
| 973 | `bc59338` | refactor: delete ext/security-guard and the verification machinery |
| 974 | `9bbc798` | docs: state what no policy in rad defends against |
| 975 | `606348c` | refactor(modules): drop mcp's unsafe Send claim |
| 976 | `f9a6f4d` | docs: record the config cleanup and correct AWU 973's finding |

**Stage 8 — `rad-orchestrator` → `agent-loop` (paused)**

| AWU | Commit | What |
|---|---|---|
| 977 | `b285055` | test(kernel): demonstrate the cross-thread lock-order deadlock |
| 978 | `696d096` | feat(kernel): drive drain_posts in production |
| 979 | `f84a887` | feat(modules): agent-loop — the event intake |
| 980 | `8121c6a` | feat(modules): agent-loop — the pure core of llm.rs |
| 981 | `c0dc25b` | test(modules): verify agent-loop's message list against the extension |
| 982 | `b4555e4` | feat(kernel): kernel.dag, so a module can read the conversation |
| 983 | `f63d46b` | feat(modules): agent-loop — the digest, and the gap the differential missed |
| 984 | `27bc6ab` | feat: route the extension's message assembly through agent-loop |

Approach **(C)**, chosen before any code: drive `drain_posts` in production and
route events through `post`, leaving §3.6.1's async wasmtime and per-module
Stores for later. AWU 979's original scope ("`llm-openai` from pull to push")
was **mis-scoped and abandoned** — the polling thread is not part of the
lock-order hazard, and there was no `post` target until `agent-loop` existed.

**Stage 9 — `dag` / `ui-repl` → modules (in progress)**

| AWU | Commit | What |
|---|---|---|
| 985 | `a488ba6` | feat(modules): dag — the conversation graph as a module |
| 986 | `49d4de8` | feat: the host writes the conversation through modules/dag |
| 987 | `d805ba0` | feat(modules): ui — the terminal's output half |
| 988 | `2aba526` | fix(dag): route rollback and session reset through the module |
| 989 | `d5e21ee` | refactor: one read path and one write path for the conversation |

Decision (A) and its reasoning now live in `ARCHITECTURE-NEXT.md` §9.3, and the
lock-order invariant they rest on in §3.6.9 — both are design, not history.

### 🔜 Next: AWU 990 — make the `dag` module required

Delete `Orchestrator::dag` and `DagSubsystemImpl::dag`, the last two copies of
the conversation the host keeps. AWU 989 already routed every host reader and
writer through `Orchestrator::conversation()` or the module, so what is left is
the deletion itself and its blast radius:

- **25 `Orchestrator::new` call sites** and **9 `DagSubsystemImpl` sites**
- 19 test files touch the DAG; each will need the module in its config, which
  couples them to the `wasm32-wasip2` build
- `models/src/dag.rs` stays — the module depends on it (AWU 988's dedup), so it
  is shared, not duplicated

Do it in steps that each leave the suite green, not as one scripted edit: the
regex-breaks-a-neighbouring-block accident happened three times in stages 3, 5
and 6, and the rule since is that a bulk edit carries asserts for what it removed.

### ⚠️ Still open

- **Stage 8's three files** (`orchestrator.rs`, `runner/done.rs`,
  `reasoning.rs`) need `WriteStdout` ×22, DAG writes ×10 and `CompleteTask` ×5.
  The terminal half now exists (`modules/ui`); the DAG half lands with AWU 990.
- **`ext/rad-orchestrator` has two `unsafe` blocks** in
  `orchestrator/reasoning.rs`. `modules/` has none.
- **`execute_tool` and `execute_tool_unverified`** (`src/wasm/imports_tool.rs`)
  are dead — probed in AWU 972 — and stay only because `wit/rad.wit` declares
  `execute-tool`. They go with the extension world.
- **`report_tool_inventory`** (`src/orchestrator/runner/runtimes.rs`) is
  uncovered; nothing asserts on its `[OK]`/`[FAILED]` wording.
- **`runtimes/tests.rs`'s 53-line test function** is over CODING.md §2's
  40-line rule, left alone deliberately.
- **`set_node_semantic_references`** (`models/src/dag.rs`) has no production
  caller — `tests/repo_map_tests.rs` alone keeps it alive.
- **Config-file cleanup stays deferred** until the migration finishes, with the
  exception already taken: `~/.rad/config.json` was brought current in AWU 976
  and 987 because it had stopped booting.

### 📋 Queued for when the migration finishes

**Merge `ARCHITECTURE.md` and `ARCHITECTURE-NEXT.md`, and fold still-current
rationale out of `PLANS-ARCHIVE.md` into the result.** One pass, not three:
once the migration lands, "NEXT" *is* the architecture, and deciding what each
archived decision still explains is the same judgement as deciding what belongs
in the merged document.

Deliberately after, not before. Stages 8 and 9 still have to delete
`rad-orchestrator`, the old WIT worlds and the old RPC surface — so a good part
of what `PLANS-ARCHIVE.md` explains is about to stop being current. Extracting
it now would mean moving rationale for things scheduled for deletion.

Two measurements the pass should not re-derive:

- **`PLANS-ARCHIVE.md` cannot be dropped in favour of git.** 31 of its 45 AWUs
  have no commit naming them — the `(AWU nnn)` subject convention only starts
  around AWU 948. Unlike the Phase 71 records, deleting these would lose them.
- **It is not a straight move into `ARCHITECTURE.md` either.** Most of it
  concerns components that no longer exist (`llm-connector`,
  `mcp-tool-provider`, `security-guard`, `skill-tool-provider`) or verification
  results true only at the time. `ARCHITECTURE.md` describes what *is*;
  separating the two needs the 1,118 lines read and judged one entry at a time.

### 📌 State at the end of stage 7

- **1 extension** (`rad-orchestrator`) + **5 modules** (`context`, `skills`,
  `mcp`, `llm-openai`, `policy`), plus four `ship = false` test fixtures
  (`echo`, `relay`, `spawn`, `net`).
- 269 passed / 0 failed. Clippy clean on native and `wasm32-wasip2`.
- **Policy is one hook, inside `modules/mcp`.** The host models no policy at
  all: no `verify_rpc_exclude`, no `verify_rpc`, no `security` role, no
  `verify-rpc` anywhere in the WIT.
- `modules/` contains no `unsafe`.

Carried forward, none of it blocking:

- **The known flake below is still unreproduced.**
- **Cross-thread lock ordering is unaddressed** (AWU 968), and as of AWU 977 it
  is **demonstrated rather than theoretical** — see
  `tests/kernel_lock_order_tests.rs`. The old note said no module pair could do
  it; that was true only because no second forwarding fixture existed.
- **`ext/rad-orchestrator` has two `unsafe` blocks**
  (`orchestrator/reasoning.rs`). That extension goes in stage 8.
- **`report_tool_inventory`** (`src/orchestrator/runner/runtimes.rs`) is still
  uncovered.
- **`runtimes/tests.rs`'s 53-line test function** is still over CODING.md §2's
  40-line rule, still left alone deliberately.
- **`execute_tool` and `execute_tool_unverified`** (`src/wasm/imports_tool.rs`)
  are dead — probed in AWU 972 — and stay only because `wit/rad.wit` declares
  `execute-tool`. They go with the extension world in stage 8.

### ⚠️ Known flake (unresolved)
One `cargo test --workspace` run during AWU 963 reported `122 passed / 1 failed`;
cargo's fail-fast truncated the run, so 122 is a partial count. It did not
reproduce across five subsequent full runs, and the failing test's name was not
captured. Recorded so a recurrence is recognised rather than investigated from
scratch.

### 📚 Earlier phases

Phase 70 and everything before it moved to `PLANS-ARCHIVE.md` when this file
passed 2,400 lines. Nothing in the migration has needed to consult them — but
they are not redundant with git: 31 of their 45 AWUs predate the `(AWU nnn)`
commit-subject convention and exist nowhere else. See the queued documentation
pass above.

