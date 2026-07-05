# PLANS

Last Updated: 2026-07-05

## Objective
Establish a comprehensive roadmap to build `rad` (Rust Agent Dispatcher) as a production-ready agent runner, incorporating process isolation, filesystem safety, WebAssembly plugins, PTY support, and streaming LLM API connection.

## Roadmap Overview
- [x] **Version 0.1: Core Infrastructure** (Process, FS, DAG, Wasm, PTY, HTTP, CI)
- [x] **Version 0.2: Single-Process Agent Shell** (REPL, Event Streaming, Human-in-the-loop, Session)
- [x] **Version 0.2.1: OpenAI-Compatible Wasm Extension & Enhanced UX**
- [>] **Version 0.2.2: DAG-Based Context Management (Current)**

## Detailed Plan: Version 0.2.2 (DAG-Based Context Management)

### Atomic Work Units (AWUs)

* **AWU 20: Add GetDag RPC to Core and Wasm API**
  - Add `GetDag` to `RasRpcCommand` in both `src/ipc.rs` and `ext/openai-orchestrator/src/lib.rs`
  - Implement `GetDag` handling in the Core Wasm host RPC handler returning serialization of `Dag`
* **AWU 21: Refactor Wasm Extension to Load and Persist History using DAG**
  - Update `ext/openai-orchestrator/src/lib.rs` to query `GetDag` on input events
  - Reconstruct `messages: Vec<Message>` by traversing history nodes in DAG topological order
  - Save new user inputs (`CreateNode`, `SetNodeText`) and assistant responses into the DAG
  - Remove `STATE` memory-based message array persistence
* **AWU 23: Translate README.md to English & Document pi-coding-agent Inspiration (Current)**
  - Translate the existing Japanese content in `README.md` to English to comply with the language policy.
  - Explicitly document that `rad` is a coding agent inspired by `pi-coding-agent`.
  - Ensure all formatting is clean and professional.
