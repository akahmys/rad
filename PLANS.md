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


## Bug Fixes

* **AWU 50: Fix Orchestrator Hang & Test Compatibility**
  - Fix `event_tx` being dropped prematurely in `src/orchestrator.rs`.
  - Update test files to include `hitl_enabled` field in `CoreConfig` initializers.


