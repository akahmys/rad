# TASKS

Last Updated: 2026-07-05

## Completed Milestones
- [x] Initial Setup, Configuration & Architecture (v0.0)
- [x] Core Subsystems: Process, FS, DAG, Wasm, PTY, HTTP, CI (v0.1)
- [x] Single-Process Agent Shell: REPL, Event Streaming, Human-in-the-loop, Session (v0.2)
- [x] Wasm Extension: OpenAI Compatibility & Enhanced REPL UX (v0.2.1)

## Version 0.2.2 DAG-Based Context Management in Wasm Extension (Current)
- [ ] AWU 20: Add GetDag RPC to Core and Wasm API
  - [ ] Add `GetDag` to `RasRpcCommand` in both `src/ipc.rs` and `ext/openai-orchestrator/src/lib.rs`
  - [ ] Implement `GetDag` handling in the Core Wasm host RPC handler returning serialization of `Dag`
- [ ] AWU 21: Refactor Wasm Extension to Load and Persist History using DAG
  - [ ] Update `ext/openai-orchestrator/src/lib.rs` to query `GetDag` on input events
  - [ ] Reconstruct `messages: Vec<Message>` by traversing history nodes in DAG topological order
  - [ ] Save new user inputs (`CreateNode`, `SetNodeText`) and assistant responses into the DAG
  - [ ] Remove `STATE` memory-based message array persistence
- [ ] AWU 22: Verify Context Restoration & Zero-Warning Audit
  - [ ] Write integration test validating context restoration across session restarts/Extension reloads
  - [ ] Achieve zero Clippy warnings, check secrets, and ensure all tests pass
- [x] AWU 23: Translate README.md to English & Document pi-coding-agent Inspiration (Current)
  - [x] Translate Japanese contents of `README.md` to English
  - [x] Explicitly describe that `rad` is inspired by `pi-coding-agent`
