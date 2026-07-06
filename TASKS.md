# TASKS

Last Updated: 2026-07-05

## Completed Milestones
- [x] Initial Setup, Configuration & Architecture (v0.0)
- [x] Core Subsystems: Process, FS, DAG, Wasm, PTY, HTTP, CI (v0.1)
- [x] Single-Process Agent Shell: REPL, Event Streaming, Human-in-the-loop, Session (v0.2)
- [x] Wasm Extension: OpenAI Compatibility & Enhanced REPL UX (v0.2.1)

## Version 0.2.2 DAG-Based Context Management & Core Refactoring
- [x] AWU 19: Refactor Core Subsystems with Trait Abstractions
  - [x] Define Traits for `FsSubsystem`, `ProcessSubsystem`, `DagSubsystem`, and `NetworkSubsystem`
  - [x] Implement API Gateway for Wasm RPC requests with centralized `rad.json` permission checks
  - [x] Split Core code into modular source files complying with the 300-line limit
- [x] AWU 20: Add GetDag RPC to Core and Wasm API
  - [x] Add `GetDag` to `RasRpcCommand` in both `src/ipc.rs` and `ext/openai-orchestrator/src/lib.rs`
  - [x] Implement `GetDag` handling in the Core Wasm host RPC handler returning serialization of `Dag`
- [x] AWU 21: Refactor Wasm Extension to Load and Persist History using DAG
  - [x] Update `ext/openai-orchestrator/src/lib.rs` to query `GetDag` on input events
  - [x] Reconstruct `messages: Vec<Message>` by traversing history nodes in DAG topological order
  - [x] Save new user inputs (`CreateNode`, `SetNodeText`) and assistant responses into the DAG
  - [x] Remove `STATE` memory-based message array persistence
- [x] AWU 22: Verify Context Restoration & Zero-Warning Audit
  - [x] Write integration test validating context restoration across session restarts/Extension reloads
  - [x] Achieve zero Clippy warnings, check secrets, and ensure all tests pass
- [x] AWU 23: Translate README.md to English & Document pi-coding-agent Inspiration
  - [x] Translate Japanese contents of `README.md` to English
  - [x] Explicitly describe that `rad` is inspired by `pi-coding-agent`

## Version 0.2.3 Tool Execution Loop & Autonomy (Current)
- [x] AWU 24: Add Tool Call Schemas to Wasm Extension Request
  - [x] Define `tools` parameter structure in Wasm Extension OpenAI request
  - [x] Map `file_read`, `file_write`, `file_edit_patch`, `spawn_bash_process` to JSON Schemas
- [x] AWU 25: Implement Tool Call Stream Parsing and Core RPC Dispatching
  - [x] Buffer and parse streaming `tool_calls` from chunk chunks
  - [x] Map and invoke Core RPC methods on complete tool call extraction
- [x] AWU 26: Implement Autonomy Loop with System Prompt and Multi-turn Execution
  - [x] Send execution results back to the chat context as tool messages
  - [x] Add system prompt explaining rad architecture, constraints, and tool usage
- [x] AWU 26.5: Document Extension-Based Unified Tooling & Safety Architecture
  - [x] Update README.md to describe the unified tool and extension-based policy architecture
  - [x] Update ARCHITECTURE.md to clarify that Core is decoupled from MCP/Skills and offloads policies to Extensions
  - [x] Update PLANS.md to align version plans with the extension-offloaded design
- [x] AWU 27: Verify Tool Execution Integration Test & Zero-Warning Audit
  - [x] Write integration test validating complete LLM-driven tool loop execution
  - [x] Pass check secrets, clippy, and unit/integration tests with zero warnings

## Version 0.2.x Stabilization (Comprehensive Audit & Refactoring)
- [x] AWU 27.5: Unify Data Models & Codebase-wide Refactoring
  - [x] Create shared workspace crate `models` and migrate IPC, DAG, Timeout types
  - [x] Refactor Core Subsystems (Process, FS, DAG, Wasm) to standardise error handling and clean up debt
  - [x] Write end-to-end integration tests for context recovery + tool execution
  - [x] Ensure zero clippy warnings and no secrets in git stage

## Version 0.3.0 Interactive UX & Human-in-the-Loop
- [x] AWU 28: Support Shell Escape (`!`) in REPL
  - [x] Parse and execute commands starting with `!` in Core's REPL
- [x] AWU 29: Dynamic Slash Commands
  - [x] Support `/rollback`, `/status`, and custom slash commands
- [x] AWU 30: Autonomous Execution Loop (YOLO by Default)
  - [x] Implement full auto-execution loop in Core and Extension
  - [x] Verify that Wasm Extensions can optionally intercept and block for user confirmation

## Version 0.3.x Stabilization (Comprehensive Audit & Refactoring)
- [x] AWU 30.5: UX & REPL Control Stabilization
  - [x] Test async shell escapes combined with main loop edge cases
  - [x] Refactor REPL command management to cleanly decouple core/wasm commands

## Version 0.4.0 Resiliency & Extension-based Security Hooks
- [x] AWU 31: Extension-based Security Verification Hooks
  - [x] Implement interception hooks in API Gateway for dynamic operation inspection by Wasm Extensions
- [x] AWU 32: Extension Self-Healing
  - [x] Reload Wasm Extension and restore DAG context on Wasm panic/crash

## Version 0.4.x Stabilization (Comprehensive Audit & Refactoring)
- [x] AWU 32.5: Security & Chaos E2E Testing
  - [x] Verify Wasm Extension interception logic under malicious/heavy load operations
  - [x] Execute chaos tests simulating sudden Wasm crash/timeouts during runtime operations

## Version 0.5.0 API Freeze & Distribution
- [x] AWU 33: API Freeze & Serialization Optimization
  - [x] Finalize RPC models and improve communication efficiency
- [/] AWU 34: Packaging & Distribution (Current)
  - [ ] Automate CI release builds for macOS, Linux, and Windows



