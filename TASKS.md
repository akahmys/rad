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

## Version 0.7.0 Core Extensibility & Integration Layer (Current)
- [x] AWU 46: WIT-based Wasm Interface IDL Definition & WASI Integration
  - [x] Define WIT files for RPC commands and event schemas
  - [x] Adapt Core Wasm runtime loader to support `wit-bindgen` style structures
- [x] AWU 47: Human-in-the-Loop (HITL) with Default YOLO Mode
  - [x] Add `hitl_enabled` boolean field to `CoreConfig` in `src/config.rs`
  - [x] Implement `RasRpcCommand::AskHumanApproval` in Core
  - [x] Implement integration tests verifying HITL prompting and YOLO mode runs
- [x] AWU 48: Secure MCP (Model Context Protocol) Gateway Orchestration
  - [x] Add `spawn_mcp_server` RPC in Core to manage external MCP server processes
  - [x] Implement `allowed_mcp_servers` checklist verification in Core API Gateway
  - [x] Integrate MCP JSON-RPC protocol implementation inside Extension
- [/] AWU 49: Integration Testing & Verification Audit (Current)
  - [ ] Write tests validating WIT bindings, HITL prompting, and supervised MCP processes
  - [ ] Verify zero Clippy warnings, check secrets, and ensure all tests pass

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


