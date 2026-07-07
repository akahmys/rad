# TASKS

Last Updated: 2026-07-06

## Completed Milestones
- [x] Initial Setup, Configuration & Architecture (v0.0)
- [x] Core Subsystems: Process, FS, DAG, Wasm, PTY, HTTP, CI (v0.1)
- [x] Single-Process Agent Shell: REPL, Event Streaming, Human-in-the-loop, Session (v0.2)
- [x] Wasm Extension: OpenAI Compatibility & Enhanced REPL UX (v0.2.1)
- [x] DAG-Based Context Management & Core Refactoring (v0.2.2)
- [x] Tool Execution Loop & Autonomy (v0.2.3)
- [x] Interactive UX & Human-in-the-Loop (v0.3.0)
- [x] Resiliency & Extension-based Security Hooks (v0.4.0)
- [x] API Freeze & Distribution (v0.5.0)
- [x] Multi-extension Support (v0.6.0)
- [x] Core Extensibility & Integration Layer (v0.7.0)

## Version 0.8.0 Large Codebase Optimization & Autonomy
- [x] AWU 52: Semantic Repository Map Integration
  - [x] Extract code structure definitions using tree-sitter
  - [x] Inject semantic references into history DAG context
- [x] AWU 53: Autopilot Git Integration & Failure Recovery
  - [x] Implement dynamic local branching and auto-commits
  - [x] Trigger rollbacks on verification failures

## Bug Fixes

* [x] **AWU 50: Fix Orchestrator Hang & Test Compatibility**
  - [x] Fix `event_tx` lifecycle in `src/orchestrator.rs`
  - [x] Update test files with `hitl_enabled` in `CoreConfig`
  - [x] Run `cargo test` and verify

* [x] **AWU 51: Investigate & Fix LLM Second Turn Hang (Deadlock)**
  - [x] Verify deadlock hypothesis and plan the fix in `ext/openai-orchestrator/src/orchestrator.rs`
  - [x] Implement fix (release `STATE` lock before `call_host` in `handle_done` and `execute_and_collect_tools`)
  - [x] Update `load_messages_from_dag` to correctly handle valid LLM roles and empty message content
  - [x] Run audit and commit

## Refactoring & Code Quality

* [x] **AWU 54: Refactor Orchestrator & WASM RPC to adhere to File Size Limit**
  - [x] Refactor `src/orchestrator.rs` to move helper logic/runner into `src/orchestrator/runner.rs` or similar, reducing file size below 300 lines.
  - [x] Refactor `src/wasm/rpc.rs` by extracting RPC command handlers to reduce file size below 300 lines.
  - [x] Run verification tests and lint checks (Clippy zero warnings, Cargo test, check secrets).

## Extension Developer Experience (DX) Kickoff (v0.0)

* [x] **AWU 55: Create Extension Developer Guide (EXTENSIONS.md)**
  - [x] Write detailed documentation for the `radcomp:extension` WIT interface and API endpoints.
  - [x] Document permission configurations (`rad.json`) and runtime security guidelines.
  - [x] Add instructions for compilation, environment setup, and debugging.
  - [x] Verify formatting and paths.

* [x] **AWU 56: Create Rust Extension Boilerplate Template**
  - [x] Implement a minimal `templates/rust` directory structure.
  - [x] Add `Cargo.toml`, `wit/rad.wit` link, and basic `src/lib.rs` implementing `on_event` and `verify_rpc`.
  - [x] Validate compilation of the template.

* [x] **AWU 57: Create Go Extension Boilerplate Template**
  - [x] Implement a minimal `templates/go` directory structure.
  - [x] Add `go.mod`, skeleton code with `wit-bindgen-go`, and compilation script.
  - [x] Validate compilation of the Go template using tinygo.

* [x] **AWU 58: Enhance Extension Runtime Error Logs**
  - [x] Catch WASM engine errors and log descriptive panics or stack traces.
  - [x] Add error contexts for RPC serialization/deserialization failures.

## Local Installation

* [x] **AWU 59: Install RAD executable on Mac**
  - [x] Execute `cargo install --path .` to compile and install RAD to ~/.cargo/bin.
  - [x] Verify the installation by calling `rad --version`.

## REPL Improvements

* [x] **AWU 60: Enable tab completion for shell commands and file paths**
  - [x] Update `CommandHelper` in `src/command.rs` to support file path completion using `rustyline::completion::FilenameCompleter`.
  - [x] Implement command completion for the `!` shell command prefix.
  - [x] Run verification tests and compile checks.






