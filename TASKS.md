# TASKS

Last Updated: 2026-07-05

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

## Version 0.7.0 Core Extensibility & Integration Layer
- [x] AWU 46: WIT-based Wasm Interface IDL Definition & WASI Integration
  - [x] Define WIT files for RPC commands and event schemas
  - [x] Adapt Core Wasm runtime loader to support `wit-bindgen` style structures
- [ ] AWU 47: Human-in-the-Loop (HITL) with Default YOLO Mode (Current)
  - [ ] Implement `request_human_approval` RPC in Core
  - [ ] Add `hitl_enabled` to `rad.json` and enforce default YOLO behavior in Core API Gateway
  - [ ] Implement Extension-side verification triggers
- [ ] AWU 48: Secure MCP (Model Context Protocol) Gateway Orchestration
  - [ ] Add `spawn_mcp_server` RPC in Core to manage external MCP server processes
  - [ ] Implement `allowed_mcp_servers` checklist verification in Core API Gateway
  - [ ] Integrate MCP JSON-RPC protocol implementation inside Extension
- [ ] AWU 49: Integration Testing & Verification Audit
  - [ ] Write tests validating WIT bindings, HITL prompting, and supervised MCP processes
  - [ ] Verify zero Clippy warnings, check secrets, and ensure all tests pass




