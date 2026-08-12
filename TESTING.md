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

## 2. Making a Test Mean Something

A test that passes proves nothing until you know it *can* fail. Every rule below
was learned by a test that passed while the thing it was meant to check was
broken.

### 2.1 Negative Controls Are Not Optional

**Before calling a check verification, break the thing it checks and watch it
fail.** A passing suite is consistent with the fix working, and equally
consistent with the test never reaching it.

When a change spans several sites, break them **one at a time**: if removing any
one of them fails the same set of tests, the sites are not separately covered.
Each site should have a test that is about *it*.

### 2.2 An Unfired Probe and a Probe That Cannot Fire Look Identical

Instrumenting code to prove it is unreachable — a `panic!` on entry, a counter —
only means something once the same instrument has been shown to fire somewhere
that *is* reached. Otherwise "it never triggered" and "it could never trigger"
produce the same green suite.

### 2.3 Self-Consistency Is Not Verification

A component can agree with itself while disagreeing with everything else. A
module wrote its session file to a path the host cannot read, and every
module-side test passed, because the writes and the reads went through the same
wrong path. It surfaced only when a test read the file **the way the host reads
it**.

Where two implementations must agree, have the test cross the boundary between
them rather than staying on one side.

### 2.4 A Differential Is Only as Good as the Behaviour Its Input Provokes

Comparing two implementations on an input that reaches neither one's interesting
path proves they agree about nothing. A message-list comparison passed while one
side was missing a whole feature, because the fixture's tool call carried no
argument that would have triggered it.

Build the fixture from what the code has opinions about, then assert that the
fixture provoked them.

### 2.5 A Control Has to Discriminate

Confirming that a module shares the host's type by *adding a field* to that type
proves nothing — a struct built through `Default` still compiles. Renaming a
method the module actually calls fails its build, which is the control that
distinguishes sharing from coincidence.

Ask what your control would do if the property were false. If the answer is
"the same thing", it is not a control.

### 2.6 Run the Whole Suite, Not Just the Part You Changed

Tests that pass alone and fail together are the only way process-global state
shows up: an environment variable, a singleton, a shared file. Both have
happened here — `RAD_TEST_PORT` poisoning four unrelated tests, and a module
writing every test's conversation into one file in the repo.

Use `--no-fail-fast`. Fail-fast truncates the count and hides how much actually
ran.

### 2.7 Cover the Operations That Exist, Not the Ones You Thought Of

When routing calls through a new layer, enumerate the callers rather than
recalling them. Four separate write paths to the conversation graph were missed
across three consecutive units — each found by asking "what else writes this?"
rather than by a failing test.

### 2.8 Do Not Widen Visibility to Suit a Test

If a test wants a `pub(crate)` item, reach it the way production does, or put the
test in a companion file inside the crate. Making an item `pub` for a test
changes the API to fit the test rather than the other way round.

## 3. Mocking Policy

Reliable testing requires isolating the component under test from volatile or complex environmental factors.

### 3.1 Trait-based Substitution (Core)
The `rad` architecture relies heavily on Rust Traits for subsystem abstraction (e.g., `FsSubsystem`, `ProcessSubsystem`). 
- **Standard Practice**: When testing Core components that depend on these subsystems, **always** provide a mock implementation of the trait.
- **Mocking Targets**:
  - **Network**: Simulate latency, timeouts, and connection failures.
  - **Filesystem**: Use an in-memory filesystem or a temporary directory to avoid side effects on the host OS.
  - **Process**: Simulate process spawns, exit codes, and stdout/stderr streams without actually executing shell commands.

### 3.2 Wasm Boundary Mocking (Extension)
When testing Extensions:
- **Test pure logic directly**: Most extension logic (frontmatter parsing, windowing, context-exhaustion classification, budget scaling) takes plain data in and returns plain data out. Test those functions without a host at all — this is where the bulk of extension coverage should live.
- **Simulate Events**: Drive an extension end-to-end by injecting `RasCoreEvent` variants and observing the resulting `RasRpcCommand` calls, as `tests/tool_loop_tests.rs` and `tests/circuit_breaker_tests.rs` do with a real runtime plus a mock LLM server.

---

## 4. Test Data Management

Maintaining a consistent and reproducible test state is critical, especially for the DAG-based history.

### 4.1 DAG (History Graph) Initialization
- **Scenario-based Construction**: Instead of relying on real user history, tests must explicitly construct the required DAG state.
- **Helper Functions**: Provide utility functions in test modules to build specific DAG topologies (e.g., `build_linear_history()`, `build_branching_history()`, `build_error_state_history()`).
- **Snapshot Loading**: For complex scenarios, use small, version-controlled snapshot files that can be loaded into the Core during test setup.

### 4.2 Filesystem & Process State
- **Ephemeral Environments**: Use `tempfile` for all filesystem-dependent tests to ensure isolation and automatic cleanup.
- **Deterministic Seeds**: If any testing involves stochastic elements (rarely used), use fixed seeds for reproducibility.

---

## 5. Continuous Integration

`.github/workflows/ci.yml` gates every push and pull request to `main`:

1. `betterleaks git --redact .` (rules in `.betterleaks.toml`)
2. `cargo check --workspace --all-targets` (ubuntu / macOS / windows)
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace`
5. Clippy and build for `wasm32-wasip2` across all six extension crates

**`--workspace` is required, not decorative.** Without it cargo selects only the
root package and silently skips every test in `ext/*` and `models/` — 52 of 149.
Any locally run check must use it too.

### 5.1 Coverage

There is no coverage measurement today: no `tarpaulin`, `llvm-cov`, or `grcov`
is configured, and CI has no coverage step. Earlier revisions of this document
stated an 85% minimum and required coverage reports in CI; both described
tooling that does not exist, so neither could be enforced or even evaluated.

If coverage becomes a requirement, add the tooling first and state the
threshold afterwards — a number with no measurement behind it is worse than
no number, because it reads as satisfied.

---

**Note**: These standards are mandatory for all contributors to the `rad` ecosystem.
