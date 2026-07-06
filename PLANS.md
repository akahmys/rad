# PLANS

Last Updated: 2026-07-06

## Objective
Establish a comprehensive roadmap to build `rad` (Rust Agent Dispatcher) as a production-ready agent runner, incorporating WebAssembly plugins, PTY support, streaming LLM API connection, and extensible hooks (allowing users to delegate security and sandboxing to extensions).

## Roadmap Overview
- [x] **Version 0.1: Core Infrastructure** (Process, FS, DAG, Wasm, PTY, HTTP, CI)
- [x] **Version 0.2: Single-Process Agent Shell** (REPL, Event Streaming, Human-in-the-loop, Session)
- [x] **Version 0.2.1: OpenAI-Compatible Wasm Extension & Enhanced UX**
- [x] **Version 0.2.2: DAG-Based Context Management & Core Refactoring**
- [x] **Version 0.2.3: Tool Execution Loop & Autonomy**
- [x] **Version 0.2.x Stabilization: Comprehensive Audit & Refactoring**
- [x] **Version 0.3.0: Interactive UX & Human-in-the-Loop (YOLO & Slash Commands)**
- [x] **Version 0.3.x Stabilization: Comprehensive Audit & Refactoring**
- [x] **Version 0.4.0: Resiliency & Extension-based Security Hooks (Recovery & Custom Hooks)**
- [x] **Version 0.4.x Stabilization: Comprehensive Audit & Refactoring**
- [x] **Version 0.5.0: API Freeze & Distribution (API Freeze, Packaging)**
- [x] **Version 0.6.0: Multi-extension Support**
- [x] **Version 0.7.0: Core Extensibility & Integration Layer (WASM Bindings, HITL-YOLO, MCP Gateway)**
- [x] **Version 0.8.0: Large Codebase Optimization & Autonomy**

## Detailed Plan: Version 0.7.0 (Core Extensibility & Integration Layer)

* **AWU 46: WIT-based Wasm Interface IDL Definition & WASI Integration**
  - Define Wasm Interface Types (WIT) for all RPC commands and events between Core and Extension.
  - Transition the Wasm host in Core to use `wit-bindgen` compatible WASI bindings, enabling multi-language plugin development (Go, TypeScript, etc.).

* **AWU 47: Human-in-the-Loop (HITL) with Default YOLO Mode**
  - Add `hitl_enabled` boolean field to `CoreConfig` in `src/config.rs` (defaults to `false` which is YOLO mode).
  - Implement the handler for `RasRpcCommand::AskHumanApproval` in Core (`src/ipc.rs` and `src/wasm.rs`).
    - If `hitl_enabled` is `false`, immediately return `"true"` (auto-approved).
    - If `hitl_enabled` is `true`, prompt the user in the terminal (stdout/stdin) for `y/n` confirmation, and return `"true"` or `"false"` based on user response.
  - Implement extension-side triggers (e.g. within `openai-orchestrator`'s host call handler) to use this interface when necessary, or ensure existing HITL workflows delegate cleanly to it.
  - Add integration tests verifying both HITL-enabled (prompt waiting) and YOLO mode (auto-approving) runs.

* **AWU 48: Secure MCP (Model Context Protocol) Gateway Orchestration**
  - Implement `spawn_mcp_server` RPC command in Core to spawn and supervise external MCP server processes.
  - Add `allowed_mcp_servers` verification under `rad.json` inside the API Gateway to restrict which external MCP servers can be loaded.
  - Integrate a JSON-RPC based MCP client component in the Extension workspace.

* **AWU 49: Integration Testing & Verification Audit**
  - Implement integration tests validating HITL prompting behavior, multi-language bindings compile checks, and supervised MCP process execution.
  - Complete full codebase audit (Clippy zero warnings, Cargo test, check secrets).

## Detailed Plan: Version 0.8.0 (Large Codebase Optimization & Autonomy)

* **AWU 52: Semantic Repository Map Integration**
  - Extract code structure definitions (classes, functions, types) using tree-sitter or similar tools.
  - Inject semantic references directly into the DAG context to guide LLM search without context token bloat.

* **AWU 53: Autopilot Git Integration & Failure Recovery**
  - Implement dynamic local branching and automatic checkpoint commits during multi-turn edits.
  - Trigger automatic rollbacks (using Git branch reset and DAG state rehydration) if local verification checks fail.

## Bug Fixes

* **AWU 50: Fix Orchestrator Hang & Test Compatibility**
  - Fix `event_tx` being dropped prematurely in `src/orchestrator.rs`.
  - Update test files to include `hitl_enabled` field in `CoreConfig` initializers.

* **AWU 51: Investigate & Fix LLM Second Turn Hang (Deadlock)**
  - Release `STATE` mutex lock in `ext/openai-orchestrator/src/orchestrator.rs` before invoking any synchronous `call_host` RPCs (e.g., in `handle_done` and `execute_and_collect_tools`).
  - Fix DAG traversal in `load_messages_from_dag` to skip non-LLM nodes (like `"merge"`) and filter out empty content or invalid messages to prevent LLM API errors.

## Refactoring & Code Quality

* **AWU 54: Refactor Orchestrator & WASM RPC to adhere to File Size Limit**
  - Refactor `src/orchestrator.rs` (503 lines) to be under 300 lines by splitting concerns (e.g., separating runner/execution loop into `src/orchestrator/runner.rs` or similar helper modules).
  - Refactor `src/wasm/rpc.rs` (307 lines) to be under 300 lines by extracting handlers for `RasRpcCommand` variants into dedicated helper functions or a sub-module.
  - Ensure all refactored code passes `cargo check`, `cargo clippy --all-targets` with zero warnings, and `cargo test`.

## Extension Developer Experience (DX) Kickoff (v0.0)

* **AWU 55: Create Extension Developer Guide (EXTENSIONS.md)**
  - Document radcomp:extension WIT interface details, API behaviors, configuration options, and permissions setup.
  - Provide compilation, deployment, and debugging guides for third-party extensions.

* **AWU 56: Create Rust Extension Boilerplate Template**
  - Implement a minimal templates/rust template using cargo and wit-bindgen to bootstrap third-party Rust extensions.

* **AWU 57: Create Go Extension Boilerplate Template**
  - Implement a minimal templates/go template using tinygo and wit-bindgen-go to bootstrap third-party Go extensions.

* **AWU 58: Enhance Extension Runtime Error Logs**
  - Improve error reporting in WASM runtime by printing detailed panics, stack traces, and RPC serialization error contexts to guide developers.



