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
- [x] **Version 0.9.0: Generic MCP Server Integration**
- [x] **Version 0.9.1: Project Rule Loading & Identity Alignment**
- [x] **Version 0.9.2: Local LLM Token Optimization & Status Metrics**
- [x] **Version 0.9.3: LLM Reasoning & Thought Formatting UX Optimization**
- [x] **Version 0.9.4: REPL Shell General Path Completion Fix**
- [x] **Version 0.9.5: Local Installation of Updated Binary & Extensions**
- [x] **Version 0.9.6: Remove Thinking Indicator Output**
- [x] **Version 0.9.7: Hide Agent Process Outputs from REPL Terminal**
- [x] **Version 0.9.8: Fix AGENTS.md Autoloading via Host RPC**
- [x] **Version 0.9.9: Fix LLM Memory Loss via Sliding Window Fix**
- [x] **Version 0.9.10: Silence Host RPC error log**
- [x] **Version 0.9.11: Combine multiple AGENTS.md rules**
- [x] **Version 0.9.12: Fix 400 Bad Request**
- [x] **Version 0.9.13: Support Task Abort via Esc**
- [ ] **Version 0.9.14: Fix CRLF Line Endings in Raw Mode** (Current)





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

## Local Installation

* **AWU 59: Install RAD executable on Mac**
  - Install the compiled RAD binary to the local user cargo bin directory (~/.cargo/bin) so that 'rad' can be executed globally.

## REPL Improvements

* **AWU 60: Enable tab completion for shell commands and file paths**
  - Integrate rustyline's FilenameCompleter into CommandHelper.
  - Enable tab completion for file paths in both normal inputs and '!' shell command prefixes.
  - Implement a basic command completer for '!' prefixes.

## Detailed Plan: Version 0.9.0 (Generic MCP Server Integration)

* **AWU 61: Auto-spawn Configured MCP Servers in Orchestrator**
  - Update `ext/openai-orchestrator` to read configured MCP servers on startup.
  - Invoke `RasRpcCommand::SpawnMcpServer` to spin up the configured MCP servers through the WASM host-RPC layer.

* **AWU 62: Dynamically Register MCP Tools in LLM Context**
  - Communicate with spawned MCP servers to discover their exposed tools (schema).
  - Inject these dynamic tools into the OpenAI API completions requests along with default filesystem/bash tools.

* **AWU 63: Execute MCP Tools and Route Responses**
  - Implement dynamic routing in the tool execution loop: if a tool call belongs to an MCP server, send it via `RasRpcCommand::SendMcpRequest`.
  - Handle asynchronous or streamed responses back to the LLM agent.

* **AWU 64: E2E Verification with Tavily/Playwright MCP**
  - Register external MCP servers (like Tavily and/or Playwright) in `rad.json`.
  - Verify E2E flow: AI successfully calls the MCP search and browser tools to fetch real-time information.

## Detailed Plan: Version 0.9.1 (Project Rule Loading & Identity Alignment)

* **AWU 65: Implement Automatic AGENTS.md Loading & Identity Alignment**
  - Implement `load_local_agent_rules` helper in `ext/openai-orchestrator/src/llm.rs` to dynamically scan for `.agents/AGENTS.md` and `AGENTS.md` using the WASI filesystem API.
  - Revise `get_system_prompt` to declare the agent identity statement in accordance with the minimalist `pi-coding-agent` style, adapted for `rad`.
  - Append the loaded local rules to the main system prompt when constructing LLM requests.

## Detailed Plan: Version 0.9.2 (Local LLM Token Optimization & Status Metrics)

* **AWU 66: Parse Limit Configurations from rad.json**
  - Read `max_history_messages` and `max_tool_output_chars` parameters from the `openai-orchestrator` config block inside `rad.json`.
  - Store these parameters dynamically within the Wasm extension state at launch.

* **AWU 67: Implement Sliding History Window in Wasm Orchestrator**
  - Modify `load_messages_from_dag` in `ext/openai-orchestrator/src/llm.rs` to slice conversational history to the latest `N` messages.
  - Pin the initial user request (the very first DAG node) at the top of the history list to preserve goal state and context.

* **AWU 68: Implement Tool Output Trimming Utility**
  - Add `trim_large_output` string trimming function inside `ext/openai-orchestrator/src/tool_runner.rs`.
  - Check lengths of stdout/stderr and file reads and trim anything over 2,000 characters to a short summary (top & bottom format).

## Detailed Plan: Version 0.9.3 (LLM Reasoning & Thought Formatting UX Optimization)

* **AWU 69: Unify LLM Reasoning/Thought Formatting & UX**
  - Implement `\x1b[2K\r` ANSI escape sequence for erasing `Thinking...` in `src/terminal.rs`
  - Add `is_reasoning` and `reasoning_buffered` to `OrchestratorState` in `ext/openai-orchestrator/src/types.rs`
  - Update `sse.rs` to handle `reasoning_content` and `<thought>` tags, applying coloring and clear boundaries to terminal output
  - Update `orchestrator.rs` to initialize and manage reasoning states
  - Write tests and verify output format consistency

## Detailed Plan: Version 0.9.4 (REPL Shell General Path Completion Fix)

* **AWU 70: Enable File Path Completion for General REPL Prompt**
  - Update `CommandHelper::complete` in `src/command.rs` to fallback to file completion
  - Add tests in `tests/command_tests.rs` to verify general prompt completion
  - Run verification tests and lint checks

## Detailed Plan: Version 0.9.5 (Local Installation of Updated Binary & Extensions)

* **AWU 71: Install Updated RAD and Wasm Extensions**
  - Rebuild and compile the OpenAI Wasm Orchestrator extension
  - Execute `cargo install --path .` to install the updated rad binary locally
  - Verify the installed rad binary runs and version check

## Detailed Plan: Version 0.9.6 (Remove Thinking Indicator Output)

* **AWU 72: Remove Static Thinking Indicator Text from terminal.rs**
  - Delete `Thinking...` print from `TerminalState::Thinking` match branch in `src/terminal.rs`
  - Re-run all verification tests
  - Rebuild Wasm extension and reinstall rad binary locally

## Detailed Plan: Version 0.9.7 (Hide Agent Process Outputs)

* **AWU 73: Hide Agent Process Outputs from REPL Terminal**
  - Modify `route_event_to_terminal` in `src/ipc.rs` to stop routing `ProcessStdout` and `ProcessStderr` directly to the terminal.
  - This prevents background agent execution (such as `find`, `ls` or directory map scans) from polluting the REPL session UI.
  - Run cargo test and clippy audits.

## Detailed Plan: Version 0.9.8 (Fix AGENTS.md Autoloading via Host RPC)

* **AWU 74: Fix AGENTS.md Autoloading via Host RPC**
  - Modify `load_local_agent_rules` in `ext/openai-orchestrator/src/llm.rs` to fetch `.agents/AGENTS.md` and `AGENTS.md` via `call_host(RasRpcCommand::FileRead { path })` instead of `std::fs::read_to_string`.
  - Safely decode the read bytes as a UTF-8 string.
  - Run cargo test, clippy audits, and verify local autoloading.

## Detailed Plan: Version 0.9.9 (Fix LLM Memory Loss via Sliding Window Fix)

* **AWU 75: Increase Default LLM History Window Limit**
  - Update default `max_history_messages` from `6` to `30` in `ext/openai-orchestrator/src/llm.rs` to prevent immediate memory loss in multi-turn tool loops.
  - Run cargo test and clippy audits.

## Detailed Plan: Version 0.9.10 (Silence Host RPC error log)

* **AWU 76: Silence WASM Host RPC Error Terminal Print**
  - Remove `eprintln!` from the `Err` branch of `host_rpc` inside `src/wasm.rs`.
  - This prevents normal non-fatal errors (such as `FileRead` failing due to a missing `AGENTS.md`) from polluting the user terminal with red error banners.
  - Run cargo test and clippy audits.

## Detailed Plan: Version 0.9.11 (Combine multiple AGENTS.md rules)

* **AWU 77: Combine Multiple AGENTS.md Files Instead of First-Match**
  - Modify `load_local_agent_rules` in `ext/openai-orchestrator/src/llm.rs` to aggregate rules from all found paths (both `.agents/AGENTS.md` and `AGENTS.md`) instead of returning on the first match.
  - Run cargo test and clippy audits.

## Detailed Plan: Version 0.9.12 (Fix 400 Bad Request by filtering isolated tool messages)

* **AWU 78: Filter Out Isolated Tool Messages in llm.rs**
  - Implement a filtering pass in `load_messages_from_dag` inside `ext/openai-orchestrator/src/llm.rs` to ensure any `tool` message has a corresponding `assistant` message with matching `tool_calls` preceding it.
  - Remove any orphan `tool` messages to prevent LLM API 400 errors.
  - Run cargo test, clippy audits, and reinstall rad locally.

## Detailed Plan: Version 0.9.13 (Support Task Abort via Esc)

* **AWU 79: Support Task Abort via Esc Key**
  - Add `crossterm = "0.27"` to `Cargo.toml` dependencies.
  - Implement `pub fn abort(&self)` in `src/orchestrator.rs` to safely set `abort_flag` and join the running task thread.
  - Modify the waiting loop in `src/main.rs` to poll for `Esc` keypresses using `crossterm` in raw mode during execution.
  - Re-run all tests, check clippy, and reinstall the binary.

## Detailed Plan: Version 0.9.14 (Fix CRLF Line Endings in Raw Mode)

* **AWU 80: Fix Terminal Carriage Return (CRLF) in Raw Mode**
  - Implement a `to_crlf` string converter helper inside `src/terminal.rs`.
  - Apply `to_crlf` to `write_llm_token`, `write_log`, and `write_raw` in `TerminalController`.
  - This ensures that when the terminal enters raw mode for `Esc` key polling, newlines (`\n`) are correctly printed as carriage-return + line-feed (`\r\n`), preventing the "staircase" format output.
  - Run cargo test and clippy audits.







