# `rad` Testing Standards

This document defines the testing strategy and standards for the `rad` ecosystem, ensuring high reliability of the Core and correctness of the Extension logic.

## 1. Test Hierarchy

To ensure comprehensive coverage without sacrificing development speed, we follow a three-tier testing hierarchy.

### 1.1 Unit Tests
- **Scope**: Verification of individual functions, data structures, and isolated logic within a single module.
- **Goal**: Ensure mathematical and logical correctness of pure functions and state transitions.
- **Constraint**: Must be extremely fast. No I/O, no network, no process spawning. Use mocks for any external dependencies.
- **Execution**: Part of the standard `cargo test` workflow.

### 1.2 Integration Tests
- **Scope**: Verification of interaction between subsystems (e.g., `FS` $\leftrightarrow$ `DAG`, `Process` $\leftrightarrow$ `Network`).
- **Goal**: Ensure that the boundaries between modules are correctly implemented and that data flows correctly through the subsystem traits.
- **Methodology**: Utilize the existing trait-based design to inject mock implementations of subsystems.
- **Execution**: Run as part of the standard integration test suite.

### 1.3 E2E (End-to-End) Tests
- **Scope**: Full RPC flow from the Extension to the Core, simulating a complete agent loop.
- **Goal**: Verify the integrity of the Core-to-Extension boundary (WIT/Wasm interface), JSON serialization/deserialization, and the overall system stability under realistic scenarios.
- **Methodology**: Run a real (or simulated) Wasm runtime, triggering RPC calls and observing the Core's state changes and event dispatches.
- **Execution**: These tests may be slower and are run in CI or during major feature development.

---

## 2. Mocking Policy

Reliable testing requires isolating the component under test from volatile or complex environmental factors.

### 2.1 Trait-based Substitution (Core)
The `rad` architecture relies heavily on Rust Traits for subsystem abstraction (e.g., `FsSubsystem`, `ProcessSubsystem`). 
- **Standard Practice**: When testing Core components that depend on these subsystems, **always** provide a mock implementation of the trait.
- **Mocking Targets**:
  - **Network**: Simulate latency, timeouts, and connection failures.
  - **Filesystem**: Use an in-memory filesystem or a temporary directory to avoid side effects on the host OS.
  - **Process**: Simulate process spawns, exit codes, and stdout/stderr streams without actually executing shell commands.

### 2.2 Wasm Boundary Mocking (Extension)
When testing Extensions:
- **Test pure logic directly**: Most extension logic (frontmatter parsing, windowing, context-exhaustion classification, budget scaling) takes plain data in and returns plain data out. Test those functions without a host at all — this is where the bulk of extension coverage should live.
- **Simulate Events**: Drive an extension end-to-end by injecting `RasCoreEvent` variants and observing the resulting `RasRpcCommand` calls, as `tests/tool_loop_tests.rs` and `tests/circuit_breaker_tests.rs` do with a real runtime plus a mock LLM server.

---

## 3. Test Data Management

Maintaining a consistent and reproducible test state is critical, especially for the DAG-based history.

### 3.1 DAG (History Graph) Initialization
- **Scenario-based Construction**: Instead of relying on real user history, tests must explicitly construct the required DAG state.
- **Helper Functions**: Provide utility functions in test modules to build specific DAG topologies (e.g., `build_linear_history()`, `build_branching_history()`, `build_error_state_history()`).
- **Snapshot Loading**: For complex scenarios, use small, version-controlled snapshot files that can be loaded into the Core during test setup.

### 3.2 Filesystem & Process State
- **Ephemeral Environments**: Use `tempfile` for all filesystem-dependent tests to ensure isolation and automatic cleanup.
- **Deterministic Seeds**: If any testing involves stochastic elements (rarely used), use fixed seeds for reproducibility.

---

## 4. Continuous Integration

`.github/workflows/ci.yml` gates every push and pull request to `main`:

1. `betterleaks git --redact .` (rules in `.betterleaks.toml`)
2. `cargo check --workspace --all-targets` (ubuntu / macOS / windows)
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace`
5. Clippy and build for `wasm32-wasip2` across all six extension crates

**`--workspace` is required, not decorative.** Without it cargo selects only the
root package and silently skips every test in `ext/*` and `models/` — 52 of 149.
Any locally run check must use it too.

### 4.1 Coverage

There is no coverage measurement today: no `tarpaulin`, `llvm-cov`, or `grcov`
is configured, and CI has no coverage step. Earlier revisions of this document
stated an 85% minimum and required coverage reports in CI; both described
tooling that does not exist, so neither could be enforced or even evaluated.

If coverage becomes a requirement, add the tooling first and state the
threshold afterwards — a number with no measurement behind it is worse than
no number, because it reads as satisfied.

---

**Note**: These standards are mandatory for all contributors to the `rad` ecosystem.
