# PLANS

Last Updated: 2026-07-05

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
- [>] **Version 0.3.x Stabilization: Comprehensive Audit & Refactoring (Current)**
- [ ] **Version 0.4.0: Resiliency & Extension-based Security Hooks (Recovery & Custom Hooks)**
- [ ] **Version 0.4.x Stabilization: Comprehensive Audit & Refactoring**
- [ ] **Version 1.0.0: Production Release & Stabilization (API Freeze, Packaging)**


## Detailed Plan: Version 0.2.2 (DAG-Based Context Management & Core Refactoring)

### Atomic Work Units (AWUs)

* **AWU 19: Refactor Core Subsystems with Trait Abstractions**
  - Define traits for core subsystems: `FsSubsystem`, `ProcessSubsystem`, `DagSubsystem`, and `NetworkSubsystem` in Core
  - Refactor Wasm host RPC handler into an "API Gateway" that performs centralized permission checks (via `rad.json`) before routing to traits
  - Restructure files in `src/` to ensure strict compliance with the 300-line limit per file
* **AWU 20: Add GetDag RPC to Core and Wasm API**
  - Add `GetDag` to `RasRpcCommand` in both `src/ipc.rs` and `ext/openai-orchestrator/src/lib.rs`
  - Implement `GetDag` handling in the Core Wasm host RPC handler returning serialization of `Dag`
* **AWU 21: Refactor Wasm Extension to Load and Persist History using DAG**
  - Update `ext/openai-orchestrator/src/lib.rs` to query `GetDag` on input events
  - Reconstruct `messages: Vec<Message>` by traversing history nodes in DAG topological order
  - Save new user inputs (`CreateNode`, `SetNodeText`) and assistant responses into the DAG
  - Remove `STATE` memory-based message array persistence
* **AWU 22: Verify Context Restoration & Zero-Warning Audit**
  - Write integration test validating context restoration across session restarts/Extension reloads
  - Achieve zero Clippy warnings, check secrets, and ensure all tests pass

* **AWU 23: Translate README.md to English & Document pi-coding-agent Inspiration** (Completed)
  - Translate the existing Japanese content in `README.md` to English to comply with the language policy.
  - Explicitly document that `rad` is a coding agent inspired by `pi-coding-agent`.
  - Ensure all formatting is clean and professional.

## Detailed Plan: Version 0.2.3 (Tool Execution Loop & Autonomy)

### Atomic Work Units (AWUs)

* **AWU 24: Add Tool Call Schemas to Wasm Extension Request**
  - Define `tools` request structure (OpenAI compatible schema) in `ext/openai-orchestrator`
  - Write schemas for physical primitives: `file_read`, `file_write`, `file_edit_patch`, and `spawn_bash_process`
* **AWU 25: Implement Tool Call Stream Parsing and Core RPC Dispatching**
  - Parse `tool_calls` from the SSE chunk stream in Wasm Extension
  - Accumulate arguments and invoke corresponding Core RPCs on tool call completion
* **AWU 26: Implement Autonomy Loop with System Prompt and Multi-turn Execution**
  - Feed tool execution results back to LLM as `tool` role messages
  - Implement automatic multi-turn loop and inject a system prompt explaining rad context and tools
* **AWU 26.5: Document Extension-Based Unified Tooling & Safety Architecture (Current)**
  - Reflect the unified tool framework concept in README.md, ARCHITECTURE.md, and PLANS.md.
  - Document that basic commands, Skills, Workflows, and MCP are treated as unified Tool Calls by the LLM, offloading safety and workflow policies entirely to optional Wasm Extensions.
* **AWU 27: Verify Tool Execution Integration Test & Zero-Warning Audit**
  - Add integration tests verifying LLM-driven tool execution (e.g., executing a command and reading a file)
  - Verify zero Clippy warnings across all workspace targets

## Detailed Plan: Version 0.2.x Stabilization (Comprehensive Audit & Refactoring)

* **AWU 27.5: Core & Autonomy Stabilization**
  - Perform comprehensive end-to-end (E2E) testing of the DAG-based context recovery combined with the tool execution loop.
  - Refactor technical debt in IPC/RPC serialization schemas between Core and Wasm.
  - Run clippy, tests, and secret checks with zero warnings.

## Detailed Plan: Version 0.3.0 (Interactive UX & Human-in-the-Loop) (Current)

* **AWU 28: Support Shell Escape (`!`) in REPL**
  - Implement parsing for lines starting with `!` in Core's REPL to execute commands directly on the host shell without triggering the LLM.
* **AWU 29: Dynamic Slash Commands**
  - Add support in Core and Extension for metadata commands (e.g., `/rollback <node_id>`, `/status`).
* **AWU 30: Autonomous Execution Loop (YOLO by Default)**
  - Implement full auto-execution loop in Core and Wasm Extension (YOLO by default).
  - Ensure the event-driven design permits Extensions to optionally intercept tool calls and block for human input (`HumanInputReceived`) if custom HITL logic is desired.

## Detailed Plan: Version 0.3.x Stabilization (Comprehensive Audit & Refactoring) (Current)

* **AWU 30.5: UX & REPL Control Stabilization (Current)**
  - Audit and test edge cases combining async shell escapes (`!`), slash commands, and the main Wasm loop.
  - Refactor REPL command manager logic to decouple built-in commands from Wasm-level interceptors.

## Detailed Plan: Version 0.4.0 (Resiliency & Extension-based Security Hooks)

* **AWU 31: Extension-based Security Verification Hooks**
  - Implement custom request/response interception hooks in the API Gateway to allow WebAssembly Extensions to dynamically inspect, approve, or reject filesystem and process operations (offloading sandbox/security to extensions).
* **AWU 32: Extension Self-Healing**
  - Implement automatic Wasm instance recovery in Core. If the Extension crashes, Core will reload it and re-hydrate its state from the active DAG node.

## Detailed Plan: Version 0.4.x Stabilization (Comprehensive Audit & Refactoring)

* **AWU 32.5: Security & Chaos E2E Testing**
  - Run integration tests verifying that WebAssembly Extensions can successfully intercept and restrict filesystem/process requests.
  - Conduct chaos testing (abruptly crashing the Wasm runtime during file writes/process runs) to verify self-healing resilience.

## Detailed Plan: Version 1.0.0 (Production Release & Stabilization)

* **AWU 33: API Freeze & Serialization Optimization**
  - Freeze the RPC command/event schemas. Optimize communication overhead.
* **AWU 34: Packaging & Distribution**
  - Setup CI/CD release pipeline to build static binaries for target platforms (macOS, Linux, Windows).



