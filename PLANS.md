# PLANS

## Objective
Establish a comprehensive roadmap to build `rad` (Rust Agent Dispatcher) Version 0.1 as a production-ready agent runner, incorporating process isolation, filesystem safety, WebAssembly plugins, PTY support, and streaming LLM API connection.

---

## Detailed Version 0.1 Implementation Plan

```mermaid
graph TD
    AWU0[AWU 0: Security Rules & Secret Leak Prevention Harness] --> AWU1[AWU 1: Project Setup & Config Loader]
    AWU1 --> AWU2[AWU 2: Process Isolation & PGID Cleanup]
    AWU2 --> AWU3[AWU 3: Filesystem Sandbox & Snapshots]
    AWU3 --> AWU4[AWU 4: DAG Tracking & JSON IPC Bridge]
    AWU4 --> AWU5[AWU 5: WebAssembly Runtime Integration]
    AWU5 --> AWU6[AWU 6: PTY Support & Filesystem Watcher]
    AWU6 --> AWU7[AWU 7: HTTP Streaming Client & Timeouts]
    AWU7 --> AWU8[AWU 8: Security Audit & Integration Tests]
```

### Atomic Work Units (AWUs)

* **AWU 0: Security Rules & Secret Leak Prevention Harness**
  - Define rules in `CODING_RULES.md` and `.agents/AGENTS.md` to prevent committing secrets (API keys, tokens) and local absolute paths.
  - Implement a verification script `scripts/check_secrets.sh` to scan for secrets and absolute paths.
  - Install a Git `pre-commit` hook that runs the verification script on staged changes.
  - Add the verification script run to the Strict Audit phase in `.agents/AGENTS.md`.
* **AWU 1: Project Setup & Configuration Parser**
  - Setup Cargo project structure and define core data structures.
  - Implement comments-supported (JSONC) loader merging `rad.json` and `rad.local.json`.
* **AWU 2: Process Subsystem with PGID Isolation**
  - Spawn child bash processes under dedicated PGIDs.
  - Implement a `Drop` manager to automatically force-terminate (`SIGKILL`) process groups on exit/panic.
* **AWU 3: Filesystem Subsystem, Sandbox & Snapshots**
  - Implement safe file primitives (`read`, `write`, `patch`) with normalized path capability checks.
  - Track and restore physical snapshots associated with DAG nodes under `.rad/snapshots/`.
* **AWU 4: DAG Tracking & JSON IPC Bridge**
  - Manage session history in-memory using a Directed Acyclic Graph.
  - Implement a dual-channel JSON IPC bridge mapping Core events (`RasCoreEvent`) and Extension commands.
* **AWU 5: WebAssembly Runtime Integration**
  - Add `wasmtime` (version 29 or stable equivalent compatible with edition 2024) to `Cargo.toml`.
  - Create a new module `src/wasm.rs` to manage the WebAssembly runtime execution.
  - Implement memory allocation helper methods to transfer data between host and guest.
  - Define guest functions to export (`rad_on_event`, `alloc`, `dealloc`).
  - Implement host import function `rad_host_rpc` which takes an RPC JSON request from the guest, verifies capabilities against `PermissionConfig`, forwards the command to Core subsystems (FsSandbox, ProcessManager, Dag), and returns the serialized `RasRpcResponse` back to the guest.
  - Write integration and unit tests in `src/wasm/tests.rs` (using a mock/test Wasm module or compiling a minimal Wasm guest dynamically if possible, or using a pre-compiled test Wasm bytes embedded in tests).
* **AWU 6: PTY Allocation & Reactive Sensors**
  - Integrate PTY (pseudoterminal) allocation to run interactive shells and capture raw terminals.
  - Implement filesystem monitoring via `notify` crate.
* **AWU 7: HTTP Streaming Client & Dynamic Timeouts**
  - Add asynchronous dependencies (`reqwest`, `tokio`, `futures-util`) to enable stream connection to OpenAI/Anthropic.
  - Implement dynamic timeout monitoring structure (`HttpStreamClient`) using shared state (`Arc<Mutex<TimeoutPolicy>>`) for real-time heartbeat and connection timeout.
  - Update `RunningProcess` and host RPC handling to spawn background polling threads for process stdout/stderr capture and inactivity timeout.
  - Expose `OpenHttpStream` and `SetStreamTimeoutPolicy` RPC methods, bridging streaming tokens and timeout triggers back to the Extension through `RasCoreEvent`.
* **AWU 8: Security Audit & E2E Integration Tests**
  - Conduct path traversal prevention audit.
  - Run comprehensive integration tests verifying the full flow (Wasm Extension -> PTY command -> file edit -> snapshot -> rollback -> cleanup).
  - Verify zero Clippy warnings and test pass.
* **AWU 9: GitHub Actions CI Setup**
  - Create `.github/workflows/ci.yml` to build and test the project on main target platforms (Ubuntu, macOS, Windows).
  - Enforce Rust toolchain (stable, compatible with edition 2024).
  - Run `cargo check`, `cargo clippy`, and `cargo test` on all target platforms in CI.
  - Integrate secret and absolute path check script into the workflow to ensure commit safety.

---

## Detailed Version 0.2 Implementation Plan (Single-Process Agent Shell / REPL)

```mermaid
graph TD
    AWU10[AWU 10: In-Process REPL Loop & Stdin Monitoring] --> AWU11[AWU 11: Real-time Event-to-Stdout Streaming Router]
    AWU11 --> AWU12[AWU 12: In-Process Human-in-the-Loop Approval Prompt]
    AWU12 --> AWU13[AWU 13: Session State Persistence & DAG Recovery]
    AWU13 --> AWU14[AWU 14: Autonomous Execution Loop Integration]
    AWU14 --> AWU15[AWU 15: Single-Process E2E Integration Tests & Zero-Warning Audit]
```

### Atomic Work Units (AWUs)

* **AWU 10: In-Process REPL Loop & Stdin Monitoring**
  - Implement a terminal-based REPL loop in `src/main.rs` that defaults to showing `rad > ` when launched with no subcommands.
  - Parse CLI arguments, routing execution flow directly into the interactive REPL.
* **AWU 11: Real-time Event-to-Stdout Streaming Router**
  - Route Wasm Extension events (like `TokenReceived` and process execution stdout/stderr) directly to the active terminal's stdout/stderr in real-time.
  - Ensure zero network socket overhead by executing Wasm and OS primitives within the same single process.
* **AWU 12: In-Process Human-in-the-Loop Approval Prompt**
  - Intercept privileged RPC commands (e.g. executing commands, writing files) to request manual confirmation.
  - Present approval requests directly on the active terminal (`Approve? (y/n): `) and halt execution synchronously until the user responds.
* **AWU 13: Session State Persistence & DAG Recovery**
  - Implement JSON serialization for DAG state.
  - Save sessions to `.rad/sessions/` on changes and add support for reloading session on startup via CLI (e.g. `rad --session <session_id>`).
* **AWU 14: Autonomous Execution Loop Integration**
  - Integrate orchestrator loop inside the REPL thread, ensuring LLM calls keep running autonomously until task goal is met or human interaction is triggered.
* **AWU 15: Single-Process E2E Integration Tests & Zero-Warning Audit**
  - Add comprehensive E2E tests validating the REPL flow (startup -> stdin task -> auto stream output -> human approval -> completion).
  - Run clippy, tests, secret checks, and achieve zero warning status.

