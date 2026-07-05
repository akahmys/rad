# TASKS

Last Updated: 2026-07-05

## Completed Milestones
- [x] Initial Setup, Configuration & Architecture (v0.0)
- [x] Core Subsystems: Process, FS, DAG, Wasm, PTY, HTTP, CI (v0.1)
- [x] Single-Process Agent Shell: REPL, Event Streaming, Human-in-the-loop, Session (v0.2)
- [x] Wasm Extension: OpenAI Compatibility & Enhanced REPL UX (v0.2.1)

## Version 0.2.2 DAG-Based Context Management & Core Refactoring (Current)
- [x] AWU 19: Refactor Core Subsystems with Trait Abstractions
  - [x] Define Traits for `FsSubsystem`, `ProcessSubsystem`, `DagSubsystem`, and `NetworkSubsystem`
  - [x] Implement API Gateway for Wasm RPC requests with centralized `rad.json` permission checks
  - [x] Split Core code into modular source files complying with the 300-line limit
- [x] AWU 20: Add GetDag RPC to Core and Wasm API
  - [x] Add `GetDag` to `RasRpcCommand` in both `src/ipc.rs` and `ext/openai-orchestrator/src/lib.rs`
  - [x] Implement `GetDag` handling in the Core Wasm host RPC handler returning serialization of `Dag`
- [/] AWU 21: Refactor Wasm Extension to Load and Persist History using DAG (Current)
  - [ ] Update `ext/openai-orchestrator/src/lib.rs` to query `GetDag` on input events
  - [ ] Reconstruct `messages: Vec<Message>` by traversing history nodes in DAG topological order
  - [ ] Save new user inputs (`CreateNode`, `SetNodeText`) and assistant responses into the DAG
  - [ ] Remove `STATE` memory-based message array persistence
- [ ] AWU 22: Verify Context Restoration & Zero-Warning Audit
  - [ ] Write integration test validating context restoration across session restarts/Extension reloads
  - [ ] Achieve zero Clippy warnings, check secrets, and ensure all tests pass
- [x] AWU 23: Translate README.md to English & Document pi-coding-agent Inspiration
  - [x] Translate Japanese contents of `README.md` to English
  - [x] Explicitly describe that `rad` is inspired by `pi-coding-agent`

## Version 0.2.3 Tool Execution Loop & Autonomy
- [ ] AWU 24: Add Tool Call Schemas to Wasm Extension Request
  - [ ] Define `tools` parameter structure in Wasm Extension OpenAI request
  - [ ] Map `file_read`, `file_write`, `file_edit_patch`, `spawn_bash_process` to JSON Schemas
- [ ] AWU 25: Implement Tool Call Stream Parsing and Core RPC Dispatching
  - [ ] Buffer and parse streaming `tool_calls` from chunk chunks
  - [ ] Map and invoke Core RPC methods on complete tool call extraction
- [ ] AWU 26: Implement Autonomy Loop with System Prompt and Multi-turn Execution
  - [ ] Send execution results back to the chat context as tool messages
  - [ ] Add system prompt explaining rad architecture, constraints, and tool usage
- [ ] AWU 27: Verify Tool Execution Integration Test & Zero-Warning Audit
  - [ ] Write integration test validating complete LLM-driven tool loop execution
  - [ ] Pass check secrets, clippy, and unit/integration tests with zero warnings

## Version 0.2.x Stabilization (Comprehensive Audit & Refactoring)
- [ ] AWU 27.5: Core & Autonomy Stabilization
  - [ ] Write end-to-end integration tests for context recovery + tool execution
  - [ ] Refactor technical debt in Wasm-Core serialized message definitions
  - [ ] Ensure zero clippy warnings and no secrets in git stage

## Version 0.3.0 Interactive UX & Human-in-the-Loop
- [ ] AWU 28: Support Shell Escape (`!`) in REPL
  - [ ] Parse and execute commands starting with `!` in Core's REPL
- [ ] AWU 29: Dynamic Slash Commands
  - [ ] Support `/rollback`, `/status`, and custom slash commands
- [ ] AWU 30: Autonomous Execution Loop (YOLO by Default)
  - [ ] Implement full auto-execution loop in Core and Extension
  - [ ] Verify that Wasm Extensions can optionally intercept and block for user confirmation

## Version 0.3.x Stabilization (Comprehensive Audit & Refactoring)
- [ ] AWU 30.5: UX & REPL Control Stabilization
  - [ ] Test async shell escapes combined with main loop edge cases
  - [ ] Refactor REPL command management to cleanly decouple core/wasm commands

## Version 0.4.0 Hardening, Security & Resiliency
- [ ] AWU 31: Path Canonicalization & Jail Checks
  - [ ] Enforce strict jail check in `FsSubsystem`
- [ ] AWU 32: Extension Self-Healing
  - [ ] Reload Wasm Extension and restore DAG context on Wasm panic/crash

## Version 0.4.x Stabilization (Comprehensive Audit & Refactoring)
- [ ] AWU 32.5: Security & Chaos E2E Testing
  - [ ] Conduct security audit on API Gateway path validation
  - [ ] Execute chaos tests simulating sudden Wasm crash/timeouts during runtime operations

## Version 1.0.0 Production Release & Stabilization
- [ ] AWU 33: API Freeze & Serialization Optimization
  - [ ] Finalize RPC models and improve communication efficiency
- [ ] AWU 34: Packaging & Distribution
  - [ ] Automate CI release builds for macOS, Linux, and Windows



