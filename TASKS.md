# TASKS

- [x] Create `CONFIG.md`
- [x] Simplify `README.md` and link to `CONFIG.md`
- [x] Remove 'Honke' keyword from README.md
- [x] Remove configuration section from README.md
- [x] Create `ARCHITECTURE.md` based on the specified architecture
- [x] Configure remote origin, tag `v0.0.0`, and push to GitHub
- [x] Unify configuration file to `rad.json` and update `CONFIG.md` and `ARCHITECTURE.md`
- [x] Add global path layout rules based on the recommended proposal to `CONFIG.md`
- [x] Refine `rad.json` schema to be minimal, update `CONFIG.md`, and create `rad.json` at root
- [x] Create full `rad.json` with explanatory comments, and update `CONFIG.md` to support JSONC
- [x] Delete `version` field from both `rad.json` and `CONFIG.md`
- [x] Integrate `rad.json` into `CONFIG.md` and delete `rad.json`
- [x] Audit all files for consistency and redundancy (specifically ARCHITECTURE.md vs CONFIG.md schemas)

## Version 0.1 Implementation Phase (Atomic Work Units)
- [x] AWU 0: Security Rules & Secret Leak Prevention Harness
- [x] AWU 1: Project Setup & Configuration Parser
  - [x] Initialize Cargo project structure (`Cargo.toml`, `src/main.rs`, `src/config.rs`)
  - [x] Add dependencies (`serde`, `serde_json`, `jsonc-parser`, `dirs`, `clap`)
  - [x] Define Rust configuration structs (`Config`, `CoreConfig`, `TimeoutConfig`, `ExtensionConfig`, `PermissionConfig`, etc.) based on `CONFIG.md`
  - [x] Implement XDG/Windows configuration lookup logic
  - [x] Implement JSONC loading and merging for `rad.local.json`
  - [x] Write integration and unit tests in companion files (e.g., `src/config/tests.rs`)
  - [x] Ensure all code adheres to `#![deny(clippy::pedantic)]` and passes Clippy, check_secrets.sh, and cargo test
- [x] AWU 2: Process Subsystem with PGID Isolation & Group Cleanup
  - [x] Add `nix` crate to `Cargo.toml` for safe OS/POSIX primitive calls
  - [x] Implement `ProcessManager` structure with `Drop` implementation to manage active PGIDs
  - [x] Implement isolated process spawning logic (`spawn_bash_process`) using `setpgid` from the parent process to avoid `unsafe`
  - [x] Implement stdout/stderr tracking and dynamic timeout monitoring helper structures
  - [x] Write companion tests in `src/process/tests.rs` to verify process group creation, signal killing on drop, and stdout/stderr capture
  - [x] Verify compliance with `#![deny(clippy::pedantic)]`, check_secrets.sh, and cargo test
- [x] AWU 3: Filesystem Sandbox & Snapshot Backup/Restoration
  - [x] Implement `FsSandbox` with normalized path verification and permission checks (`fs_read_allow`, `fs_write_allow`)
  - [x] Implement safe file primitives: `file_read`, `file_write`, and `file_edit_patch` (using diff/patch)
  - [x] Implement snapshot creation (`take_snapshot`) backing up specified files to `.rad/snapshots/<node_id>/`
  - [x] Implement snapshot restoration (`checkout_snapshot`) restoring workspace files from `.rad/snapshots/<node_id>/`
  - [x] Write unit tests for filesystem sandbox and snapshots in `src/fs/tests.rs`
  - [x] Verify compliance with `#![deny(clippy::pedantic)]`, check_secrets.sh, and cargo test
- [x] AWU 4: DAG Tracking & Dual-Channel JSON IPC Bridge
  - [x] Implement `Dag` structure managing `DagNode` history in `src/dag.rs`
  - [x] Implement DAG operations: `create_node`, `set_node_text`, `merge_nodes`, `delete_node`
  - [x] Define communication protocols (`RasCoreEvent`, `RasRpcCommand`, `RasRpcResponse`) in `src/ipc.rs`
  - [x] Implement `IpcBridge` utilizing JSON Lines over `Read` / `Write` streams
  - [x] Add comprehensive tests in `src/dag/tests.rs` and `src/ipc/tests.rs`
  - [x] Verify compliance with `#![deny(clippy::pedantic)]`, check_secrets.sh, and cargo test
- [x] AWU 5: WebAssembly Runtime Integration (wasmtime)
  - [x] Add `wasmtime = "29.0.0"` (or compatible version) to `Cargo.toml`
  - [x] Create `src/wasm.rs` and its internal helper structures (`WasmRuntime`, `WasmState`)
  - [x] Implement Wasm Guest Memory management helpers (alloc/dealloc invocation from Host)
  - [x] Implement Host Imports (`rad_host_rpc`) mapping JSON RPC commands to `FsSandbox`, `ProcessManager`, `Dag` with permission enforcement
  - [x] Implement `WasmRuntime::on_event` to dispatch events to Wasm Guest
  - [x] Create a test Wasm fixture (e.g. `tests/fixtures/test_extension.wat` or raw wat compilation)
  - [x] Add tests in `src/wasm/tests.rs` verifying event dispatch, RPC command invocation, memory management, and permission checking
  - [x] Verify compliance with `#![deny(clippy::pedantic)]`, check_secrets.sh, and cargo test
- [x] AWU 6: PTY Support & Filesystem Watcher Sensor
  - [x] Add `term` and `pty` features to `nix` dependency, and add `notify` crate to `Cargo.toml`
  - [x] Refactor `ProcessManager` in `src/process.rs` to support PTY allocation for raw shell interactive control
  - [x] Implement filesystem watcher module (`src/fs/watcher.rs`) utilizing `notify` to report workspace changes
  - [x] Integrate watcher and PTY into the runner core/test scenarios
  - [x] Add unit tests in `src/process/tests.rs` and `src/fs/tests.rs` for verification
  - [x] Adhere to `#![deny(clippy::pedantic)]` and ensure zero warnings or leaks
- [x] AWU 7: HTTP Streaming Client & Dynamic Timeout Policies
  - [x] Add `reqwest`, `tokio` and `futures-util` dependencies to `Cargo.toml`
  - [x] Update `WasmState` and `WasmRuntime` to hold an event channel sender
  - [x] Implement `HttpStreamClient` in `src/http.rs` to handle streaming and dynamic timeout policies
  - [x] Implement background process monitoring emitting standard I/O events and enforcing timeouts
  - [x] Implement RPC handlers for `OpenHttpStream` and `SetStreamTimeoutPolicy` in `src/wasm/rpc.rs`
  - [x] Write tests verifying streaming response chunks, timeout detection, and policy dynamic updates
  - [x] Verify compliance with clippy, check_secrets.sh, and cargo test
- [x] AWU 8: Security Audit & E2E Integration Tests
  - [x] Audit `src/fs.rs` for path traversal prevention
  - [x] Create E2E integration test module under `tests/`
  - [x] Implement full flow in E2E test (Wasm -> PTY -> File Edit -> Snapshot -> Rollback)
  - [x] Verify zero clippy warnings and ensure all tests pass

- [x] AWU 9: GitHub Actions CI Setup
  - [x] Create `.github/workflows/ci.yml` supporting Ubuntu, macOS, and Windows
  - [x] Configure Rust toolchain for edition 2024 compatibility (stable)
  - [x] Add jobs for `cargo check`, `cargo clippy` (denying warnings), and `cargo test`
  - [x] Integrate secret and path leak scanner validation check in CI
  - [x] Verify compliance with clippy, check_secrets.sh, and cargo test locally

## Version 0.2 Implementation Phase (Single-Process Agent Shell / REPL)
- [x] AWU 10: In-Process REPL Loop & Stdin Monitoring
  - [x] Implement stdin reader loop in `src/main.rs` showing the `rad > ` prompt by default when no args are provided
- [x] AWU 11: Real-time Event-to-Stdout Streaming Router
  - [x] Route LLM tokens (`TokenReceived`) and spawned PTY process stdout/stderr directly to the terminal stdout/stderr in real-time
- [x] AWU 12: In-Process Human-in-the-Loop Approval Prompt
  - [x] Intercept privileged RPC commands (spawn process, write file) and prompt user `Approve? (y/n)` on terminal synchronously
- [x] AWU 13: Session State Persistence & DAG Recovery
  - [x] Implement JSON serialization for DAG state, saving sessions to `.rad/sessions/` and reloading via `rad --session <id>`
- [x] AWU 14: Autonomous Execution Loop Integration
  - [x] Link LLM orchestrator inside the single-process REPL, allowing autonomous execution to progress until goal completion or approval request
- [x] AWU 15: Single-Process E2E Integration Tests & Zero-Warning Audit
  - [x] Implement E2E integration test for the full REPL pipeline (task input, stream output, approval, and exit)
  - [x] Conduct full path security audit, zero Clippy warnings, and ensure all tests pass





