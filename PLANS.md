# Project Work Plan (PLANS.md)
**Last Updated**: 2026-08-07

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
- [🔄] Phase 69: Microkernel Migration — Preparation & Stage 0
- [ ] Phase 70: Microkernel Migration — Kernel & SDK (`ARCHITECTURE-NEXT.md` §9 stages 1–2)
- [ ] Phase 71: Microkernel Migration — Module Porting (§9 stages 3–5)
- [ ] Phase 72: Microkernel Migration — Remaining Modules & Wasmtime-Only Kernel (§9 stages 6–8)

---

## 🛠️ Short-Term Plan: Phase 69 (Microkernel Migration — Preparation & Stage 0)

**Direction**: `ARCHITECTURE-NEXT.md` defines the target: the Core becomes a Wasm
runtime and dispatcher, and every capability becomes a module above it. The
contract splits in two — typed syscalls that may only grow by new functions, and
an opaque `dispatch(target, method, payload)` whose type never changes. This is a
rebuild, not a refactor: stages 1–8 run in a new directory alongside the current
implementation, which stays working until stage 5.

**Do not treat `ARCHITECTURE-NEXT.md` as current.** Nothing in it is implemented.

### 💡 Current AWU Status
- [ ] AWU 946: Verify CI is green after the workspace fix
- [ ] AWU 947: Settle `net-open` vs `wasi:http` (blocks `wit/kernel.wit`)
- [ ] AWU 948: Stage 0 — dialect table in `ext/llm-connector`

### 📝 AWU Details

#### AWU 946: Verify CI is green after the workspace fix
- **Objective**: Establish a trustworthy baseline before a long parallel rebuild.
- **Context**: CI has never actually run `clippy`/`test` — the `check-secrets` job
  was failing on a transient `actions/checkout` resolution error, which skips
  `build-and-test`. Phase 68 fixed what CI runs; that it passes is unconfirmed.
- **DoD**: A green run on `main` covering all three OSes plus the wasm job.

#### AWU 947: Settle `net-open` vs `wasi:http`
- **Objective**: Decide whether the kernel needs a custom HTTP syscall at all.
- **Context**: `ARCHITECTURE-NEXT.md` §3.1 lists three syscalls and flags this one
  as unresolved. `wasi:http`'s outgoing-handler may cover it, which would leave
  two syscalls and drop `resource stream`. **This determines the content of
  `wit/kernel.wit`, so it must be settled before stage 1 begins.**
- **DoD**: A measured answer on whether `wasi:http` can carry SSE token streaming
  at acceptable latency, recorded in §3.1.

#### AWU 948: Stage 0 — dialect table in `ext/llm-connector`
- **Objective**: `const Dialect` table plus `LlmEndpointProfile.dialect`, bringing
  Gemini and Azure into range.
- **Scope**: `ext/llm-connector/src/`, `wit/connector/llm-connector.wit`,
  `src/config.rs`, `src/command/llm/`.
- **Context**: Self-contained in the connector's own WIT package, so it does not
  touch the shared `wit/rad.wit`. Independent of every other decision here and
  ports to the new design unchanged — the one item that cannot become rework.
- **DoD**: Existing local profiles behave identically (regression-tested); a
  Gemini or Azure profile resolves to the correct URL and auth header.

---

## 🛠️ Short-Term Plan: Phase 68 (Repository Hygiene & Convention Audit)

### 💡 Current AWU Status
- [✅] AWU 943: Rewrite commit authorship across all 195 commits (Result: Success — `git filter-repo`, repo deleted and recreated so old SHAs are unreachable)
- [✅] AWU 944: Fix CI to run the whole workspace (Result: Success — `--all-targets` selected only the root package, skipping 52 of 149 tests; cleared 11 latent `needless_update` errors and split two files over the 300-line limit)
- [✅] AWU 945: Audit and correct the convention documents (Result: Success — TESTING.md's 85% coverage target had no measurement tooling behind it; CODING.md mandated the same `--all-targets` trap and banned an `#[allow]` that generated code requires; AGENTS.md's governance map omitted over half the documents)

---

## 🛠️ Short-Term Plan: Phase 67

### 💡 Current AWU Status
- [✅] AWU 942: Spec-First Architecture Reconciliation (Result: Success)

### 📝 AWU Details

#### AWU 942: Spec-First Architecture Reconciliation (ARCHITECTURE.md & README.md)
- **Trigger**: User asked for a refactoring pass beginning with a fresh comparison of `ARCHITECTURE.md` against the actual code. Two prior audits had covered adjacent ground (AWU 934 documentation alignment, AWU 935 architecture re-verification), but both compared *listings* — enum variants, RPC command tables, file paths. This pass instead checked the document's **behavioral claims**, which is where the surviving defects were: every finding below describes a mechanism the document asserted but the code never performed, and none of them were detectable by diffing symbol lists.
- **Process**: user directed a spec-first flow — write `ARCHITECTURE.md` as the intended design first, then move code to match — and explicitly asked for the *correct* design even where implementation cost was higher. Three findings were genuine forks (implement the doc vs. correct the doc); the rest were unambiguous documentation defects. Each fork was decided on architectural merit rather than cost, which produced a split verdict rather than a blanket "implement everything".
- **Finding 1 — §5.3 Slash Commands was entirely fictional (doc corrected)**: the section claimed the Core "simply passes the text event, and the Extension handles parsing and execution control", with a sequence diagram showing `HumanInputReceived{"/rollback node_a1b2"}` being delivered to an extension that parses the leading `/`. No such flow exists: `process_input` (`src/main.rs`) resolves commands entirely host-side through the static `CommandSpec` registry (`src/command.rs`) → `cmd_rollback` (`src/command/handlers.rs`) → `orchestrator.rollback()`. **The host-side design is the correct one and was kept**: a command like `/rollback` or `/reload` acts *on* the extension runtimes (rewinding the DAG, discarding cached Wasm instances), so it cannot be implemented by the extensions it manipulates. The section was rewritten to document the real four-tier resolution order (`!`-shell → built-in registry → markdown templates → agent-task fallthrough) and the reason it must be host-side.
- **Finding 2 — L3 recovery was specified but unimplemented, and classified in a layer where it can never be observed (implemented)**: the doc promised that on context exhaustion the Orchestrator would "re-invoke `context-tools`'s compaction … then reset state". The actual L3 arm (`done.rs`'s `handle_l2_l3_error`) only prints a message. Worse, that classifier runs on the **tool-execution** path, whereas context exhaustion is reported by the backend rejecting a request — it arrives on the **LLM response** path (`LlmConnectorEvent` with an error payload) and could never reach the L3 arm at all. This was judged a real correctness gap rather than aspirational prose, because it is the designed complement to an earlier decision: Phase 49-2 deliberately declined exact tokenization, leaving budgeting on a `chars/4` approximation, on the reasoning that the approximation errs toward under-use. A reactive backstop for the cases where it errs the other way (dense code, CJK) is exactly what makes that trade-off sound. **Implemented** as a new `context_recovery` module owning the policy: narrow case-insensitive matching of context-exhaustion phrasings (deliberately excluding generic words like "exceeds"/"limit" that also appear in rate-limit and output-token errors, since a false positive burns retries on a permanent failure), a compounding 60% budget backoff saturating at 1% rather than 0 (a zero budget would make compaction discard everything), and a hard cap of 2 retries. Bounding is the essential property — an unbounded shrink-and-retry would spin forever whenever a failure merely *looks* budget-related. The reduced budget is threaded through `load_messages_from_dag`'s `max_content_chars`, and reset to full on any turn that completes without rejection so one transient over-estimate cannot starve the rest of the task.
- **Finding 3 — FS watcher was unreachable in production (deleted)**: §2.1 listed filesystem change detection as a tracked subsystem state. `FsWatcher` (`src/fs/watcher.rs`, `notify`-based) existed and worked, but was instantiated **only by its own tests** — and even had it been wired up, `handle_event` matches `HumanInputReceived`/`LlmConnectorEvent`/`Rehydrate` and drops everything else through `_ => Ok(())`, so `FileChanged` had no consumer either. Inventing a consumer now would be speculative infrastructure with no demand (the Phase 47-4 caution), and the concrete motivation one might reach for — an agent editing against externally-stale content — is already handled correctly by content-addressed edits failing loudly on mismatch (AWU 940). **Deleted end-to-end** following the AWU 932 precedent: `src/fs/watcher.rs` + tests, `RasCoreEvent::FileChanged`, the `file-changed` WIT variant and `file-change-info` record, both conversion arms, and the now-unused `notify` dependency.
- **Finding 4 — `execute-tool`'s gating was overstated (doc corrected)**: §1.1 listed `open-file`/`open-process`/`execute-tool` together as passing through a gateway enforcing `rad.json` whitelists. `imports_tool.rs` calls only `verify_rpc_exclude` (the security-guard hook), never `check_permissions`. **Not "fixed" in code deliberately**: `PermissionConfig` has no dimension describing which tools may be invoked, so a mask check on `ExecuteTool` would fall through `_ => Ok(())` — structurally vacuous. Making it meaningful would require inventing a per-extension tool allowlist, which is a new feature, not a reconciliation. The section now documents the real two-layer model: the security-guard hook gates the call, and the tool's physical side-effects are masked one layer down when the tool-provider extension performs the actual `file-write`/`spawn-bash-process` RPC against *its own* permissions.
- **Findings 5–8 — documentation-only defects**: §3.1's `RasCoreEvent` listing omitted `LlmConnectorEvent` (the Orchestrator's *primary* input during a turn); §1.2 claimed "count-based windowing only", contradicting §1.3 and the actual `optimize` pipeline (stale tool-result clearing → count- *and* size-bounded windowing → relevance-based retention); §5.5 named a nonexistent RPC `request_human_approval` (real: `ask-human-approval`), while §3.2.3 listed it correctly — an internal contradiction; §5.4's intro still asserted every tool comes from MCP servers, contradicting §5.4.1 immediately below it after Skills landed. Also corrected two stale *code* comments found in passing, both claiming compaction is count-based-only (`llm.rs`, `context-tools/src/lib.rs`) — the same drift as §1.2, in the code rather than the doc. The tool-failure circuit breaker (AWU 939) was undocumented and is now specified as §5.1.3 alongside L3 as §5.1.2, since the two are the same architectural pattern: converting a potentially infinite retry into a bounded one that terminates with a clear diagnosis.
- **Scope**: `ARCHITECTURE.md`, `ext/rad-orchestrator/src/context_recovery.rs` (new), `ext/rad-orchestrator/src/context_recovery/tests.rs` (new), `ext/rad-orchestrator/src/{lib,types,llm,orchestrator}.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `ext/context-tools/src/lib.rs`, `src/fs.rs`, `src/fs/watcher.rs` (deleted), `models/src/lib.rs`, `wit/rad.wit`, `src/wasm/bindings_event.rs`, `ext/rad-orchestrator/src/conv/event.rs`, `Cargo.toml`.
- **Finding 9 — post-implementation review caught a bug in this AWU's own L3 code (fixed)**: a second verification pass over the just-written implementation found that the retry path never cleared the per-turn streaming buffers. A rejected attempt can leave partial text in `state.assistant`, buffered reasoning, half-assembled `tool_calls` deltas, and `is_reasoning = true`; because a retry re-runs the *whole* turn, the retry's tokens would append to that residue and `handle_done` would write one corrupted assistant message to the DAG — with tool-call deltas that never received closing arguments. Fixed by clearing all four in the `Retry` branch of `plan_context_recovery`, where the decision to retry is committed under the same `STATE` lock. Notably the *original* (unimplemented) spec text said "…then reset state", a clause dismissed as vague during Finding 2's design; it was in fact naming this exact requirement. §5.1.2 now states the buffer reset as a numbered step rather than leaving it implicit.
- **Finding 10 — the WIT contract is duplicated with no sync mechanism (guard added)**: `wit/rad.wit` is copied verbatim into `templates/rust/wit/rad.wit` and `templates/go/wit/rad.wit` so scaffolded extensions compile standalone. Nothing keeps them in sync, and Finding 3's `file-changed` removal silently desynced both — the *same* class of breakage AWU 934 had already repaired once (for `spawn-mcp-server`). Two occurrences on consecutive WIT edits makes the duplication itself the defect, not the individual misses. Copies re-synced, and `scripts/build_all.sh` gained a pre-flight `diff` gate that fails the build with the exact `cp` fix-up command. The guard was verified in both directions: passing on synced copies, and correctly failing on a deliberately introduced drift. (The many `rad.wit` copies under `.rad/snapshots/` are frozen rollback snapshots and are intentionally excluded — they *should* preserve their historical contents.)
- **Finding 11 — `README.md` audited on the same basis; its contributor instructions were actively harmful (fixed)**: the same behavioral-claim check was applied to `README.md`, and the worst defect was in the instructions contributors are told to follow before submitting code. §2.3/§4.4 both prescribed `cargo test -- --test-threads=1`; measured, a bare `cargo test` runs **97 tests versus 149 for `cargo test --workspace`** — it silently skips every test in `ext/*` and `models/`, about a third of the suite, so anyone following the README could believe a green run covered code it never touched. The `--test-threads=1` requirement was itself stale: env-var-sharing tests now serialize themselves via an in-file `TEST_MUTEX` (6 files), and `scripts/build_all.sh` — the actual project standard — has long run plain parallel `cargo test --workspace`, directly contradicting the README. §4.4's `cargo clippy --all-targets` was likewise weaker than the enforced `cargo clippy --workspace -- -D warnings` (no `-D warnings` means warnings don't fail). Both sections now match `build_all.sh`, with a note on why `--workspace` is mandatory and an added `wasm32-wasip2` lint step, since a native-only check misses target-specific breakage in the extensions.
- **Finding 12 — remaining `README.md` inaccuracies (fixed)**: prerequisites claimed Rust 1.75 while the whole workspace is on edition 2024 (needs 1.85+); the clone URL was still the `yourusername` placeholder rather than the real remote; §4.1 described "isolated micro-extensions (LLM Orchestrator, Security Guard, Tool Provider)" — three — contradicting §1's correct list of six in the same document; §4.2's L3 strategy still read "run `context-tools` pruning/summarization and reset", describing summarization that was never implemented and predating Finding 2's actual bounded-backoff design; §4.3 claimed `cargo build` "copies" the artifact to the configured location, which it does not (only `build_all.sh` installs), and omitted the `/reload` step needed for a running session to pick up a replaced `.wasm`. §4.3 now also points at the WIT-mirroring requirement from Finding 10. Verified as correct and left unchanged: the §3.1 slash-command list (all 10 match the `CommandSpec` registry — an initial mismatch was traced to a flawed extraction regex on my side, not a doc defect) and §3.2's "6 extensions" count.
- **Finding 13 — the onboarding path dead-ended (new `README.md` §3.3 added)**: §3.2 correctly warned that "without at least one MCP server configured, the agent has zero tools and can't act on anything" — and then never told the reader where to obtain one, leaving a fresh install with no route to a working agent. Added §3.3 "Companion MCP Servers" documenting the two servers `rad` is dogfooded against, [`core-utilities-mcp`](https://github.com/akahmys/core-utilities-mcp) (15 tools: filesystem, shell, structured data) and [`web-access-mcp`](https://github.com/akahmys/web-access-mcp) (4 tools: fetch/search with headless-Chromium fallback), with install commands and the `config.mcp_servers` snippet. Framed explicitly as "any stdio MCP server works — `rad` is not coupled to a particular one" so the section documents an example rather than implying a hard dependency, consistent with §5.4's provider-agnostic invariant. Every claim was verified against the sources rather than written from memory: tool counts (15 and 4) counted from each server's own tool table and cross-checked against a live run reporting `Verified 19 tools`; repository URLs checked against the actual `git remote`s rather than assumed; installed binary paths confirmed present at `~/.cargo/bin/`; the `edit_file` description confirmed content-addressed (`old_string`, no `start_line`) per AWU 940; and the Chrome/Chromium prerequisite taken from `web-access-mcp`'s own documented requirements.
- **Finding 14 — `README.md` prose pass surfaced a false quantitative claim (fixed)**: user asked for an editorial pass, specifically calling the opening redundant and asking for factual rather than promotional phrasing. The intro carried two competing definitions in consecutive paragraphs ("minimal, ultra-fast agent-oriented shell runtime" then "coding agent runtime inspired by pi-coding-agent"); collapsed to one sentence naming what `rad` is, plus one describing the mechanism/policy split in plain terms. Removing marketing adjectives then required checking whether the measurable ones were even true — and **"a tiny memory footprint of just a few megabytes" was false: the installed binary is 24 MB** (extensions add 2.8 MB). "Starts instantly" was likewise unmeasured; actual cold start including extension load is ~0.4 s. Both were long-standing claims inherited from the original README, and an earlier draft of this pass had preserved the size figure verbatim while rewriting around it — a reminder that copy-editing propagates unverified assertions unless each one is independently checked. The bullet was replaced with a factual statement about the Core/extension split. Other unsubstantiated qualifiers removed throughout ("instantly", "simply", "robust", "Zero Dependencies" — the last also contradicted the design, since the binary does nothing without its `.wasm` extensions).
- **Finding 15 — `/reload`'s documented behavior omitted its most useful effect (fixed)**: §3.1 described `/reload` as "dynamically reloads the configuration file". Reading `Orchestrator::reload()` shows it also reapplies sandbox permissions and — the operationally important part — calls `wasm_runtime.lock().clear()`, dropping cached runtimes so the next task re-loads extensions from disk. That is precisely what makes a rebuilt `.wasm` take effect without restarting, a workflow §4.3 now depends on. Description corrected, and the §3.1 command list made grammatically consistent (two of ten entries were third-person amid eight imperatives). Every command description was re-checked against its handler: `/session`'s reported fields, `/new`'s session-ID rotation plus DAG clear, and `/compact`'s persist-versus-ephemeral distinction all verified against the code rather than trusted from the existing text.
- **Definition of Done (DoD)**: `ARCHITECTURE.md` and `README.md` contain no claim the code does not perform, and every command they instruct a contributor to run is the one the project actually enforces; all tests + Clippy (`-D warnings`) pass on native and `wasm32-wasip2`; CODING.md's 300-line file limit respected; no regression in a real `rad` run.
- **Result**: Success. 7 new unit tests in `context_recovery` covering backend phrasing recognition, false-positive rejection (rate limits, output-token limits, connection errors), compounding backoff, and the non-zero saturation floor. Full workspace suite green on both targets (the one removed test is the deleted watcher's own). `llm.rs` had crossed to 301 lines and was brought back to 299 by rewriting — not merely shortening — the stale comment flagged above. Verified against the real installed binary and a live local LLM: multi-turn tool loop, compaction, and thinking display all unaffected. A final mechanical re-verification confirms every remaining `ARCHITECTURE.md` claim against the code: the §3.1 event listing and §3.2.3 RPC listing both diff clean against `models/src/lib.rs` and `wit/rad.wit`, §5.5's RPC name resolves, §5.1.2's specified behaviors are present, and §2.1's "no FS events" claim holds with zero `FileChanged` references left in `src`/`models`/`wit`.
- **Known pre-existing flakiness (investigated, not introduced here, left as-is)**: `src/http/tests.rs`'s three streaming tests failed once during a full-workspace run immediately after a large rebuild, then passed 3/3 in isolation, 2/2 parallel, 2/2 single-threaded, and 2/2 in subsequent full-workspace runs. Root cause is their tight 2-second `recv_timeout` budget for connect-plus-stream, which can be exceeded when many test binaries and tokio runtimes contend for a loaded machine. Ruled out as a regression from this AWU: a `tokio` feature-unification hypothesis (that dropping `notify` had disabled `tokio/net`) was tested directly with `cargo tree -e features` and disproved — `net` is enabled identically with and without `notify`. Raising the timeout is a reasonable follow-up but was left out of scope rather than changed on a hunch.

---

## 🛠️ Short-Term Plan: Phase 66

### 💡 Current AWU Status
- [✅] AWU 941: Rebuild WASM Component Extensions and Reinstall `rad` Binary Locally (Result: Success)

### 📝 AWU Details

#### AWU 941: Rebuild WASM Component Extensions and Reinstall `rad` Binary Locally
- **Trigger**: User requested re-installing `rad` locally (`ローカルにインストールし直して`).
- **Scope**: `~/.rad/wasm/`, `~/.cargo/bin/rad`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Run `./scripts/build_all.sh` with network bypass (`BypassSandbox: true`) so all verification steps (WASM compilation, formatting, license checks, secret scanning, unit/integration tests, Clippy audit, and `cargo install --path .`) succeed cleanly.
  - Verify that all 6 WASM extension components are copied to `~/.rad/wasm/`.
  - Verify that `rad` binary is installed cleanly at `~/.cargo/bin/rad`.
- **Result**: Success. Executed `./scripts/build_all.sh` with network bypass. All 6 WASM extension components (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`) built and deployed to `~/.rad/wasm/`, all audit checks and workspace tests passed, and `rad` binary installed successfully at `~/.cargo/bin/rad`.

---

## 🛠️ Short-Term Plan: Phase 65

### 💡 Current AWU Status
- [✅] AWU 940: Comprehensive Audit of Unit and Integration Test Coverage across Core & WASM Extensions (Result: Success)

### 📝 AWU Details

#### AWU 940: Comprehensive Audit of Unit and Integration Test Coverage across Core & WASM Extensions
- **Trigger**: User requested an audit to verify if test code in the workspace has any gaps, excess, or missing scenarios.
- **Scope**: `tests/`, `src/`, `ext/`, `models/`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Audit all test files in `tests/` and companion test modules in `src/` and `ext/`.
  - Check coverage for Core subsystems (FS, Process, DAG, Network, Wasm loader, RPC meta, Permissions, Commands, Config), WASM extensions (rad-orchestrator, llm-connector, security-guard, mcp-tool-provider, skill-tool-provider, context-tools), and IPC contract (`models`).
  - Identify any redundant/dead tests (excess) or missing critical test scenarios (deficiency/gaps).
  - Run test suite and present audit findings clearly to the user.
- **Result**: Success. Audited total of 17 integration test files in `tests/` and 12 companion test modules across `src/`, `ext/`, and `models/`. All test suites are active, well-structured, compliant with the CODING.md requirement (companion test files `< 300` lines, zero `#[cfg(test)]` in production files), and 100% passing across the workspace. Zero dead/redundant legacy tests remain (cleaned up during AWU 932/933).

---

## 🛠️ Short-Term Plan: Phase 64

### 💡 Current AWU Status
- [✅] AWU 939: Fix & Update `scripts/build_all.sh` to Align with Current Project Workspace & Audit Pipeline (Result: Success)

### 📝 AWU Details

#### AWU 939: Fix & Update `scripts/build_all.sh` to Align with Current Project Workspace & Audit Pipeline
- **Trigger**: User requested updating `scripts/build_all.sh` to match current project state, components, and verification pipeline.
- **Scope**: `scripts/build_all.sh`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - `scripts/build_all.sh` builds all 6 current WASM component extensions (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`).
  - Explicitly copy only the built WASM components to `~/.rad/wasm/` and `target/wasm32-wasip2/debug/` instead of wildcards.
  - Include code audit/verification steps (`cargo fmt --check`, `python3 scripts/check_licenses.py`, `scripts/check_secrets.sh --all`).
  - Perform clean build, test (`cargo test --workspace`), clippy (`cargo clippy --workspace -- -D warnings`), and local binary installation (`cargo install --path .`).
  - Verification: `./scripts/build_all.sh` executes cleanly.
- **Result**: Success. Updated `scripts/build_all.sh` to explicitly build all 6 WASM extension crates (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`), copy specific WASM files instead of wildcards, add code formatting (`cargo fmt --check`), license audit (`python3 scripts/check_licenses.py`), and secrets/path scanning (`scripts/check_secrets.sh --all`). Verified clean end-to-end execution of `./scripts/build_all.sh`.

---

## 🛠️ Short-Term Plan: Phase 64

### 💡 Current AWU Status
- [✅] AWU 939: Edit-Failure & Infinite-Loop Countermeasures (Result: Success)

### 📝 AWU Details

#### AWU 939: Edit-Failure & Infinite-Loop Countermeasures
- **Trigger**: User reported that the LLM frequently fails at file edits and sometimes falls into infinite retry loops, and asked for countermeasures spanning both `rad` and the `core-utilities-mcp` server that provides the edit tool. Investigation found two independent layers at fault, plus (during implementation) a third, more serious pre-existing bug.
- **Rejected scope**: a per-task *total* tool-call cap. It can't distinguish a legitimately long task (a 20-file refactor makes 50+ successful calls) from a stuck one, so any threshold either kills real work or detects loops too late. Narrowed by agreement to tracking **consecutive failures of the same tool** only, which stays silent as long as progress is being made.
- **Fix 1 — shell-quoting corruption (pre-existing bug, found while testing the circuit breaker)**: `spawn_std_pipe_process` (`src/process.rs`) has a Phase-37 "fast direct binary execution" path that skips `bash -c` when the command contains no shell metacharacters — but it builds its argv with a naive `split_whitespace()`, which destroys quoting. Since every tool provider returns its result via `open_process("echo -n '<result>'")` (the only way to satisfy the WIT `execution-handle` return type), `echo` ran directly with argv `["-n", "'{\"directories\":...", ...]`, so **every MCP tool result reached the LLM wrapped in literal single quotes** (`'{"directories":...}'` instead of `{"directories":...}`). Confirmed against real transcripts from earlier in this session. Fixed by treating `'` and `"` as shell features, so quoted commands go through `bash` like redirection already did; the direct-exec optimization still applies to its actual target (MCP server binaries like `~/.cargo/bin/core-utilities-mcp`, which have no quotes). This bug would also have silently defeated the new circuit breaker, since `'Error: ...` doesn't start with `Error:`.
- **Fix 2 — consecutive-failure circuit breaker (`rad`)**: `mcp-tool-provider`'s `execute_tool` flattens `CallToolResult` to plain text and previously discarded `isError` entirely, leaving no structural failure signal. It now normalizes `isError: true` into a guaranteed `Error:` prefix (independent of any server's own wording), and `handle_l2_l3_error`'s L2 branch was aligned to the same prefix. `OrchestratorState` gained `last_tool_name`/`consecutive_tool_failures`/`max_consecutive_tool_failures`; `execute_pending_calls` (`orchestrator/runner/done.rs`) updates the streak after each result, resets it on any success or on a *different* tool failing, and at 4 consecutive same-tool failures stops the task cleanly (skips remaining calls from that turn, prints an explanation, calls `CompleteTask`) instead of triggering another LLM turn. Threshold deliberately generous — 2-3 retries while a model converges on a correct edit is normal, not stuck.
- **Fix 3 — content-addressed `edit_file` (`core-utilities-mcp`)**: the root cause of the edit failures themselves. `EditChunk { start_line, end_line, target_content, replacement_content }` required both a correct line range *and* a near-exact content match, so it broke whenever an earlier edit shifted line numbers or the model's memory of the file went stale after context compaction. Replaced with `EditChunk { old_string, new_string }`: no line numbers at all, matched by locating `old_string` in the file's current content. Uniqueness is required (0 matches → "not found, re-read the file"; 2+ → "not unique, add surrounding context"), so an ambiguous edit is refused rather than guessed at. A whitespace-tolerant line-based search is tried only if there's no exact match, covering invisible trailing-whitespace differences. All chunks are resolved and overlap-checked before anything is written, preserving the previous all-or-nothing atomicity. `replace_all` was deliberately not added — refusing ambiguity is safer than silently editing every occurrence.
- **Scope**: rad — `src/process.rs`, `ext/mcp-tool-provider/src/lib.rs`, `ext/rad-orchestrator/src/types.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `tests/circuit_breaker_tests.rs` (new). core-utilities-mcp (separate repo) — `core-utilities-mcp-lib/src/file_ops/mutate/edit.rs`, its `tests.rs`, `core-utilities-mcp-lib/src/text_ops/read.rs`, `core-utilities-mcp/src/tools.rs`, `core-utilities-mcp/tests/integration_test.rs`, `README.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass in both repos, native and `wasm32-wasip2`; both fixes verified against the real installed binaries, not just mocked tests.
- **Result**: Success. 3 new `rad` integration tests (breaker fires at exactly 4 consecutive same-tool failures leaving later turns unserved; 3 failures stay under threshold; a mid-streak success resets the counter so 6 non-consecutive failures still complete normally) driving a full `Orchestrator` + mocked LLM. 13 rewritten/added `core-utilities-mcp` unit tests covering not-found, ambiguity, disambiguation-by-context, whitespace-tolerant fallback, out-of-order and overlapping chunks, deletion via empty `new_string`, and all-or-nothing rollback. Full test suites and Clippy clean in both repos. Verified against real installed binaries: raw JSON-RPC against `core-utilities-mcp` confirmed all four `edit_file` behaviors including `isError: true` on failure (the exact signal the breaker consumes), and a real `rad` run against a real local LLM confirmed both that tool results now arrive unquoted (`{"directories":...}`) and that a full read→edit→verify cycle succeeds.

---

## 🛠️ Short-Term Plan: Phase 63

### 💡 Current AWU Status
- [✅] AWU 938: Release Build, Local Installation & Push to Main (Result: Success)

### 📝 AWU Details

#### AWU 938: Release Build, Local Installation & Push to Main
- **Trigger**: User requested local installation and pushing current changes to git remote repository.
- **Scope**: `Cargo.toml`, `PLANS.md`, Git repository.
- **Definition of Done (DoD)**:
  - Run `./scripts/build_all.sh` to compile WASM components and install `rad` locally.
  - Commit all modified files with a clear commit message.
  - Push changes to git remote (`main` branch).
- **Result**: Success. Executed `./scripts/build_all.sh` successfully, compiling all WASM extensions and installing `rad` binary to `~/.cargo/bin/rad`. Committed modified files and pushed successfully to `origin/main` (`0419abc`).

---

## 🛠️ Short-Term Plan: Phase 62

### 💡 Current AWU Status
- [✅] AWU 937: Make LLM Thinking Process Display Independent of `RAD_DEBUG` (Result: Success)

### 📝 AWU Details

#### AWU 937: Make LLM Thinking Process Display Independent of `RAD_DEBUG`
- **Trigger**: User requested displaying LLM thinking/reasoning process text without requiring `RAD_DEBUG=1`.
- **Scope**: `ext/rad-orchestrator/src/orchestrator/reasoning.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Thinking process display is enabled by default independently of `RAD_DEBUG`.
  - Display can be suppressed if explicitly set via `RAD_SHOW_THINKING=0` or `RAD_HIDE_THINKING=1`.
  - All workspace unit tests, clippy checks (`-D warnings`), and `./scripts/build_all.sh` pass cleanly.
- **Result**: Success. Updated `thinking_enabled()` in `ext/rad-orchestrator/src/orchestrator/reasoning.rs` so that thinking traces and markers default to being shown. Added support for explicit opt-out via `RAD_SHOW_THINKING=0` or `RAD_HIDE_THINKING=1`. Added unit tests in `reasoning.rs`. Verified all workspace tests, zero-warning clippy audit, and clean `./scripts/build_all.sh` installation.

---

## 🛠️ Short-Term Plan: Phase 61

### 💡 Current AWU Status
- [✅] AWU 936: Fix False-Positive `[FAILED]` Tool Initialization Status for Non-MCP Extensions (`skill-tool-provider`) (Result: Success)

### 📝 AWU Details

#### AWU 936: Fix False-Positive `[FAILED]` Tool Initialization Status for Non-MCP Extensions (`skill-tool-provider`)
- **Trigger**: Launching `rad` without any custom skills defined in `.agents/skills` or `~/.rad/skills` outputs misleading `[FAILED] Extension 'skill-tool-provider' initialized with 0 tools. See [MCP Diagnostic] lines above for the actual cause.`, mistaking empty skill discovery for a failed MCP initialization.
- **Scope**: `src/orchestrator/runner/runtimes.rs`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - `runtimes.rs` distinguishes between `mcp-tool-provider` (where 0 tools indicates MCP server failure/misconfiguration) and other tool-provider extensions like `skill-tool-provider` (where 0 tools is a normal state when no skills are defined).
  - Non-MCP tool provider extensions initializing with 0 tools print `[OK] Extension '<name>' initialized with 0 tools` in green instead of `[FAILED]` in red with MCP diagnostic hints.
  - All workspace tests and `cargo clippy --workspace -- -D warnings` pass cleanly.
- **Result**: Success. Updated `src/orchestrator/runner/runtimes.rs` to check `ext.name == "mcp-tool-provider"` before reporting a 0-tool `[FAILED]` error with MCP diagnostics. Non-MCP tool providers like `skill-tool-provider` now output `[OK] Extension 'skill-tool-provider' initialized with 0 tools` when no local skills are present. Verified clean `cargo check --workspace`, zero-warning `cargo clippy --workspace -- -D warnings`, and full workspace test suite pass.


### 💡 Current AWU Status
- [✅] AWU 935: Comprehensive Re-verification of Basic Architecture Design & Implementation (Result: Success)

### 📝 AWU Details

#### AWU 935: Comprehensive Re-verification of Basic Architecture Design & Implementation
- **Trigger**: User requested a fundamental re-verification starting from the basic architecture design (`ARCHITECTURE.md` and related design policies).
- **Scope**: `ARCHITECTURE.md`, `wit/rad.wit`, `src/`, `ext/`, `models/`, `scripts/`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Perform thorough verification comparing basic architecture specification (`ARCHITECTURE.md`) against actual codebase implementation.
  - Run mechanical audit checks (`cargo check`, `cargo clippy --workspace -- -D warnings`, `cargo test`, `./scripts/build_all.sh`).
  - Document verification findings, structural compliance, and any discrepancies or confirming evidence in a structured report.
- **Result**: Success. Verified strict alignment between `ARCHITECTURE.md` specification and physical implementation across all 6 Wasm extension crates, Trait-based Core subsystems, IPC contract in `wit/rad.wit` (27 RPC variants, zero legacy artifacts), process group (PGID) management, single capability configuration (`~/.rad/config.json`), and error handling workflows. Mechanical audit passed: `cargo check --workspace` clean, `cargo clippy --workspace -- -D warnings` clean, full workspace test suite passed cleanly.

---

## 🔭 Phase 46–51 Scope Notes (Planned, Not Yet Started)

Captured from a 2026-07-27 design review of local-LLM context-overflow prevention, which
expanded into a broader architecture/technical-debt pass (`/llm` command design, RPC
contract duplication, `context-tools` WIT scope, slash-command extensibility). Recorded here
at roadmap granularity; each phase gets its own AWU-numbered "Short-Term Plan" section with
Trigger/Fix/Result detail once work actually starts on it.

### Phase 46: RPC Contract Consolidation & `context-tools` WIT Unification — ✅ complete
*(46-1/46-3/46-4 shipped as AWU 915; 46-2 shipped as AWU 916 — see "Short-Term Plan: Phase 46" below.)*

### Phase 47: Slash Command Registry Redesign — ✅ complete
- **Context**: A `/llm` UX review (positional-arg-only `add`, reserved subcommand names shadowing profile names, no `delete`, inconsistent help text) traced back to a structural issue in `src/command.rs` itself: `Command` enum, `CommandParser::parse`, `CommandManager::execute`, the hand-written `/help` text, and `CommandHelper`'s tab-completion list were **5** independently-maintained sources of truth for "what commands exist" (the 5th, tab-completion, didn't even know `/llm` existed — a live bug). Compared against pi-coding-agent's `registerCommand(name, {description, handler})` unified-registry design.
- **47-1 — done (AWU 917)**: Refactored `src/command.rs`'s built-ins onto a single `CommandSpec` registry generating the parser, dispatcher, `/help` text, and tab-completion from one table.
- **47-1b — done (AWU 918)**: Applied the same registry pattern recursively to `/llm`'s own subcommands, fixing the residual issues 47-1 didn't touch. Also uncovered and fixed a real bug where testing it clobbered a developer's real `~/.rad/config.json`.
- **47-2 — done (AWU 919)**: Markdown-template-based lightweight command tier — see "Short-Term Plan: Phase 47" below.
- **47-3 — done (AWU 919)**: Extension loading switched from lazy to eager-at-startup.
- **47-4 — deliberately not implemented**: Extension-provided slash commands via a WIT export. Re-evaluated after 47-2 shipped, per this item's own stated condition: no current extension (rad-orchestrator/security-guard/mcp-tool-provider/llm-connector/context-tools) has a concrete need for a UI-facing command, and markdown templates already cover the actual demand this was meant to serve (user-defined prompt shortcuts). Building the WIT plumbing now would be speculative infrastructure with no consumer — revisit only if/when a real extension needs to do more than expand into a task prompt (e.g. programmatic output bypassing the LLM).
- **47-5 — done (AWU 924, Phase 52)**: Broader review of the slash-command *composition* itself, revisited at user request once Phase 51 wrapped. See Phase 52 below.

### Phase 48: Security Policy & Extension Config Hygiene — ✅ complete
*(48-1 and 48-2 shipped together as AWU 920, since the fix for one is the mechanism for the other — see "Short-Term Plan: Phase 48" below.)*
- **48-1 — done (AWU 920)**: `ext/security-guard/src/lib.rs::verify_rpc` hardcoded demo-fixture policy (`"blocked.txt"`/`"blocked_command"` string literals) with no config-driven policy mechanism at all. Replaced with a policy fetched from the extension's own config.
- **48-2 — done (AWU 920)**: `ExtensionConfig.config: HashMap<String, serde_json::Value>` (`src/config.rs`) is now actually read at runtime — via a new `GetExtensionConfig` RPC that `security-guard` calls to resolve its own blocklist patterns.

### Phase 49: Context-Overflow Feature Completion — ✅ complete
*(49-1 shipped as AWU 921; 49-2 deliberately not implemented — see "Short-Term Plan: Phase 49" below.)*
- **49-1 — done (AWU 921)**: Manual `context_length` override for backends `detect_context_length` can't probe — shipped as `/llm context <n>` atop Phase 47's registry.
- **49-2 — deliberately not implemented**: Exact token counting via llama.cpp's `/tokenize` endpoint. Would only cover llama.cpp (Ollama has no equivalent endpoint), making context-window budgeting backend-inconsistent — some profiles precise, others still approximated — for a benefit the existing chars/4 approximation already covers safely (it reserves a 10% margin, so the failure direction is under- not over-using the budget). Also adds an HTTP round-trip to every single `optimize` call on the hot path. Revisit only if the chars/4 approximation is observed to actually cause overflow in practice.

### Phase 50: Session Storage Operational Hygiene — ✅ complete
*(50-1 shipped as AWU 922 — see "Short-Term Plan: Phase 50" below.)*
- **50-1 — done (AWU 922)**: `.rad/sessions/<timestamp>.json` had no rotation/cleanup mechanism; files accumulated unbounded (150+ observed in this repo's own `.rad/` during investigation).

### Phase 51: Advanced Context Compression Techniques — ✅ complete
*(51-1/51-2/51-3 shipped as AWU 923; 51-4 deliberately not implemented — see "Short-Term Plan: Phase 51" below.)*
- **51-1 — done (AWU 923)**: Size-aware tool-result clearing, extending `context-tools.optimize`'s existing request/response (no parallel mechanism, no new WIT field).
- **51-2 — done (AWU 923)**: Lexical relevance-based retention (keyword overlap with the goal) reinstating earlier turns windowing would otherwise drop, when size budget leaves slack.
- **51-3 — done (AWU 923)**: Deterministic structured-fact digest (files touched, commands run) extracted from tool-call metadata in `rad-orchestrator`, attached to the system prompt so it survives windowing/clearing unconditionally.
- **51-4 — deliberately not implemented**: LLM-based recursive summarization (MemGPT-style). The scope note's own framing — "last resort only" — is the reason not to build it yet: 51-1/51-2/51-3 just shipped and haven't been exercised in real usage, so their effectiveness at the stated goal (preventing local-LLM context overflow) is unproven. Adding a mechanism that consumes the same constrained local-LLM compute the user is trying to protect (a nested summarization call competing with the primary task, on the very hardware this whole effort was scoped around) is a meaningfully larger commitment than 51-1–51-3 combined, and premature before knowing whether the cheaper techniques already solve the problem. Revisit only if real usage shows the cheaper techniques still overflow.

### Phase 52: Slash Command Composition Review, Role-Based Extension Lookup & Manual Compaction — ✅ complete
*(All shipped together as AWU 924, since `/compact`'s persistence step is what motivated the role-based lookup fix — see "Short-Term Plan: Phase 52" below.)*
- **Context**: User asked to revisit the deferred 47-5 composition review now that Phase 51 shipped, this time compared directly against pi-coding-agent's built-in slash-command set (fetched from its official docs) rather than in the abstract. That comparison surfaced concrete, externally-validated findings: pi merges what rad split across `/status`+`/session` into one `/session` command; pi has no `/clear` at all; pi's session-reset command is named `/new`, not `/reset`; and pi has a manual `/compact` with no rad equivalent — directly relevant given this whole session's context-overflow focus.
- **Command changes**: `/session` and `/status` merged (removed `/status`, `/session` now shows the full former-`/status` output — matches pi's own `/session` semantics: "session file, ID, messages, tokens, and cost"). `/clear` removed (no pi precedent, and terminals already provide Ctrl+L). `/reset` renamed to `/new`.
- **Extension-provided slash commands, reconsidered and re-confirmed not needed**: designing `/compact` surfaced that its logic is reachable entirely through the *existing* `CallExtension` RPC — proof that "host command delegating to an extension via RPC" already covers the use case a dedicated extension-owned-command mechanism would exist for. 47-4's original call stands.
- **Role-based `CallExtension` lookup**: `call-extension-payload.extension-id` now resolves by the target's declared `role` (`ExtensionConfig.role`) instead of its literal name, via a new `Orchestrator::find_extension_arc_by_role` (reads the role from static config rather than locking every candidate `WasmRuntime`, avoiding the nested-lock pattern AWU 900 fixed away from). A user can now swap in their own compatible `context-tools` replacement under any name without breaking the automatic per-turn optimize call or `/compact`. `wit/rad.wit`'s payload doc comment updated; no wire-schema change.
- **`/compact`**: new host-side command — no new WIT surface, since `rad-orchestrator`'s world only exports `on-event` (no generic "invoke me now" entry point the host could call on demand). Implemented entirely in `src/command/compact.rs`: walks the DAG natively (host already holds it directly, no RPC), calls `context-tools.optimize` the same way `rad-orchestrator` does (`WasmRuntime::call_extension_method`, found via the new role-based lookup), diffs surviving vs. dropped node IDs from the response, and persists the result via `Dag::merge_nodes` — split into one `merge_nodes` call per *contiguous* dropped run (Phase 51-2's relevance retention can make the dropped set non-contiguous, and `merge_nodes` isn't designed to collapse a non-contiguous list correctly). Failures surface as an explicit message (unlike the automatic path's silent fallback, appropriate for a deliberately user-triggered action).
- **`merge_nodes` bug fix (found while building `/compact`)**: previously reassigned `current_node_id` to the new merge node *unconditionally*, even when merging a historical (non-tip) range — would have silently rewound the active conversation pointer. Fixed to mirror `delete_node`'s existing conditional-clear: only follow the pointer if it was actually pointing at one of the merged nodes. No prior consumer of `merge_nodes` existed (`/compact` is the first), so this was a safe, uncontested correction rather than a behavior change with a dependent to worry about.
- **Scope**: `src/command.rs`, `src/command/handlers.rs`, `src/command/compact.rs` (new), `src/command/compact/tests.rs` (new), `src/orchestrator/runner/runtimes.rs`, `src/orchestrator/runner/runtimes/tests.rs` (new), `src/wasm/rpc_meta.rs`, `ext/rad-orchestrator/src/llm.rs`, `models/src/dag.rs`, `src/dag/tests.rs`, `wit/rad.wit`, `tests/command_tests.rs`, `README.md`, `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. New tests: 2 `merge_nodes` `current_node_id` tests, 1 role-based-lookup test (registers a `context-tools`-role extension under a different name and confirms resolution), 3 `/compact` tests (too-few-messages no-op, missing-extension error, and a full end-to-end run through the real `context_tools.wasm` that actually reduces DAG node count). `./scripts/build_all.sh` clean end-to-end.

---

## 🛠️ Short-Term Plan: Phase 59

### 💡 Current AWU Status
- [✅] AWU 934: Documentation & Template Alignment with Current Codebase Architecture (Result: Success)

### 📝 AWU Details

#### AWU 934: Documentation & Template Alignment with Current Codebase Architecture
- **Trigger**: User requested an audit to ensure `README.md`, `ARCHITECTURE.md`, `EXTENSIONS.md`, and all project documentation are fully consistent with the codebase.
- **Audit Findings**:
  1. `README.md` §4.3 references `cd ext/openai-orchestrator` (renamed to `ext/rad-orchestrator`).
  2. `ARCHITECTURE.md` §1 diagram omits `skill-tool-provider` and `context-tools` micro-extensions; §3.2.3 `ras-rpc-command` variant listing omits `list-dir`, `get-active-llm-profile`, `get-extension-config`, `generate-llm-stream`, `call-extension`, `log-traced-event`; §4.2 example config references obsolete extension names (`openai-orchestrator`), obsolete `wasm32-wasip1` target, and obsolete `"rpc_allow"` permission.
  3. `EXTENSIONS.md` §2 claims single `rad-extension` world (now multiple worlds: `rad-extension`, `rad-orchestrator`, `rad-security-guard`, `rad-tool-provider`); §3 lists deleted variants `spawn-mcp-server / send-mcp-request` and omits newer variants; §4 lists obsolete `"rpc_allow"`; §5 step 4 recommends `wasm32-wasip1` instead of `wasm32-wasip2`.
  4. Template WIT files (`templates/rust/wit/rad.wit`, `templates/go/wit/rad.wit`) contain deleted `spawn-mcp-server` variant and lack `list-dir`, etc.
  5. Template `README.md` files (`templates/rust/README.md`, `templates/go/README.md`) reference `wasm32-wasip1` target and `"rpc_allow"`.
- **Scope**: `README.md`, `ARCHITECTURE.md`, `EXTENSIONS.md`, `templates/rust/README.md`, `templates/rust/wit/rad.wit`, `templates/go/README.md`, `templates/go/wit/rad.wit`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - All identified documentation discrepancies resolved.
  - Template `rad.wit` files synchronized with `wit/rad.wit`.
  - All workspace tests and `./scripts/build_all.sh` pass cleanly without errors or Clippy warnings.
- **Result**: Success. Updated `README.md`, `ARCHITECTURE.md`, and `EXTENSIONS.md`. Synchronized WIT contracts in `templates/rust/wit/rad.wit` and `templates/go/wit/rad.wit` with `wit/rad.wit`. Updated Rust and Go template READMEs to reference `wasm32-wasip2` and removed obsolete `rpc_allow`. Verified compilation of `templates/rust` (`cargo check --target wasm32-wasip2`), clean workspace tests (`cargo test --workspace -- --test-threads=1`), and Clippy audit (`cargo clippy --workspace -- -D warnings`).

---

## 🛠️ Short-Term Plan: Phase 58

### 💡 Current AWU Status
- [✅] AWU 933: Skills — Autonomously-Discoverable Markdown Tool Definitions (Result: Success)

### 📝 AWU Details

#### AWU 933: Skills — Autonomously-Discoverable Markdown Tool Definitions
- **Trigger**: User asked how `rad` handles Skills/Workflows (a concept from Claude Code and other agent tools) — confirmed via grep that no such feature exists, and that a *fictional* version of it was previously purged from docs in AWU 926. Follow-up discussion converged on a concrete design: unlike `.agents/commands/` (explicit `/name` invocation), Skills should be discoverable the same way any other tool is — the model sees a description in the tool list and chooses to invoke it autonomously.
- **Design pivot during planning**: the first design considered was fully host-native (`.agents/skills/` scanned directly in `src/wasm/rpc_meta.rs`'s `GetTools`/`ExecuteTool` handlers, no new WIT/RPC surface at all). The user pushed back: this would have repeated the exact anti-pattern just removed in AWU 932 (Core hosting a "tool" directly, contradicting README.md's "no built-in tools of its own" invariant from Phase 19/20). Re-designed as a genuine WASM extension (`skill-tool-provider`) implementing the existing `rad-tool-provider` WIT world — architecturally identical to `mcp-tool-provider`, just backed by local Markdown files instead of MCP servers.
- **New host primitive — `ListDir`**: extensions could already read one known file (`FileRead`/`open-file`) but had no way to enumerate a directory's entries (`.agents/commands/` only worked host-side because that feature runs as trusted Core code with direct `std::fs::read_dir` access — extensions have no equivalent). Since "drop a folder in, get auto-discovered" is core to the UX, added `RasRpcCommand::ListDir { path }` — a minimal, `FileRead`-symmetric primitive (`models/src/lib.rs`, `wit/rad.wit`'s `list-dir(string)` variant, both `rpc_conversion` macros, `FsSubsystem::dir_list`/`FsSandbox::dir_list` reusing `file_read`'s exact `fs_read_allow` permission-check pattern, `rpc_fs.rs` handler, `permissions.rs` check) rather than anything Skills-specific — any future extension needing directory enumeration can reuse it.
- **`skill-tool-provider` (`ext/skill-tool-provider/`, new crate)**: discovers `.agents/skills/<name>/SKILL.md` (project-local, checked first) then `~/.rad/skills/<name>/SKILL.md` (user-global) — same precedence direction and first-seen-wins dedup as `src/command/templates.rs`'s custom slash commands, implemented via `ListDir` + `FileRead` instead of direct `std::fs` calls. Each `SKILL.md` is a `---`-delimited frontmatter block (hand-rolled `key: value` parser, no new YAML dependency — the schema is a handful of scalar fields) plus a body: `description` (required — a skill missing it is logged and skipped, not fatal to the rest), `mode` (reserved for a future nested-task/subagent execution mode; defaults to `inline`, and specifying `subagent` now returns a clear "not implemented" error rather than silently running inline or being ignored), `allowed_tools` (reserved, parsed but unenforced). Invocation substitutes an optional `args` string via the same `$ARGUMENTS`-or-append rule as custom commands. `execute_tool`'s inline-mode result has to come back through the WIT `execution-handle` return type, so — matching `mcp-tool-provider`'s own pattern — it's produced via `open_process(echo ...)`, which means the extension needs bash execution permission even though it never runs arbitrary commands itself.
- **Bug found and fixed along the way**: verifying this via a direct (non-LLM) integration test surfaced that `WasmRuntime::execute_tool` (`src/wasm.rs`) had become **dead code returning nonsense** — it called into the extension's `execute-tool` export but then returned `res.rep().to_string()` (the WASM resource table's raw internal index, e.g. `"0"`) instead of resolving the handle to actual process output. Its only caller had been `rpc_meta.rs`'s `ExecuteTool` RPC handler, which AWU 932 correctly identified and removed as unreachable — but that inadvertently orphaned this method too, since nothing else ever called it (the real production path is `imports_tool.rs`'s WIT-import-level `execute_tool`, which properly transfers the resource handle to the caller for `rad-orchestrator`'s `execute_tool_sync` to resolve). Removed the dead method; rewrote the test to exercise the real path instead (full `Orchestrator` + mocked LLM tool call, matching `tool_loop_tests.rs`'s pattern) rather than calling a method that never actually worked correctly for this.
- **Scope**: `ext/skill-tool-provider/` (new crate: `Cargo.toml`, `src/lib.rs`, `src/skill.rs`, `src/skill/tests.rs`), `Cargo.toml` (workspace members), `scripts/build_all.sh`, `models/src/lib.rs`, `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `wit/rad.wit`, `src/subsystems.rs`, `src/fs.rs`, `src/wasm/{rpc_fs,rpc,permissions,wasm}.rs`, `tests/skill_tool_provider_tests.rs` (new), `CONFIG.md`, `README.md`, `ARCHITECTURE.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2` (including the new crate's own `#![deny(clippy::pedantic)]`); verified against the real installed `rad` binary with a real local LLM, not just mocked tests.
- **Result**: Success. 10 new unit tests in `ext/skill-tool-provider` (frontmatter parsing, `$ARGUMENTS` substitution, precedence ordering, missing-description skip) — genuinely runnable via plain `cargo test` despite the crate's WASM-only WIT imports, since the test binary never calls them. 4 new integration tests in `tests/skill_tool_provider_tests.rs` (discovery via a real compiled `.wasm`, full-Orchestrator execution round-trip, unknown-skill error, missing-description skip) exercising the real production paths for both halves of the feature. Full workspace test suite and Clippy clean on both targets. Real end-to-end verification against the actual installed `rad` binary and a real local LLM (`gemma-4-26B-A4B-it-qat`): the model autonomously selected the registered skill based purely on its tool description (no explicit naming in the prompt) and the exact `SKILL.md` body content came back as the tool result — confirmed after two earlier attempts where the same local model emitted malformed pseudo-tool-call text instead of a proper `tool_calls` JSON payload for this specific tool (a known local-model formatting quirk, not a `rad`-side defect — the integration tests independently prove the mechanism itself is correct regardless of any one model's tool-calling reliability).

---

## 🛠️ Short-Term Plan: Phase 57

### 💡 Current AWU Status
- [✅] AWU 932: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem (Result: Success)

### 📝 AWU Details

#### AWU 932: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem
- **Trigger**: While designing a Skills feature (deferred — see below), a candidate design leaned on `src/wasm/rpc_meta_fallback.rs`'s "read"/"write"/"edit"/"bash" built-in-tool fallback as precedent for "Core can host tools directly." User asked to verify that precedent first. It turned out to directly contradict README.md's own documented invariant — "`rad` ships 5 extensions with no built-in tools of its own" (a Phase 19/20 decision) — and further investigation kept surfacing more dead code in the same shape, so the user asked to finish a full cleanup pass before returning to the Skills work.
- **Finding 1 — the built-in-tool fallback was genuinely unreachable in two different ways**:
  - `src/wasm/rpc_meta_fallback.rs` (reached via `RasRpcCommand::ExecuteTool` sent through generic `host_rpc`) had **zero callers anywhere** — no extension, no test, ever constructs and sends this command for execution (the one place it's constructed, `imports_tool.rs`, only builds a throwaway copy to describe the call to `verify_rpc_exclude` for security checks). Deleted entirely; `rpc_meta.rs`'s `ExecuteTool` match arm now just explains why the variant still exists (kept only because `imports_tool.rs`'s security-check construction and `security-guard`'s policy matching still reference the enum variant) and errors if ever actually dispatched.
  - `src/wasm/imports_tool.rs`'s *separate* fallback (reached via the real `execute-tool` WIT import that `rad-orchestrator` actually calls) **was** reachable, but only because three tests (`self_healing_tests.rs`, `self_healing_core_auto_tests.rs`, `hitl_tests.rs`, `tool_loop_tests.rs`) never registered a real tool-provider extension, so every "bash"/"write" tool call in their mocked LLM responses fell through to it by construction. Not a real product code path — no tool named `read`/`write`/`edit`/`bash` is ever advertised by `GetTools` in normal operation.
- **Finding 2 — a second, larger legacy subsystem for MCP servers, fully superseded**: `RasRpcCommand::SpawnMcpServer`/`SendMcpRequest`, their WIT payloads, the `RasCoreEvent::McpResponse` event, `PermissionConfig::allowed_mcp_servers`, and a dedicated `src/mcp.rs` (`McpProcess`, using plain `std::process::Command` with **no** PGID isolation, unlike the real process manager) — an entirely separate MCP-process-management path from the one `mcp-tool-provider` actually uses today (`RasRpcCommand::OpenProcess` + raw stdio piping, confirmed in Phase 55/AWU 930's work on that extension). Only `tests/mcp_tests.rs` ever exercised it. Independent corroboration found in this file's own AWU 926 entry, which had already flagged `allowed_mcp_servers` as a "wrong field name" removed from the *docs* back then — the code was simply never cleaned up to match. All of it removed: `src/mcp.rs` deleted; `RasRpcCommand::SpawnMcpServer`/`SendMcpRequest` and `RasCoreEvent::McpResponse` removed from `models/src/lib.rs` and their WIT variants/payloads from `wit/rad.wit`; `PermissionConfig::allowed_mcp_servers` removed (and its `/tools` display line in `src/command/tools.rs`); `active_mcp_servers` field removed from `WasmState`/`RpcContext`/`execute_rpc_command`'s threading; conversion arms removed from `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `src/wasm/bindings_event.rs`, `ext/rad-orchestrator/src/conv/event.rs`; stale copies of the removed WIT variants/event fixed in `ARCHITECTURE.md` (§3.2.3's reproduced `ras-rpc-command` listing, §4.1's PGID text, and the `RasCoreEvent` listing) plus a real, pre-existing PGID-isolation inaccuracy corrected in the same pass (the deleted `McpProcess` never actually called `setpgid`, contradicting what §4.1 claimed).
- **Bug found and fixed along the way**: `mcp-tool-provider`'s `get_tools()` (`ext/mcp-tool-provider/src/lib.rs`) pushed its `RAD_TEST_PORT` synthetic tools ("read"/"write"/"execute") but then **unconditionally** still called `init_mcp_servers()?` afterward, discarding those synthetic tools via `?` the instant no real `mcp_servers` config existed — silently defeating the whole point of test mode (self-contained, no real MCP servers needed) any time `get_tools()` (as opposed to `execute_tool()`, which was already correct) was the thing under test. Fixed by returning the synthetic tools directly, before ever touching real MCP discovery — this is what surfaced Finding 1's tests' true breakage once the dead fallback was gone (they'd been silently relying on it to paper over this bug too).
- **Test restructuring** (following from Finding 1): `tests/self_healing_tests.rs` deleted outright — its manual single-`WasmRuntime` driving style only worked because of two simultaneous dead-code shortcuts (no tool-provider needed thanks to the built-in fallback; no real `Orchestrator` needed because `ctx.orchestrator == None` routed `GenerateLlmStream` to a *third*, still-legitimate test-only fallback, `rpc_meta_llm_fallback.rs`, confirmed still load-bearing for `tool_loop_tests.rs` and kept as-is). Fixing the tool-provider lookup requires a real `Orchestrator`, which forecloses that LLM-mocking shortcut too — properly fixing it would have converged on exactly what `self_healing_core_auto_tests.rs` (kept) already tests, so it was redundant once fixed rather than worth reconstructing. `self_healing_core_auto_tests.rs`, `hitl_tests.rs`, and `tool_loop_tests.rs` were restructured to register a real `mcp-tool-provider` (in its self-contained `RAD_TEST_PORT` mode) alongside `rad-orchestrator`/`llm-connector`, using the `execute` test-tool (with real shell commands, e.g. `echo -n '...' > file`) instead of the no-longer-served `bash`/`write` names. `tests/mcp_tests.rs` (Finding 2) deleted outright — it exclusively tested the removed `SpawnMcpServer`/`SendMcpRequest` mechanism.
- **Flakiness found and fixed while restructuring** (not pre-existing regressions from this AWU, but surfaced by touching these files): `hitl_tests.rs` and `tool_loop_tests.rs` each run 2 `#[test]` functions that set process-global `RAD_YOLO`/`RAD_TEST_APPROVE`/`RAD_TEST_PORT` env vars — under `cargo test`'s default parallel execution these raced between threads in the same binary. Fixed with the same `static TEST_MUTEX: std::sync::Mutex<()>` pattern already established in `llm_command_tests.rs`, verified stable across repeated parallel runs. Separately, `self_healing_core_auto_tests.rs`'s 5s completion timeout became occasionally too tight after gaining a third registered extension (one more Wasm component to compile/instantiate on the self-healing retry); bumped to 8s and reverified stable across 5 consecutive runs.
- **Scope**: `src/mcp.rs` (deleted), `src/wasm/rpc_meta_fallback.rs` (deleted), `tests/mcp_tests.rs` (deleted), `tests/self_healing_tests.rs` (deleted), `src/lib.rs`, `src/wasm.rs`, `src/wasm/{rpc,rpc_meta,rpc_process,imports_rpc,imports_tool,permissions,bindings_event,loader}.rs`, `src/config.rs`, `src/command/tools.rs`, `models/src/lib.rs`, `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `ext/rad-orchestrator/src/conv/event.rs`, `ext/mcp-tool-provider/src/lib.rs`, `wit/rad.wit`, `ARCHITECTURE.md`, `tests/{hitl_tests,tool_loop_tests,self_healing_core_auto_tests}.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; a real `rad` run against real MCP servers confirms no functional regression.
- **Result**: Success. Full workspace build/clippy clean on both targets. Full test suite green, including repeated-run stability checks for the two previously-racy files (5+ consecutive clean parallel runs each) and the retimed self-healing test. Verified end-to-end via the real installed `rad` binary against the real `core-utilities-mcp`/`web-access-mcp` servers (`~/projects/test`): a multi-tool-call task (directory listing, two file reads, a real shell `wc`-style tool call) completed correctly with 19 tools recognized at startup, confirming the real WIT-import-based tool-execution path is completely unaffected by removing the dead RPC-command-based one.

---

## 🛠️ Short-Term Plan: Phase 56

### 💡 Current AWU Status
- [✅] AWU 931: wasmtime Compile Cache & MCP Tool-List Caching (Result: Success)

### 📝 AWU Details

#### AWU 931: wasmtime Compile Cache & MCP Tool-List Caching
- **Trigger**: User asked where `rad`'s own architecture (as opposed to raw LLM
  inference latency, acknowledged as out of scope) could be sped up. A
  code-reading pass surfaced two candidates; both were verified empirically
  (not just reasoned about) before being scoped for a fix, and a couple of
  other candidates raised along the way were explicitly retracted or ruled
  out of scope once measured — see below.
- **Fix 1 — wasmtime compile cache**: any single extension crash triggers
  `clear_runtimes()` (`src/orchestrator/runner/runtimes.rs`), which discards
  **all 5** extension runtimes; the retry then recompiles all 5 from scratch
  via `WasmRuntime::new` (`src/wasm/loader.rs`), which builds a fresh
  `Engine` + `Component::from_file` with no caching. Measured with a
  throwaway benchmark (same wasmtime 29.0.1, same debug `.wasm` files
  `~/.rad/wasm` actually points at): **210ms cold, uncached**. Enabled
  wasmtime's built-in disk-based compilation cache
  (`Config::cache_config_load_default()`, gated behind the crate's `cache`
  feature — added to `wasmtime`'s features in `Cargo.toml`) in
  `WasmRuntime::new`; a load failure only logs via `log_host!` and falls
  back to always-recompile rather than blocking extension load, since
  caching is an optimization, not a correctness requirement. Deliberately
  **did not** pursue sharing a single `Engine` across extensions/self-heals
  (would have required changing `WasmRuntime::new`'s signature and touching
  11 call sites — 1 production + 10 tests): the benchmark showed
  `Engine::new()` itself is already near-zero cost after the first call
  in-process, and wasmtime's compile cache is a content-hash-keyed disk
  cache independent of which `Engine` instance requests the compile, so
  caching alone captures effectively all the available win without that
  blast radius.
- **Fix 2 — MCP tool-list caching**: `mcp-tool-provider`'s `get_tools()`
  (`ext/mcp-tool-provider/src/lib.rs`) issued a live, uncached `tools/list`
  to every configured MCP server on **every single call** — and it's called
  from both `trigger_llm_stream()` (every LLM turn, via
  `get_available_tools()` in `ext/rad-orchestrator/src/tool.rs`) and the
  host's `ExecuteTool` routing lookup (`src/wasm/rpc_meta.rs`, once per tool
  call, to find which extension owns a tool name). A raw JSON-RPC timing
  test against the real `core-utilities-mcp`/`web-access-mcp` binaries
  showed each server's own `tools/list` response is negligible
  (~0.06–0.08ms) — an earlier suspicion that `mcp_transport.rs::read_line`'s
  10ms poll-on-empty-read loop was adding latency here was **investigated
  and retracted**: the host-side stream read
  (`src/wasm/imports_resources.rs`) blocks on `mpsc::Receiver::recv()`
  rather than polling, so that path barely triggers in normal operation.
  Even so, the *design* was pure waste: `init_mcp_servers()`
  (`ext/mcp-tool-provider/src/client.rs`) now returns `Result<bool, String>`
  (`did_reinit` — true only when it actually had to (re)spawn a server, false
  when the existing connections' liveness check passed) instead of
  `Result<(), String>`; `get_tools()` skips the entire live-fetch loop and
  returns a cached `Vec<Tool>` (new `MCP_TOOLS_CACHE` static, alongside the
  existing `MCP_TOOL_MAPPING`) whenever `did_reinit` is false and the cache
  is populated, and `init_mcp_servers()` itself clears the cache whenever it
  does have to reinitialize. The live-fetch loop was extracted into a
  `fetch_and_cache_mcp_tools()` helper to keep `get_tools()` under the
  100-line Clippy pedantic limit. `mcp_transport.rs`'s two
  `init_mcp_servers()?` call sites needed no changes at all — both already
  discard the result as a bare statement, and `bool` isn't `#[must_use]`.
  Host-side `ExecuteTool`'s separate `get_tools()` call for routing was left
  untouched: with the extension-side cache in place it's now a cheap
  in-memory return, so restructuring host routing logic for a now-marginal
  gain wasn't worth the added risk.
- **Retracted/out-of-scope after investigation**: `read_line`'s 10ms poll
  loop (see above — real code path, but not the normal-operation bottleneck
  it first appeared to be); `main.rs`'s 50ms Esc/completion poll loop
  (estimated impact too small to matter); `security-guard`'s
  `verify_rpc_exclude` WASM-boundary crossing on every RPC (inherent
  sandboxing cost — removing it would weaken the security model for a small
  gain).
- **Scope**: `Cargo.toml`, `src/wasm/loader.rs`,
  `ext/mcp-tool-provider/src/client.rs`, `ext/mcp-tool-provider/src/lib.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass,
  native and `wasm32-wasip2`; both fixes verified with real measurements
  against the actual artifacts involved, not just reasoning from the code.
- **Result**: Success. Fix 1: re-measured with the same benchmark — cold
  compile of all 5 extensions went from 210ms to (with the cache warm)
  **~110ms** (a fresh from-scratch timing run measured 308ms cold → 107ms
  warm, ~65% reduction, stable across repeated runs; confirmed the on-disk
  cache directory `~/Library/Caches/BytecodeAlliance.wasmtime` was actually
  created and used). Fix 2: verified with temporary unconditional `eprintln!`
  markers (added, checked, then reverted — not part of the shipped diff)
  during a real multi-tool-call task via the installed `rad` binary against
  the real `core-utilities-mcp`/`web-access-mcp` servers: **1 live fetch
  followed by 7 cache hits** for a task with 3 tool calls (previously would
  have been 8 separate live `tools/list` round-trips). Full workspace test
  suite (native, including both self-healing tests) and Clippy clean on
  both targets; end-to-end task output unchanged/correct with the cache
  path active.

---

## 🛠️ Short-Term Plan: Phase 55

### 💡 Current AWU Status
- [✅] AWU 930: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider` (Result: Success)

### 📝 AWU Details

#### AWU 930: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider`
- **Trigger**: Continued dogfooding review of `ext/mcp-tool-provider/src/client.rs` found it hardcoded `protocolVersion: "2024-11-05"` — two spec generations behind — and still performed the `initialize`/`notifications/initialized` handshake that the 2026-07-28 MCP spec redesign removed entirely (stateless `server/discover` replaces it). All JSON-RPC request/response construction across the crate (`client.rs`'s handshake, `lib.rs`'s `tools/list` and `tools/call`) was hand-built via `serde_json::json!()` and parsed via manual `.get("...")` chains, with no compile-time guarantee of matching the real MCP schema. User asked for a fresh survey of the MCP-Rust-crate ecosystem (`rmcp`, gateway/aggregator tools like AgentGateway/MetaMCP, `rust-mcp-schema`) before deciding how to fix it.
- **Rejected alternatives**: `rmcp` (the official SDK) is Tokio-based and assumes direct OS access to spawn processes/open sockets — architecturally incompatible with `mcp-tool-provider`'s WASM-sandboxed, host-mediated-`host_rpc` transport model. MCP gateway/aggregator tools (AgentGateway, MetaMCP) solve a different-scale problem (multi-tenant aggregation behind one endpoint) than rad's single-user local use case, add operational complexity conflicting with rad's "single static binary, zero dependencies" philosophy, and don't even eliminate the need for an MCP client implementation since rad would still have to speak MCP to the gateway itself.
- **Fix**: adopted `rust-mcp-schema` (crates.io, MIT, transport-agnostic — just `serde` structs, no async runtime) for protocol *types only*, keeping the existing host-mediated process-spawn/stdin-stdout-pipe transport unchanged. Verified it builds cleanly for `wasm32-wasip2` with zero shim code needed. `client.rs`'s `init_mcp_servers` handshake now builds `ClientJsonrpcRequest::new(RequestId::String("init_1"), RequestFromClient::InitializeRequest(InitializeRequestParams { protocol_version: ProtocolVersion::latest().to_string(), .. }))` (currently resolves to spec `2025-11-25`, the newest the crate ships — still 2 generations ahead of the previous hardcoded `2024-11-05`) and `InitializedNotification::new(None)`, extracted into a new `perform_handshake` helper to keep `init_mcp_servers` under the 100-line Clippy pedantic limit. `lib.rs`'s `get_tools` builds `RequestFromClient::ListToolsRequest(None)` and parses the response via `serde_json::from_value::<ListToolsResult>`; `execute_tool` builds `RequestFromClient::CallToolRequest(CallToolRequestParams { name, arguments, .. })` and parses via `serde_json::from_value::<CallToolResult>`, extracting `ContentBlock::TextContent` variants. `RequestId::String(...)` (not `Integer`) was used throughout deliberately, to match `mcp_transport.rs::send_mcp_bytes`'s existing `req_val.get("id").and_then(|v| v.as_str())` response-correlation logic without needing to touch that file at all — the JSON-RPC wire format and transport layer stay exactly as they were; only the construction/parsing at the two call sites is now typed.
- **Scope**: `ext/mcp-tool-provider/Cargo.toml`, `ext/mcp-tool-provider/src/client.rs`, `ext/mcp-tool-provider/src/lib.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; verified end-to-end against the real installed `core-utilities-mcp` and `web-access-mcp` servers via the actual `rad` binary, not just automated tests.
- **Result**: Success. Full workspace test suite and Clippy (`#![deny(clippy::pedantic)]` in this crate) clean on both targets. Verified via two real `rad` runs from `~/projects/test` against the real running MCP servers: a `core-utilities-mcp` directory-listing call and a `web-access-mcp` HTTP-fetch call both round-tripped correctly end-to-end through the new typed handshake/list/call path (`[OK] Verified 18 tools from extension 'mcp-tool-provider'`).

---

## 🛠️ Short-Term Plan: Phase 54

### 💡 Current AWU Status
- [✅] AWU 926: MCP/LLM Onboarding Fixes & Documentation Consolidation (Result: Success)
- [✅] AWU 927: Fix `llm-connector` Eager-Load Stale-Environment-Snapshot Bug (Result: Success)
- [✅] AWU 928: Dedupe Repeated Inline Tool Calls from Local-Model Output (Result: Success)
- [✅] AWU 929: Restore Esc-to-Abort for Running Tasks (Result: Success)

### 📝 AWU Details

#### AWU 926: MCP/LLM Onboarding Fixes & Documentation Consolidation
- **Trigger**: User began real-world dogfooding (first actual `rad` run outside this repo, in a fresh `~/projects/test` directory) per the plan agreed at the end of Phase 53. Two real failures surfaced immediately.
- **Issue 1 — zero tools available**: `[FAILED] Extension 'mcp-tool-provider' get_tools error: load_mcp_config returned None (no mcp_servers config found)`. Root cause: post-Phase-19/20, `rad` has no built-in file/shell tools at all — MCP servers registered under `mcp-tool-provider`'s own `config.mcp_servers` are the *only* source of tools, and the user's `~/.rad/config.json` (assembled in Phase 48) never had any registered. **Fix**: registered the user's already-installed `core-utilities-mcp` and `web-access-mcp` binaries (found via `which`/`~/.cargo/bin`) in `~/.rad/config.json`'s `mcp-tool-provider.config.mcp_servers`. Verified via a real `rad` run: `[OK] Verified 17 tools`.
- **Issue 2 — stale/inaccurate documentation**: Investigating Issue 1 revealed `README.md`'s and `CONFIG.md`'s example configs were both stale relative to the real schema (fictional `builtin://` scheme, fictional `rad --update` flag, fictional `.rad/skills/`/`.rad/workflows/`/`extension.json` directory-packaged-extension mechanism — none implemented anywhere in the codebase, confirmed by exhaustive grep — plus wrong field names like `allowed_mcp_servers` and a wrong alternate-path claim `.rad/rad.json` instead of the real `.rad/config.json`). **Fix**: rewrote `CONFIG.md` as the single authoritative schema/discovery/directory-layout reference (real 5-extension example including `mcp_servers`, corrected precedence-cascade file paths, added previously-undocumented real features: `core.max_sessions`, `.rad/sessions/`, `~/.rad/wasm/`, `.agents/commands/`/`~/.rad/commands/` custom slash commands, `AGENTS.md` project rules). Trimmed `README.md`'s duplicate full JSON example down to a 2-line summary + pointer to `CONFIG.md`, per user's explicit "don't duplicate facts across files" direction — the duplication was the root cause of both files independently rotting out of sync in the first place. Removed the same fictional Skills/Workflows content from `README.md`'s feature list and `ARCHITECTURE.md` §5.4 (whose "Basic OS Primitives (Core)" item was independently stale for the same reason: no host-side tool primitives exist post-Phase-19/20).
- **Issue 3 — no LLM endpoint configured**: Task execution failed with `[LLM Connector Error] No LLM endpoint configured`. Root cause: `~/.rad/config.json`'s `llm.active`/`llm.endpoints` were still empty from Phase 48. Found a real `llama-server` already running locally (`qwen2.5-coder-32b-instruct-q4_k_m`, port 8080, `/props` reporting `n_ctx: 16384`) and registered it as the `local` profile. This surfaced Issue 4 below.
- **Scope**: `~/.rad/config.json` (not part of the git repo), `README.md`, `CONFIG.md`, `ARCHITECTURE.md`.
- **Result**: Success. Real `rad` run against the local `llama-server` confirmed tools + config load correctly.

#### AWU 927: Fix `llm-connector` Eager-Load Stale-Environment-Snapshot Bug
- **Trigger**: Directly following AWU 926's LLM endpoint fix, the *first* real task after starting `rad` failed with `[LLM Connector Error] No LLM endpoint configured`, self-healed (Wasm instance respawned), and the *second* attempt succeeded — a real, reproducible regression, not a one-off.
- **Root cause**: `WasmRuntime::new` (`src/wasm/loader.rs`) builds its WASI context via `WasiCtxBuilder::new().inherit_env()`, which snapshots the *host process's* environment variables once, at Wasm instance creation. `ext/llm-connector/src/connector.rs`'s `generate_stream` read its `base_url`/`api_key` from `LLM_BASE_URL`/`RAD_BASE_URL`/`OPENAI_BASE_URL`/`*_API_KEY` env vars — but those were only ever `std::env::set_var`'d inside `run_task_internal` (`src/orchestrator/runner.rs`), i.e. *after* a task starts. Phase 47-3 (AWU 919) made extension loading eager — at startup, before any task and before those `set_var` calls ever ran — so the very first `llm-connector` instance's WASI snapshot was permanently missing them; only a self-healing respawn (which recreates the instance *after* `run_task_internal` had already run once) picked up the correct values. The same staleness meant `/llm switch`/`/llm add`/`/llm model` mid-session would also silently not take effect until the next self-heal.
- **Fix (root cause, not a timing patch)**: extended `producer::generate-stream` in `wit/connector/llm-connector.wit` to take `base-url: option<string>` and `api-key: option<string>` as explicit call arguments instead of reading them from environment variables inside the guest at all. The host (`src/wasm/rpc_meta_llm_connector.rs`) now resolves the active profile fresh on *every* call via a new `resolve_active_llm_profile` helper in `rpc_meta.rs` (reading `orch.config.lock()` directly — no RPC round-trip needed, the host already holds the config) and passes `base_url`/`api_key` straight into the WIT call. This structurally eliminates the staleness vector — there's no snapshot to go stale, the values are fresh function arguments on every invocation — rather than just reordering `set_var` calls to happen earlier (which would only have fixed the startup case, not the mid-session `/llm switch` case). `resolve_active_llm_profile` deliberately isn't merged into the existing `GetActiveLlmProfile` RPC response (`active_llm_profile_json`): that RPC is generically reachable by any extension, and a credential doesn't belong in a broadly-readable fact query — `base_url`/`api_key` stay host-internal, passed only to the one extension that needs them, arguably *tighter* exposure than the old process-wide environment variables. The now-dead `unsafe { std::env::set_var(...) }` block in `run_task_internal` (`OPENAI_MODEL`/`LLM_MODEL` were already unused by anything, confirmed by grep) was removed entirely. `RAD_TEST_PORT` (test infrastructure, unrelated to this bug) was deliberately kept as an env-var override in `connector.rs`.
- **Scope**: `wit/connector/llm-connector.wit`, `ext/llm-connector/src/connector.rs`, `src/wasm/rpc_meta.rs`, `src/wasm/rpc_meta_llm_connector.rs`, `src/orchestrator/runner.rs`, `tests/llm_connector_eager_load_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; a new regression test reproduces the exact eager-load ordering with a real (non-`RAD_TEST_PORT`) `llm.endpoints` config.
- **Result**: Success. New `tests/llm_connector_eager_load_tests.rs` proves the first task after eager loading succeeds outright (one connection attempt, not two). Verified against the real local `llama-server` too: previously 2 connection attempts (1 failure + 1 self-heal retry) before success, now exactly 1. Full workspace test suite (21 binaries) and Clippy clean on both targets; `./scripts/build_all.sh` clean end-to-end.

#### AWU 928: Dedupe Repeated Inline Tool Calls from Local-Model Output
- **Trigger**: Continued dogfooding surfaced a real task where the local `qwen2.5-coder-32b` model responded with the identical inline tool-call JSON (`{"name": "list_directory_contents", "arguments": {"path": "./"}}`) twice back-to-back in one turn's plain-text content — the preceding apology sentence appeared only once, so this was the model itself repeating just the JSON snippet (a known small/quantized-model quirk), not a streaming/duplication bug in `rad`'s own SSE handling. `parse_bare_json_tool_calls` (`ext/rad-orchestrator/src/orchestrator/runner/inline_tool_calls.rs`) scans the full text for every JSON-object-shaped occurrence and fires a separate tool call for each one found, so it faithfully — but unhelpfully — executed the identical call twice, producing two identical `[Tool Output]` blocks and likely contributing to the `LLM stream stalled` timeout that followed (the model's next turn had to make sense of a confusing duplicated tool-call/tool-response pair in its context).
- **Fix**: `push_tool_call` (the shared helper both the `call:`-prefixed and bare-JSON inline parsers funnel through) now skips pushing a call whose `(name, arguments)` exactly matches the *immediately preceding* pushed call in the same parse pass. Deliberately adjacent-only, not a full-history dedup: `assistant_tool_calls` is always empty when inline parsing starts (it only runs when the proper `delta.tool_calls` streaming mechanism produced nothing), so this can't cross-contaminate with unrelated real tool calls, and calling the same tool again later in the same turn with something else in between remains a legitimate, unaffected case.
- **Scope**: `ext/rad-orchestrator/src/orchestrator/runner/inline_tool_calls.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. 3 new tests: dedupes the exact reported case (adjacent identical calls collapse to 1), confirms non-adjacent repeats of the same call still both fire, confirms adjacent calls with *different* arguments still both fire. Full workspace test suite (21 binaries) and Clippy clean on both targets; `./scripts/build_all.sh` clean end-to-end.

#### AWU 929: Restore Esc-to-Abort for Running Tasks
- **Trigger**: User asked to be able to press Esc to interrupt a running task's thinking/work — mentioned in passing during AWU 927/928's dogfooding that they'd previously used Esc to stop a stuck task. The roadmap's own Phase 14 ("Esc Key Task Abort Polish & Robustness", v0.19.0) implies this once existed, but grepping the current codebase for `Esc`/`KeyCode`/`crossterm` turned up nothing at all in `src/` — `crossterm` is declared in `Cargo.toml` but was completely unused. `main.rs`'s `run_agent_task` blocks the main thread in a plain `while orchestrator.is_running() { sleep(50ms) }` loop while a task runs, and `rustyline`'s `Editor::readline()` — the only thing that would otherwise read stdin — isn't being called during that window. Conclusion: nothing currently reads stdin at all while a task is running, so no keypress (Esc included) could have reached any code; the earlier "I stopped it with Esc" was most likely the natural 15s stall timeout coinciding with the keypress, not a real abort path. This was a confirmed regression/dead-feature, not a verification task.
- **Design constraint discovered**: `crossterm::terminal::enable_raw_mode()` (the obvious approach) clears the *output* side of termios (`OPOST`/`ONLCR`) as well as input — but the background task thread concurrently `println!`s streamed LLM tokens via `src/terminal.rs`'s `TerminalController`, which relies on the terminal's automatic `\n` → `\r\n` translation. Enabling full raw mode while that's happening would turn concurrent output into a staircase mess.
- **Fix**: new `src/esc_abort.rs` module manipulates *only* the input side of termios directly via `nix::sys::termios` (already a dependency with the `"term"` feature) — clears `ICANON`/`ECHO` from `local_flags`, sets `VMIN`=0/`VTIME`=0 for a non-blocking-style `read()`, and leaves output flags untouched. `RawInputGuard` is an RAII guard restoring the original settings on drop; `esc_pressed(&RawInputGuard)` takes the guard by reference specifically so it can't be called without raw mode active (which would otherwise block waiting for a full line). `main.rs`'s `run_agent_task` poll loop now checks `esc_pressed` each 50ms tick and calls the pre-existing (already fully implemented) `orchestrator.abort()` on the first hit, routing the confirmation message through `TerminalController::write_log` (deferred like every other log during `Thinking`/`Streaming` state, so it surfaces cleanly once things settle back to `Idle` rather than interleaving with in-flight streamed tokens).
- **Scope**: `src/esc_abort.rs` (new), `src/lib.rs`, `src/main.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 2 new tests (`contains_esc` byte-detection logic, and confirming `RawInputGuard::enable()` gracefully returns `None` rather than hanging on the non-tty stdin `cargo test` itself runs under — exercises the real fallback path for free). Full workspace test suite and Clippy clean; `./scripts/build_all.sh` clean end-to-end. The actual interactive Esc keypress behavior needs a real TTY to verify (not reproducible via piped Bash-tool input), left for the user to confirm directly.

---

## 🛠️ Short-Term Plan: Phase 53

### 💡 Current AWU Status
- [✅] AWU 925: File-Size Limit Compliance Cleanup (Result: Success)

### 📝 AWU Details

#### AWU 925: File-Size Limit Compliance Cleanup
- **Trigger**: User asked, before starting real-world dogfooding, whether a comprehensive refactor was warranted. Assessed no (wait for real usage to surface concrete pain points first) but flagged 3 files that had drifted over CODING.md's 300-line limit as cheap, mechanical, zero-design-risk cleanup worth doing regardless: `models/src/rpc_conversion.rs` (370), `tests/multi_extension_tests.rs` (350, pre-existing since AWU 920), `tests/self_healing_tests.rs` (411). User agreed to clear them.
- **`models/src/rpc_conversion.rs`**: the two large per-direction command macros (`impl_rpc_command_wit_to_core!`, `common_rpc_command_core_to_wit!`) moved to new companion files `rpc_conversion/wit_to_core.rs` / `rpc_conversion/core_to_wit.rs`. Pure mechanical move — `#[macro_export]` always exports at the crate root regardless of which module a macro is textually defined in, so every one of the 5 consumer crates was unaffected (confirmed by full rebuild across all `wasm32-wasip2` extension crates with zero changes needed at any call site).
- **`tests/multi_extension_tests.rs`**: `test_multi_extension_isolated_roles` split into a new `tests/multi_extension_isolated_roles_tests.rs`. Each file's shared `run_mock_http_server`/`security_guard_config` helpers duplicated per-file (matching the precedent already set by `security_guard_policy_tests.rs`/`security_hook_tests.rs`) rather than factored into a shared `tests/common/` module — out of scope for a same-day mechanical split. `TEST_MUTEX` dropped from both resulting files: it only ever mattered for serializing tests sharing one process, and each file is now its own single-test binary.
- **`tests/self_healing_tests.rs`**: `test_core_auto_self_healing_integration` split into a new `tests/self_healing_core_auto_tests.rs`. Unlike the multi_extension split, the two original tests used entirely disjoint helpers (`MockNetwork`+direct `WasmRuntime` driving vs. a real mock HTTP server + full `Orchestrator`), so no duplication was needed at all — a completely clean cut.
- **Scope**: `models/src/rpc_conversion.rs`, `models/src/rpc_conversion/wit_to_core.rs` (new), `models/src/rpc_conversion/core_to_wit.rs` (new), `tests/multi_extension_tests.rs`, `tests/multi_extension_isolated_roles_tests.rs` (new), `tests/self_healing_tests.rs`, `tests/self_healing_core_auto_tests.rs` (new), `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; zero files over 300 lines anywhere in `src/`, `ext/`, `models/`, `tests/`.
- **Result**: Success. Verified with a full repo-wide line-count sweep (zero violations remaining) and `./scripts/build_all.sh` end-to-end.

---

## 🛠️ Short-Term Plan: Phase 52

### 💡 Current AWU Status
- [✅] AWU 924: Slash Command Composition Review, Role-Based Extension Lookup & `/compact` (Result: Success)

### 📝 AWU Details

#### AWU 924: Slash Command Composition Review, Role-Based Extension Lookup & `/compact`
See the Phase 52 roadmap scope note above for the full trigger/design/result narrative — recorded there rather than duplicated here since this AWU's work *is* Phase 52 in its entirety (no sub-items split across multiple AWUs).

---

## 🛠️ Short-Term Plan: Phase 51

### 💡 Current AWU Status
- [✅] AWU 923: Size-Aware Tool-Result Clearing, Lexical Relevance Retention & Structured-Fact Digest (51-1, 51-2, 51-3) (Result: Success)

### 📝 AWU Details

#### AWU 923: Size-Aware Tool-Result Clearing, Lexical Relevance Retention & Structured-Fact Digest (51-1, 51-2, 51-3)
- **Trigger**: User asked to execute Phases 49–51 in one pass following Phase 48's completion.
- **51-1 (tool-result clearing)**: Added `windowing::clear_stale_tool_results` in `ext/context-tools/src/windowing.rs`, run before windowing inside `optimize`. When the full message list exceeds `max_content_chars`, it replaces the content of older `tool`-role messages (oldest first) with a short placeholder — never removing the message itself, so the `tool_calls`/`tool` pairing invariant (AWU 909) stays intact — always preserving at least the single most recent tool result untouched. Reuses the existing `max_content_chars` field entirely; no new WIT surface.
- **51-2 (lexical relevance retention)**: Added `windowing::trim_with_relevance_retention`, layered on top of the existing positional windowing. After computing the standard "first + recent tail" cutoff, if `max_content_chars` leaves slack, earlier ("middle") turns are scored by keyword overlap with the goal message (simple lowercase-word-set intersection, no ML/embedding cost) and greedily reinstated highest-score-first until the budget is used. Turns are grouped so a `user` message and everything following it up to the next `user` message move as one unit — a reinstated turn can never split an `assistant`/`tool_calls` message from its `tool` reply. Purely additive: can only add back messages positional windowing already decided to drop.
- **51-3 (structured-fact digest)**: New `ext/rad-orchestrator/src/digest.rs` walks assistant `tool_calls` JSON (heuristically, via common argument key names like `path`/`file_path`/`command`/`cmd`, since this project has no built-in tools post-Phase-19/20 and MCP tool schemas vary) to build a deterministic "files touched / commands run" digest, capped to the most recent 30 items each. Appended to the system prompt in `load_messages_from_dag`, computed from the full pre-optimization message list. Since the system message is split out before `context-tools.optimize` and re-attached unconditionally after (pre-existing behavior), the digest survives regardless of how aggressively the rest of history gets windowed or cleared — no new plumbing needed for the "survives compaction" property.
- **File-size compliance**: `ext/context-tools/src/lib.rs`'s windowing logic moved to a new `ext/context-tools/src/windowing.rs` (+ companion `windowing/tests.rs`) to stay under 300 lines after 51-1/51-2 grew it past the limit; `lib.rs` now just wires `Guest::optimize` to `windowing::` calls. Similarly split `ext/rad-orchestrator/src/llm.rs`'s `CtMessage`/`CtOptimizationRequest`/`CtOptimizationResponse` wire structs into `ext/rad-orchestrator/src/llm/context_tools_wire.rs`.
- **Scope**: `ext/context-tools/src/lib.rs`, `ext/context-tools/src/windowing.rs` (new), `ext/context-tools/src/windowing/tests.rs` (new), `ext/context-tools/src/tests.rs`, `ext/rad-orchestrator/src/digest.rs` (new), `ext/rad-orchestrator/src/digest/tests.rs` (new), `ext/rad-orchestrator/src/llm.rs`, `ext/rad-orchestrator/src/llm/context_tools_wire.rs` (new), `ext/rad-orchestrator/src/lib.rs`, `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. `context-tools` unit tests grew from 10 to 13 (windowing-specific ones now live in `windowing/tests.rs`); `rad-orchestrator` gained 7 digest tests; full workspace `cargo test` (18 test binaries) and Clippy clean on both targets. Verified through the real WASM component boundary via `tests/context_tools_tests.rs` and the full `e2e_tests`/`tool_loop_tests` suites.

---

## 🛠️ Short-Term Plan: Phase 50

### 💡 Current AWU Status
- [✅] AWU 922: Session File Rotation (50-1) (Result: Success)

### 📝 AWU Details

#### AWU 922: Session File Rotation (50-1)
- **Trigger**: Phase 50 item 50-1 — `.rad/sessions/<id>.json` files had no cleanup mechanism at all; nothing ever deleted them.
- **Fix**: Added `CoreConfig.max_sessions: usize` (default 50, `~/.rad/config.json`-configurable) and `session::prune_sessions(workspace, keep, keep_id)` in `src/session.rs`, called once at startup from `src/startup.rs::load_config_and_session` right after the session ID and DAG are resolved. Deletes session files beyond the `keep` most-recently-modified ones, always excluding the current session's own file regardless of its age (it may be an explicitly `--session`-resumed old session about to be written to again). Best-effort: I/O errors (missing directory, permissions) are silently ignored rather than surfaced, since pruning is hygiene, not a startup precondition.
- **Scope**: `src/config.rs`, `src/session.rs`, `src/startup.rs`, `tests/command_tests.rs`, `tests/llm_command_tests.rs`, `tests/git_autopilot_tests.rs`, `tests/stabilization_tests.rs`, `tests/self_healing_tests.rs` (the last 4 needed a `..Default::default()` addition to their manually-constructed `CoreConfig` literals after the new field landed).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 4 new unit tests in `src/session.rs` (keeps-newest-N, never-deletes-current, no-op-under-limit, no-op-on-missing-directory); full workspace test suite (18 binaries) and Clippy clean.

---

## 🛠️ Short-Term Plan: Phase 49

### 💡 Current AWU Status
- [✅] AWU 921: `/llm context <n>` Manual Context-Length Override (49-1) (Result: Success)

### 📝 AWU Details

#### AWU 921: `/llm context <n>` Manual Context-Length Override (49-1)
- **Trigger**: Phase 49 item 49-1 — some backends (plain OpenAI-compatible proxies with neither `/props` nor `/api/show`) never get a detected `context_length`, leaving size-based windowing permanently disabled for them with no way for the user to supply the real number.
- **Fix**: Added a `context` entry to `llm_subcommand_specs()` (`src/command/llm.rs`) and `set_manual_context_length` (`src/command/llm/profile_admin.rs`), following the exact registry pattern AWU 918 established — no new dispatch mechanism. Sets `context_length` on the active profile and persists via the existing `save_global_config`. Like every other detected value, it's overwritten again the next time real detection succeeds (`/llm add`/`/llm test`/`/llm model`), since a fresh real reading always wins over a manual guess.
- **Scope**: `src/command/llm.rs`, `src/command/llm/profile_admin.rs`, `tests/llm_command_tests.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 3 new tests (sets override, rejects non-numeric input, errors with no active profile); full test suite and Clippy clean.

---

## 🛠️ Short-Term Plan: Phase 48

### 💡 Current AWU Status
- [✅] AWU 920: Config-Driven `security-guard` Policy via New `GetExtensionConfig` RPC (48-1 & 48-2) (Result: Success)

### 📝 AWU Details

#### AWU 920: Config-Driven `security-guard` Policy via New `GetExtensionConfig` RPC (48-1 & 48-2)
- **Trigger**: User asked to proceed with Phase 48 after Phase 47 wrapped. `ext/security-guard/src/lib.rs::verify_rpc` hardcoded its blocklist (`"blocked.txt"`/`"blocked_command"` literals) directly in the match arms, with no way for `~/.rad/config.json` to influence it — misleading for a component named "security-guard", and the reason `ExtensionConfig.config` (already parsed, tested, never read) existed as dead weight.
- **Design**: Rather than inventing a bespoke config-passing side channel, added a new generic RPC (`RasRpcCommand::GetExtensionConfig`) that lets any extension ask the host "what's in my own `ExtensionConfig.config`", the same "generic fact query, no special-casing" pattern AWU 915/916 established for `GetActiveLlmProfile`. This resolved 48-1 and 48-2 as one fix: the RPC *is* the wiring 48-2 asked for, and `security-guard` consuming it *is* 48-1's real policy mechanism.
- **Host plumbing**: Added `GetExtensionConfig` to `models/src/lib.rs`'s `RasRpcCommand` and `wit/rad.wit`; the Phase-46 shared macros (`rpc_conversion.rs`) picked up both directions with a 2-line addition each, confirming the consolidation's payoff. Threaded a new `caller_name: &'a str` field through `RpcContext` (`src/wasm/rpc.rs`) and its one construction site (`src/wasm/imports_rpc.rs`, passing `&self.name`), so `rpc_meta::handle_meta`'s new arm can look up `orch.config.lock().extensions.iter().find(|e| e.name == caller_name)` and return that extension's `config` map as JSON (empty object if unregistered or no orchestrator, e.g. the lower-level `WasmRuntime` unit-test harness).
- **`security-guard` fix**: Added `ext/security-guard/src/policy.rs` — a `Policy { block_path_patterns, block_command_patterns }` fetched once via `GetExtensionConfig` and cached in a `thread_local!` `RefCell` for the lifetime of the component instance (avoids a host round-trip on every `verify_rpc` call). Empty/missing config means "block nothing" — the policy is opt-in, not a hardcoded fallback. `lib.rs` gained the core-to-wit conversion direction it previously didn't need (`impl_rpc_target_core_to_wit!`, `impl_rpc_timeout_policy_core_to_wit!`, and a hand-written `From<CoreRpcCommand> for wit::RasRpcCommand` residual for `OpenFile`/`OpenProcess`, mirroring `rad-orchestrator`'s `conv/rpc.rs` pattern) since it now constructs a command to send, not just receives one.
- **Test fallout (3 files needed the same fix)**: Making the policy opt-in broke every existing test that relied on the old hardcoded literals with an empty `config: HashMap::new()` — `src/wasm/tests.rs` (`test_verify_rpc_blocked_file`, `test_verify_rpc_blocked_command`), `tests/multi_extension_tests.rs` (both tests), `tests/security_hook_tests.rs`. Fixed by explicitly configuring `block_path_patterns`/`block_command_patterns` on each security-guard-role `ExtensionConfig` — `src/wasm/tests.rs`'s harness needed a real (if minimal) `Orchestrator` wired into `WasmRuntime::new`'s `orchestrator: Option<Weak<Orchestrator>>` parameter for the first time, since `verify_rpc` now round-trips through host config lookup. Added one new test proving the opt-in direction too — `test_security_guard_blocklist_is_opt_in_and_blocks_nothing_when_unconfigured` — split into its own `tests/security_guard_policy_tests.rs` file rather than growing `multi_extension_tests.rs` further past CODING.md's 300-line limit (that file was already over at 336 lines pre-existing this AWU; left as a known, not-worsened violation rather than a full re-split, since restructuring its two already-passing integration tests was out of scope here).
- **Scope**: `models/src/lib.rs`, `wit/rad.wit`, `models/src/rpc_conversion.rs`, `src/wasm/rpc.rs`, `src/wasm/imports_rpc.rs`, `src/wasm/rpc_meta.rs`, `ext/security-guard/src/lib.rs`, `ext/security-guard/src/policy.rs` (new), `src/wasm/tests.rs`, `tests/multi_extension_tests.rs`, `tests/security_hook_tests.rs`, `tests/security_guard_policy_tests.rs` (new), `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, for both the native host target and `wasm32-wasip2` extension crates.
- **Result**: Success. 46 `rad` lib unit tests + all integration test binaries (including the 3 security-related ones) pass; Clippy clean (`-D warnings`) on both native and `wasm32-wasip2` targets across `rad`, `rad-models`, `security-guard`, `rad-orchestrator`, `context-tools`, `llm-connector`, `mcp-tool-provider`.

---

## 🛠️ Short-Term Plan: Phase 47

### 💡 Current AWU Status
- [✅] AWU 917: Top-Level Slash Command Registry (47-1) (Result: Success)
- [✅] AWU 918: `/llm` Subcommand Registry (47-1b) & Fix Test Pollution of Real `~/.rad/config.json` (Result: Success)
- [✅] AWU 919: Markdown-Template Commands (47-2) & Eager Extension Loading (47-3) — Phase 47 Complete (Result: Success)

### 📝 AWU Details

#### AWU 917: Top-Level Slash Command Registry (47-1)
- **Trigger**: Phase 47 item 47-1. `src/command.rs`'s `Command` enum, `CommandParser::parse`, `CommandManager::execute`, and hand-written `/help` text were 4 independently-maintained sources of truth for the command list; a 5th (`CommandHelper`'s tab-completion array in `src/command/completion.rs`) was discovered while implementing this and had drifted furthest — it never listed `/llm` at all.
- **Fix**: Replaced the `Command` enum with `ParsedCommand { name: &'static str, args: String }` (canonical name + raw trailing text) and a `CommandSpec { name, aliases, description, handler: fn(&str, &Arc<Orchestrator>) -> CommandResult }` registry (`command_specs()`). `CommandParser::parse` now looks up the registry instead of a hand-written match; `CommandManager::execute` dispatches to `spec.handler`; `/help`'s text is generated by iterating `command_specs()`; `CommandHelper`'s completion list now iterates the same registry (including aliases), fixing the missing-`/llm` gap.
- **Incidental fixes surfaced while porting each handler** (preserved-behavior parity was the goal, but two handlers were clearly broken and got fixed since they were being rewritten anyway): `/session` previously required an argument and echoed it back verbatim (`/session foo` → "Current session: foo" regardless of `foo`'s value) instead of showing the actual session ID, and `/session` with no argument fell through to being sent to the LLM as a task; now shows the real `orchestrator.session_id` unconditionally. `/rollback` with no node ID previously also fell through to being sent to the LLM as a task instead of erroring; now shows a `Usage: /rollback <node_id>` message.
- **Explicitly not fixed by this AWU** (see the corrected Phase 47 scope note above): `/llm`'s own internal issues — positional-only `add`, `test`/`add`/`model` shadowing profiles of the same name, no `delete`, incomplete internal help in `render_llm_profiles` — are untouched; `cmd_llm` just delegates to the pre-existing `llm::parse_llm_command`/`LlmSubcommand` unchanged. Applying the same registry pattern to `/llm`'s own subcommands is separate, unstarted follow-up work.
- **Scope**: `src/command.rs` (rewritten), `src/command/completion.rs`, `src/main.rs` (one call-site signature update), `tests/command_tests.rs` (rewritten for the new `ParsedCommand`/registry API).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines.
- **Result**: Success. `src/command.rs` is 281 lines (was 236 — net growth from per-command handler functions replacing match arms, still under the 300-line limit). Host `cargo build`/`clippy --lib` clean. `command_tests.rs` rewritten and passing (3 tests, including a new completion assertion that `/llm`/`/models` are now suggested). Full suite verified: host `cargo test --lib` (40 tests) and integration suite (`command`/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 17 tests total) all pass with no regressions.

#### AWU 918: `/llm` Subcommand Registry (47-1b) & Fix Test Pollution of Real `~/.rad/config.json`
- **Trigger**: Phase 47 item 47-1b, per the corrected scope note left by AWU 917 (user explicitly asked for this residual work before the deferred broader command-composition review).
- **47-1b fix**: Replaced `LlmSubcommand`/`parse_llm_command`/`execute_llm_command` with an `LlmSubcommandSpec { name, usage, handler: fn(&str, &Orchestrator) -> String }` registry (`llm_subcommand_specs()`) and a single `execute_llm(args, orchestrator)` entry point, mirroring AWU 917's top-level pattern one level down. Fixed all four residual issues:
  1. **Reserved-keyword/profile-name collision**: `execute_llm` now checks whether the first word matches a *registered profile name* (or a numeric index) before checking the subcommand table, so a profile literally named `test`/`add`/`model`/etc. is switched to directly; `/llm switch <name>` remains an explicit, unambiguous escape hatch either way.
  2. **Added `/llm delete <name>`** (`delete_llm_profile` in `profile_admin.rs`) — removes a profile, falling `active` back to another remaining profile (or `None`) if the deleted one was active.
  3. **`add` switched from strict positional args to flags**: `/llm add <name> <url> [--model <m>] [--api-key <k>]`, fixing the inability to set `api_key` without `model`. Accepted as a breaking change to the old 4-positional form (pre-1.0 internal tool).
  4. **`render_llm_profiles`'s help footer now always lists every subcommand's usage** (generated from `llm_subcommand_specs()`), not just conditionally showing `add`'s usage when the endpoint list was empty.
- **Critical bug found while writing `tests/llm_command_tests.rs`**: `save_global_config` (in `profile_admin.rs`) unconditionally wrote to the *real* `dirs::home_dir()/.rad/config.json` with no test seam. The new tests — the first ones in this repo to exercise `/llm add`/`test`/`model`/`delete` — clobbered the developer's actual global config with tempdir workspace paths and dummy `test`/`primary` profiles pointing at `http://127.0.0.1:1`. Disclosed to the user immediately on discovery; **the original content could not be recovered (no backup existed)**.
- **Root-cause fix**: Added `crate::config::global_config_path()` in `src/config.rs` — the single shared resolver for `~/.rad/config.json`, honoring a `RAD_TEST_CONFIG_HOME` override — and routed both the read side (`load_config`'s three independent `dirs::home_dir()` call sites in `config/load.rs`) and the write side (`save_global_config`) through it. Previously these resolved the same conceptual path independently via separate `dirs::home_dir()` calls with no shared test seam at all. This also fixed a second, pre-existing latent bug it surfaced: `src/config/tests.rs`'s `test_load_config_default_when_no_file`/`test_load_config_with_local_override` had always implicitly depended on the real machine's `~/.rad/config.json` being absent-or-default (since `load_config` always reads it as a base layer, even with an explicit path given) — invisible until this session's corruption made `test_load_config_default_when_no_file` fail. Both tests now set `RAD_TEST_CONFIG_HOME` to a per-test tempdir, serialized via a new `CONFIG_TEST_MUTEX` (env vars are process-global; `cargo test` runs same-binary tests in parallel threads by default) — same pattern `tests/multi_extension_tests.rs` already used for its own global state.
- **New test coverage**: `tests/llm_command_tests.rs` (8 tests) — flag-based `add` with/without `model`, unrecognized-flag rejection, `delete` (success/not-found/active-profile-fallback), profile-name-vs-keyword priority (including the `switch` escape hatch), and the always-populated help footer. Every test holds a `TEST_MUTEX` (matching `multi_extension_tests.rs`) for its full duration and sets `RAD_TEST_CONFIG_HOME` to its own tempdir, so this suite cannot repeat the pollution incident.
- **Scope**: `src/command/llm.rs` (rewritten), `src/command/llm/profile_admin.rs` (`delete_llm_profile` added, `save_global_config` fixed), `src/config.rs` (`global_config_path` added), `src/config/load.rs` (3 call sites unified), `src/config/tests.rs` (2 tests made hermetic), `src/command.rs` (`cmd_llm` simplified), `tests/llm_command_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; the real `~/.rad/config.json` is verifiably untouched by a full test run (checksummed before/after).
- **Result**: Success. Host `cargo build`/`clippy --lib` clean. `src/config.rs` 203, `config/load.rs` 118, `config/tests.rs` 223, `command/llm.rs` 174, `command/llm/profile_admin.rs` 216, `tests/llm_command_tests.rs` 185 lines — all under the 300-line limit. Full suite: host `cargo test --lib` (40 tests, including the now-fixed `test_load_config_default_when_no_file`), and integration suite (`command`/`llm_command` (new, 8 tests)/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 25 tests total) all pass. `~/.rad/config.json`'s checksum confirmed identical before and after the full run (no further pollution) — but its content from before this incident remains lost.

#### AWU 919: Markdown-Template Commands (47-2) & Eager Extension Loading (47-3) — Phase 47 Complete
- **Trigger**: User asked to complete Phase 47 in full. Remaining items: 47-2 (markdown-template command tier), 47-3 (eager extension loading), and a decision on 47-4.
- **47-2 fix**: Added `src/command/templates.rs` — `expand(workspace, name, args)` looks up `<name>.md` in `.agents/commands/` (project-local, checked first) then `~/.rad/commands/` (user-global), substitutes a `$ARGUMENTS` placeholder if present, else appends `args` on a new line (or returns the template unchanged if `args` is empty and there's no placeholder). `list_names(workspace)` enumerates available template names for `/help`. Since Rust's static `command_specs()` function-pointer table can't hold entries discovered at runtime from the filesystem, this is a genuinely separate lookup tier rather than one literal array — `process_input` (`main.rs`) tries the built-in registry first, then `templates::expand` (via a new shared `command::split_slash` helper factored out of `CommandParser::parse`), before falling through to the pre-existing "send raw input as a task" behavior for a truly unrecognized `/whatever`. Added `CommandResult::RunTask(String)` as a first-class variant for "this command expanded into a task prompt" rather than piping it through a template-specific side channel. `/help` now also lists discovered template names.
- **47-3 fix**: `main()` now calls `orchestrator.get_or_init_runtimes(&throwaway_tx)` immediately after constructing the `Orchestrator`, instead of waiting for the first task or `/tools` to trigger it lazily. `get_or_init_runtimes` is idempotent (skips already-loaded extensions by name), so this doesn't change the real per-task load path that follows later (`runner.rs` still re-applies `set_event_tx` with the real per-task channel) — it just moves the one-time ~150ms-class cost earlier and surfaces a broken extension's diagnostics (e.g. an MCP tool-provider failing to find its server config) at startup instead of silently deferring them to the first time a task actually needs that extension.
- **47-4 decision**: Not implemented. Re-evaluated per this item's own stated condition (after 47-2 shipped) and found no concrete consumer: none of the 5 current extensions need a UI-facing command, and markdown templates already serve the demand this was meant to address (user-defined prompt shortcuts). Documented as deliberately deferred rather than built speculatively — revisit only when a real extension needs more than "expand into a task prompt."
- **Manual verification side-incident**: Testing `/hello world` (a throwaway template) triggered the orchestrator's own git-autopilot feature, which created and checked out a new branch (`rad-autopilot-1785133346`) carrying this session's entire uncommitted working tree with it. Confirmed via `git diff`/`git log` that the new branch had zero unique commits (identical tip to the working branch `rad-autopilot-1785051822`) before switching back and deleting the stray branch — no work was at risk, but this is a reminder that manually running `rad` for verification during a `rad`-development session can itself mutate git state.
- **Scope**: `src/command/templates.rs` (new), `src/command/templates/tests.rs` (new, 6 tests), `src/command.rs` (`CommandResult::RunTask`, `split_slash` extracted, handlers moved out), `src/command/handlers.rs` (new — built-in handler fns relocated here to stay under the 300-line limit after `templates` integration), `src/main.rs` (`process_input`'s template tier, `run_task_and_save` extracted, eager-load call in `main()`).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; manually verified end-to-end (`/help` lists a test template, `/name args` expands and runs as a task, all 5 extensions load at startup with diagnostics visible before the first task).
- **Result**: Success. `src/command.rs` 203, `src/command/handlers.rs` 112, `src/command/templates.rs` 71, `src/main.rs` 228 lines — all under the 300-line limit. Host `cargo build`/`clippy --lib` clean. Host `cargo test --lib` (46 tests, +6 new template tests) and integration suite (`command`/`llm_command`/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 25 tests) all pass. Manual verification: `rad` startup now shows `[FAILED] Extension 'mcp-tool-provider' get_tools error: ...` immediately (previously only visible after the first task); `/help` correctly listed a test `.agents/commands/hello.md` template under "Custom commands"; `/hello world` expanded to the template body with `$ARGUMENTS` substituted and ran as a real task through the full orchestrator pipeline. Test artifacts (`.agents/commands/hello.md`, the stray git branch) cleaned up after verification.

---

## 🛠️ Short-Term Plan: Phase 46

### 💡 Current AWU Status
- [✅] AWU 915: RPC Conversion Macro Consolidation, `check_endpoint()` Unification & State Constant Dedup (Result: Success)
- [✅] AWU 916: `context-tools` WIT Unification (46-2) — Discovered & Fixed a Silent Windowing Bug (Result: Success)

### 📝 AWU Details

#### AWU 915: RPC Conversion Macro Consolidation, `check_endpoint()` Unification & State Constant Dedup
- **Trigger**: Phase 46 of the 2026-07-27 long-term plan (items 46-1, 46-3, 46-4; 46-2 deferred as the next unit of work since it depends on 46-1).
- **46-1 fix**: Added `models/src/rpc_conversion.rs` with `#[macro_export]`'d `macro_rules!` definitions (`impl_rpc_target_wit_to_core!`/`_core_to_wit!`, `impl_rpc_timeout_policy_wit_to_core!`/`_core_to_wit!`, `impl_rpc_command_wit_to_core!`, `common_rpc_command_core_to_wit!`) generating the `RasRpcCommand`/`Target`/`TimeoutPolicy` wit↔core conversion boilerplate that was hand-copied across `src/wasm/bindings/rpc_command.rs`, `ext/security-guard/src/lib.rs`, `ext/mcp-tool-provider/src/conv.rs`, and `ext/rad-orchestrator/src/conv.rs`/`conv/rpc.rs`. `common_rpc_command_core_to_wit!` returns `Option` and omits `OpenFile`/`OpenProcess` (no WIT equivalent — host-only normalization variants), leaving that residual to each site (host converts for real; the 3 guest crates `panic!()`/`unreachable!()` since they never construct those). Each site now invokes only the macros for the direction(s) it actually needs (e.g. `security-guard` only wit-to-core, since `verify_rpc` inspects incoming commands but never constructs one). Hit and fixed one macro-hygiene issue mid-flight: `$wit:path` fragments produced "expected type, found module" errors when used as a path prefix (`$wit::Target`) in item position; switched to `$wit:ident` (every call site passes a single identifier — a local `use ... as wit` alias — so `path` fragment's extra generality wasn't needed and was the source of the parse ambiguity).
- **46-3 fix**: Added `EndpointStatus`/`check_endpoint()` in `src/command/llm/context_length.rs`, replacing the two previously-separate `probe_endpoint`(liveness)/`detect_context_length`(context window) call patterns. `check_endpoint` skips context-length detection when the liveness probe fails (avoids a second connect-timeout wait for a server already known down) and is now used uniformly by `main.rs`'s startup health check, `/llm add`, `/llm model`, and `/llm test` — closing the gap where startup never detected context length at all, and giving `/llm add`/`/llm model` a liveness signal they didn't previously have. `probe_endpoint`/`detect_context_length` are now private to `context_length.rs`.
- **46-4 fix**: Added `DEFAULT_MAX_HISTORY_MESSAGES`/`DEFAULT_MAX_TOOL_OUTPUT_CHARS` constants in `ext/rad-orchestrator/src/orchestrator.rs`, replacing the `Some(50)`/`Some(2000)` literals duplicated in `handle_human_input` and `handle_rehydrate`.

#### AWU 916: `context-tools` WIT Unification (46-2) — Discovered & Fixed a Silent Windowing Bug
- **Trigger**: Phase 46 item 46-2, now unblocked by AWU 915. Unify `context-tools`' bespoke single-variant `wit/context-tools.wit` (`package radcomp:context-tools`, `host-rpc` importing a fake `command(string)` "RPC") onto the shared `rad.wit` `radcomp:extension` package's real `ras-rpc-command`, so its declared `PermissionConfig.fs_read_allow`/`fs_write_allow` finally mean something instead of being silently ignored by a raw `sh -c` bridge.
- **WIT restructuring**: Changed `context-tools.wit`'s package to `radcomp:extension` and deleted its own `types`/`host-rpc` interfaces (reused rad.wit's). Since `wasm_bindgen`/`wasmtime::component::bindgen!`'s directory-scan mode requires every `.wit` file in a scanned directory to share one package (or use the `deps/` convention), and `wit/` also held `llm-connector.wit` (a third, unrelated `radcomp:connector` package), moved that file to `wit/connector/llm-connector.wit` — the two remaining call sites (`src/wasm/bindings.rs`, `ext/llm-connector/src/lib.rs`) updated to the new path. `rad_context_tools`'s host bindgen and the guest's `wit_bindgen::generate!` now both point `path` at the `wit` directory (not a single file) with `world: "context-tools-extension"`, plus a `with: { "radcomp:extension/types": ... }` mapping on the host side so its `RasRpcCommand` is the same Rust type every other extension's bindings use.
- **Host bridge**: Deleted the special-cased shell-command `impl ...host_rpc::Host for WasmState` in `src/wasm/imports_delegate.rs` entirely; `context-tools` now goes through `delegate_extension_imports!(bindings::rad_context_tools::ContextToolsExtensionImports, rpc_only)`, the same generic (permission-checked, security-guard-verified) `host_rpc` path every other extension uses.
- **`get-repo-map` upgrade**: Since `RasRpcCommand::GetRepoMap` (the real semantic/tree-sitter repo map, already used by other extensions) is now reachable, `MyContextTools::get_repo_map` calls `host_rpc(&RasRpcCommand::GetRepoMap)` instead of shelling out to `tree -L 2`.
- **Host-side call-site fallout**: `src/wasm.rs`'s `call_extension_method("optimize"/"get-repo-map")` updated from `rad_context_tools::exports::radcomp::context_tools::context_tools::...`/`.radcomp_context_tools_context_tools()` to `...::radcomp::extension::context_tools::...`/`.radcomp_extension_context_tools()` (package-rename fallout; `loader.rs`'s `ContextToolsExtension` struct name is derived from the world name, not the package, so it needed no change).
- **New integration coverage, and a real bug it caught**: No integration test had ever loaded `context-tools` as an extension before (confirmed by grep — `ext/context-tools`'s own tests only exercise guest-side logic natively, never through the real WASM component boundary). Added `tests/context_tools_tests.rs`, driving `context-tools.wasm` through `WasmRuntime::call_extension_method` directly. Writing it surfaced a **pre-existing, silent bug dating to AWU 908**: `ext/rad-orchestrator/src/llm.rs`'s `CtMessage`/`CtOptimizationRequest`/`CtOptimizationResponse` used `#[serde(rename = "node-id")]`-style kebab-case renames, on the incorrect assumption that the host's `additional_derives: [serde::Serialize, serde::Deserialize]`-generated structs serialize kebab-case (matching the WIT source spelling). They don't — wasmtime's component bindgen uses the plain (already-snake_case) Rust field names with no rename. Since `serde_json` silently ignores unrecognized JSON keys and defaults absent `Option<T>` fields to `None`, every `optimize` request `rad-orchestrator` ever sent had `max_history`/`max_content_chars`/`node_id` silently deserialize to `None`/`null` on the host side — **count-based windowing (AWU 908) and this session's size-based windowing had never actually windowed anything in the real end-to-end path**, always hitting `optimize`'s "no history, nothing to do"/"no messages were compressed" branch regardless of history size. Removed the incorrect renames (verified by round-tripping real snake_case JSON through `call_extension_method` in the new test — windowing now measurably fires, e.g. "Windowed history from 2 to 1 messages").
- **Scope**: `wit/context-tools.wit`, `wit/connector/llm-connector.wit` (moved), `src/wasm/bindings.rs`, `src/wasm/imports_delegate.rs`, `src/wasm.rs`, `ext/context-tools/src/lib.rs`, `ext/context-tools/src/tests.rs`, `ext/llm-connector/src/lib.rs`, `ext/rad-orchestrator/src/llm.rs`, `ARCHITECTURE.md`, `tests/context_tools_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; the real windowing round-trip is covered by an integration test through the actual WASM component boundary (not just native unit tests of guest-side logic).
- **Result**: Success. All 5 extensions build and pass `clippy --all-targets` cleanly under `wasm32-wasip2`; host `cargo build`/`clippy --lib` clean; `rad-models` (5 tests), `context-tools` (8 native unit tests), host `cargo test --lib` (40 tests), and the full integration suite (`context_restoration`/`context_tools` (2 new)/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 15 tests total) all pass. `ARCHITECTURE.md` §1.3.5 updated to describe size-based windowing and the `get-repo-map` semantic-map delegation.
- **File-size fallout**: `src/main.rs` grew to 302 lines from the `check_endpoint` integration (over CODING.md's 300-line limit). Split `load_config_and_session`/`check_active_llm_endpoint` into a new `src/startup.rs` (a binary-crate-root submodule — `main.rs` is its own crate root distinct from `lib.rs`, so its submodules resolve to plain `src/<name>.rs`, not the `src/<file>/<name>.rs` pattern used for `lib.rs` submodules like `config.rs`/`config/load.rs`). `main.rs` is now 192 lines, `startup.rs` 115.
- **Scope**: `models/src/lib.rs`, `models/src/rpc_conversion.rs` (new), `src/wasm/bindings.rs`, `src/wasm/bindings/rpc_command.rs`, `ext/security-guard/src/lib.rs`, `ext/mcp-tool-provider/src/conv.rs`, `ext/rad-orchestrator/src/conv.rs`, `ext/rad-orchestrator/src/conv/rpc.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `src/command/llm.rs`, `src/command/llm/context_length.rs`, `src/command/llm/profile_admin.rs`, `src/main.rs`, `src/startup.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines.
- **Result**: Success. Line-count reductions from the macro consolidation alone: host `rpc_command.rs` 253→37, `security-guard/lib.rs` 159→46, `mcp-tool-provider/conv.rs` 196→25, `rad-orchestrator` `conv.rs`+`conv/rpc.rs` 71+293→34+24. All 5 extensions build and pass `clippy --all-targets` cleanly under `wasm32-wasip2`; host `cargo build`/`clippy --lib` clean; `rad-models` (5 tests), `context-tools` (8 tests), host `cargo test --lib` (40 tests), and the full integration suite (`context_restoration`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 13 tests) all pass with no regressions — `multi_extension_tests.rs` specifically exercises `security-guard`'s `verify_rpc` end-to-end through the new macro-generated conversion.

---

## 🛠️ Short-Term Plan: Phase 45

### 💡 Current AWU Status
- [✅] AWU 914: Fix Hardcoded LLM Model Name in `GenerateLlmStream` (Result: Success)

### 📝 AWU Details

#### AWU 914: Fix Hardcoded LLM Model Name in `GenerateLlmStream`
- **Trigger**: A 2026-07-27 design review of local-LLM context-overflow prevention (which shipped `context-tools` size-aware windowing, the `GetActiveLlmProfile` host RPC, and `/llm` context-length detection earlier the same session) surfaced a separate, pre-existing bug while auditing the same file: `ext/rad-orchestrator/src/llm.rs::trigger_llm_stream` sent a hardcoded `model: "qwen".to_string()` to `GenerateLlmStream` regardless of the active LLM profile's configured model — silently defeating `/llm add <name> <url> <model>` and `/llm model <new_model>` for any backend that actually routes on the request's `model` field (Ollama, LM Studio, vLLM; llama.cpp with a single loaded model happened to ignore it, masking the bug).
- **Fix**: Added `active_llm_model()` in `ext/rad-orchestrator/src/llm.rs`, mirroring the existing `active_llm_context_length()` pattern — both query the same `GetActiveLlmProfile` host RPC (built earlier this session) for a different field. `trigger_llm_stream` now sends the real configured model, falling back to an empty string (not a guessed model name) when none is configured, so a misconfiguration surfaces as a clear server-side error instead of a silent wrong-model request.
- **Scope**: `ext/rad-orchestrator/src/llm.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. `rad-orchestrator` builds and passes `clippy --all-targets` cleanly under `wasm32-wasip2`; full host `cargo test --lib` (40 tests) and integration suite (`context_restoration`/`e2e`/`hitl`/`mcp`/`repo_map`/`self_healing`/`tool_loop`, 11 tests) pass with no regressions. `ext/rad-orchestrator/src/llm.rs` remains at 288 lines (CODING.md's 300-line limit).

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
