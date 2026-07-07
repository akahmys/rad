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
- [x] Core Extensibility & Integration Layer (v0.7.0)

## Version 0.8.0 Large Codebase Optimization & Autonomy
- [x] AWU 52: Semantic Repository Map Integration
  - [x] Extract code structure definitions using tree-sitter
  - [x] Inject semantic references into history DAG context
- [x] AWU 53: Autopilot Git Integration & Failure Recovery
  - [x] Implement dynamic local branching and auto-commits
  - [x] Trigger rollbacks on verification failures

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

## Refactoring & Code Quality

* [x] **AWU 54: Refactor Orchestrator & WASM RPC to adhere to File Size Limit**
  - [x] Refactor `src/orchestrator.rs` to move helper logic/runner into `src/orchestrator/runner.rs` or similar, reducing file size below 300 lines.
  - [x] Refactor `src/wasm/rpc.rs` by extracting RPC command handlers to reduce file size below 300 lines.
  - [x] Run verification tests and lint checks (Clippy zero warnings, Cargo test, check secrets).

## Extension Developer Experience (DX) Kickoff (v0.0)

* [x] **AWU 55: Create Extension Developer Guide (EXTENSIONS.md)**
  - [x] Write detailed documentation for the `radcomp:extension` WIT interface and API endpoints.
  - [x] Document permission configurations (`rad.json`) and runtime security guidelines.
  - [x] Add instructions for compilation, environment setup, and debugging.
  - [x] Verify formatting and paths.

* [x] **AWU 56: Create Rust Extension Boilerplate Template**
  - [x] Implement a minimal `templates/rust` directory structure.
  - [x] Add `Cargo.toml`, `wit/rad.wit` link, and basic `src/lib.rs` implementing `on_event` and `verify_rpc`.
  - [x] Validate compilation of the template.

* [x] **AWU 57: Create Go Extension Boilerplate Template**
  - [x] Implement a minimal `templates/go` directory structure.
  - [x] Add `go.mod`, skeleton code with `wit-bindgen-go`, and compilation script.
  - [x] Validate compilation of the Go template using tinygo.

* [x] **AWU 58: Enhance Extension Runtime Error Logs**
  - [x] Catch WASM engine errors and log descriptive panics or stack traces.
  - [x] Add error contexts for RPC serialization/deserialization failures.

## Local Installation

* [x] **AWU 59: Install RAD executable on Mac**
  - [x] Execute `cargo install --path .` to compile and install RAD to ~/.cargo/bin.
  - [x] Verify the installation by calling `rad --version`.

## REPL Improvements

* [x] **AWU 60: Enable tab completion for shell commands and file paths**
  - [x] Update `CommandHelper` in `src/command.rs` to support file path completion using `rustyline::completion::FilenameCompleter`.
  - [x] Implement command completion for the `!` shell command prefix.
  - [x] Run verification tests and compile checks.

## Version 0.9.0 Generic MCP Server Integration
* [x] **AWU 61: Auto-spawn Configured MCP Servers in Orchestrator**
  - [x] Update Wasm host RPC bridge or configuration resolver to pass configured MCP servers to `ext/openai-orchestrator`
  - [x] Implement startup trigger in `ext/openai-orchestrator` to call `RasRpcCommand::SpawnMcpServer` for each allowed MCP server
* [x] **AWU 62: Dynamically Register MCP Tools in LLM Context**
  - [x] Implement initial handshake / schema retrieval from spawned MCP servers within the orchestrator Wasm
  - [x] Adapt tool formatting inside `ext/openai-orchestrator/src/tool.rs` to dynamically generate tool definitions from MCP tool schemas
* [x] **AWU 63: Execute MCP Tools and Route Responses**
  - [x] Implement execution routing: intercept tool calls belonging to registered MCPs and direct them to `RasRpcCommand::SendMcpRequest`
  - [x] Map MCP responses back to tool message content for LLM consumption
* [x] **AWU 64: E2E Verification with Tavily/Playwright MCP**
  - [x] Add MCP configuration entry to `rad.json`
  - [x] Perform integration run and verify that the RAD agent can query Tavily search and operate via Playwright MCP

## Version 0.9.1 Project Rule Loading & Identity Alignment
* [x] **AWU 65: Implement Automatic AGENTS.md Loading & Identity Alignment**
  - [x] Add `load_local_agent_rules` helper in `ext/openai-orchestrator/src/llm.rs` to scan for `.agents/AGENTS.md` and `AGENTS.md`
  - [x] Update `get_system_prompt` to align with pi-coding-agent's identity statement for RAD
  - [x] Append the loaded rules to the system prompt
  - [x] Verify compilation and run test suites


## Version 0.9.2 Local LLM Token Optimization & Status Metrics
* [x] **AWU 66: Parse Limit Configurations from rad.json**
  - [x] Update `OrchestratorConfig` in `ext/openai-orchestrator/src/mcp_client.rs` to read `max_history_messages` and `max_tool_output_chars`
  - [x] Store these settings in the global `OrchestratorState` during initialization
* [x] **AWU 67: Implement Sliding History Window in Wasm Orchestrator**
  - [x] Update `load_messages_from_dag` in `ext/openai-orchestrator/src/llm.rs` to slice history to N messages, while always keeping the first node (initial user prompt)
* [x] **AWU 68: Implement Tool Output Trimming Utility**
  - [x] Add `trim_large_output` helper in `ext/openai-orchestrator/src/tool_runner.rs` and apply it to tool responses
  - [x] Run test suites and verify stability

## Version 0.9.3 LLM Reasoning & Thought Formatting UX Optimization
* [x] **AWU 69: Unify LLM Reasoning/Thought Formatting & UX** (Current)
  - [x] Implement `\x1b[2K\r` ANSI escape sequence for erasing `Thinking...` in `src/terminal.rs`
  - [x] Add `is_reasoning` and `reasoning_buffered` to `OrchestratorState` in `ext/openai-orchestrator/src/types.rs`
  - [x] Update `sse.rs` to handle `reasoning_content` and `<thought>` tags, applying coloring and clear boundaries to terminal output
  - [x] Update `orchestrator.rs` to initialize and manage reasoning states
  - [x] Write tests and verify output format consistency

## Version 0.9.4 REPL Shell General Path Completion Fix
* [x] **AWU 70: Enable File Path Completion for General REPL Prompt** (Current)
  - [x] Update `CommandHelper::complete` in `src/command.rs` to fallback to file completion
  - [x] Add tests in `tests/command_tests.rs` to verify general prompt completion
  - [x] Run verification tests and lint checks

## Version 0.9.5 Local Installation of Updated Binary & Extensions
* [x] **AWU 71: Install Updated RAD and Wasm Extensions** (Current)
  - [x] Rebuild and compile the OpenAI Wasm Orchestrator extension
  - [x] Execute `cargo install --path .` to install the updated rad binary locally
  - [x] Verify the installed rad binary runs and version check

## Version 0.9.6 Remove Thinking Indicator Output
* [x] **AWU 72: Remove Static Thinking Indicator Text from terminal.rs**
  - [x] Delete `Thinking...` print from `TerminalState::Thinking` match branch in `src/terminal.rs`
  - [x] Re-run all verification tests
  - [x] Rebuild Wasm extension and reinstall rad binary locally

## Version 0.9.7 Hide Agent Process Outputs
* [x] **AWU 73: Hide Agent Process Outputs from REPL Terminal**
  - [x] Modify `route_event_to_terminal` in `src/ipc.rs` to not output `ProcessStdout`/`ProcessStderr` to terminal
  - [x] Re-run all verification tests (`cargo test`) and check clippy
  - [x] Rebuild Wasm extensions and reinstall rad binary locally

## Version 0.9.8 Fix AGENTS.md Autoloading
* [x] **AWU 74: Fix AGENTS.md Autoloading via Host RPC**
  - [x] Modify `load_local_agent_rules` in `ext/openai-orchestrator/src/llm.rs` to fetch `.agents/AGENTS.md` and `AGENTS.md` via `call_host(RasRpcCommand::FileRead { path })`
  - [x] Safely decode the read bytes as a UTF-8 string
  - [x] Run cargo test, clippy audits, and verify local autoloading

## Version 0.9.9 Fix LLM Memory Loss
* [x] **AWU 75: Increase Default LLM History Window Limit**
  - [x] Update default `max_history_messages` from `6` to `30` in `ext/openai-orchestrator/src/llm.rs`
  - [x] Re-run all verification tests (`cargo test`) and check clippy
  - [x] Rebuild Wasm extensions and reinstall rad binary locally

## Version 0.9.10 Silence Host RPC error log
* [x] **AWU 76: Silence WASM Host RPC Error Terminal Print**
  - [x] Remove `eprintln!` from the `Err` branch of `host_rpc` inside `src/wasm.rs`
  - [x] Re-run all verification tests (`cargo test`) and check clippy
  - [x] Rebuild Wasm extensions and reinstall rad binary locally

## Version 0.9.11 Combine multiple AGENTS.md rules
* [x] **AWU 77: Combine Multiple AGENTS.md Files Instead of First-Match**
  - [x] Modify `load_local_agent_rules` in `ext/openai-orchestrator/src/llm.rs` to combine found rules
  - [x] Re-run all verification tests (`cargo test`) and check clippy
  - [x] Rebuild Wasm extensions and reinstall rad binary locally

## Version 0.9.12 Fix 400 Bad Request
* [x] **AWU 78: Filter Out Isolated Tool Messages in llm.rs**
  - [x] Modify `load_messages_from_dag` in `ext/openai-orchestrator/src/llm.rs` to filter out orphan `tool` messages
  - [x] Re-run all verification tests (`cargo test`) and check clippy
  - [x] Rebuild Wasm extensions and reinstall rad binary locally














