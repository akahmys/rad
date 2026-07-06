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
- [x] **Version 0.3.x Stabilization: Comprehensive Audit & Refactoring**
- [x] **Version 0.4.0: Resiliency & Extension-based Security Hooks (Recovery & Custom Hooks)**
- [x] **Version 0.4.x Stabilization: Comprehensive Audit & Refactoring**
- [x] **Version 0.5.0: API Freeze & Distribution (API Freeze, Packaging)**
- [x] **Version 0.6.0: Multi-extension Support**
- [>] **Version 0.7.0: Core Extensibility & Integration Layer (WASM Bindings, HITL-YOLO, MCP Gateway) (Current)**


## Detailed Plan: Version 0.7.0 (Core Extensibility & Integration Layer) (Current)

* **AWU 46: WIT-based Wasm Interface IDL Definition & WASI Integration**
  - Define Wasm Interface Types (WIT) for all RPC commands and events between Core and Extension.
  - Transition the Wasm host in Core to use `wit-bindgen` compatible WASI bindings, enabling multi-language plugin development (Go, TypeScript, etc.).

* **AWU 47: Human-in-the-Loop (HITL) with Default YOLO Mode**
  - Add `request_human_approval` RPC call to Core's Wasm host interface.
  - Update `rad.json` configuration structure to include a `hitl_enabled` boolean (defaults to `false` for YOLO mode).
  - Implement a terminal approval prompt in Core when `hitl_enabled` is active, otherwise instantly permit operations.

* **AWU 48: Secure MCP (Model Context Protocol) Gateway Orchestration**
  - Implement `spawn_mcp_server` RPC command in Core to spawn and supervise external MCP server processes.
  - Add `allowed_mcp_servers` verification under `rad.json` inside the API Gateway to restrict which external MCP servers can be loaded.
  - Integrate a JSON-RPC based MCP client component in the Extension workspace.

* **AWU 49: Integration Testing & Verification Audit**
  - Implement integration tests validating HITL prompting behavior, multi-language bindings compile checks, and supervised MCP process execution.
  - Complete full codebase audit (Clippy zero warnings, Cargo test, check secrets).


