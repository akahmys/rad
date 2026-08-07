# rad (Rust Agent Dispatcher)

`rad` is a coding agent written in Rust, inspired by the minimalist philosophy of [pi-coding-agent](https://pi.dev/).

The Core handles mechanism only: supervising OS processes, snapshotting the filesystem, and storing conversation history as a DAG. Every decision about what the agent does — prompting, tool selection, safety policy, context management — runs in sandboxed WebAssembly extensions loaded at startup. The Core has no tools, no prompts, and no model-specific code of its own.

---

## 1. Features

*   **Separate Core and extension binaries**: The Core installs as a single `rad` binary; the five extensions are `.wasm` files loaded from paths given in the config. An extension can be rebuilt and swapped in with `/reload` without restarting or rebuilding the Core.
*   **Process group cleanup (PGID)**: Each process the agent spawns is placed in its own process group. When the Core exits — normally, on `Ctrl+C`, or by panicking — it sends `SIGKILL` to every registered group, so descendants are not left running.
*   **Snapshot-based rollback, separate from Git**: File changes are snapshotted to `.rad/snapshots/` before each step, so the workspace can be restored to a checkpoint without involving Git history.
*   **Capability-centric Wasm interface (UCCA)**: Extensions work through typed resource handles (`stream-handle`, `file-handle`, `execution-handle`) instead of flat JSON RPC, which keeps stream reads pull-based and resource access capability-gated inside the guest.
*   **All policy in sandboxed extensions**: Prompting, tool resolution, safety filtering, and context management run as WebAssembly, split into five single-responsibility extensions that communicate over WIT-defined interfaces:
    *   **LLM Orchestrator**: Builds the prompt and drives the reasoning/tool loop.
    *   **Security Guard**: Approves or denies each resource request before the host hands back a handle.
    *   **Tool/MCP Provider**: Connects to external MCP servers and merges their schemas into one tool pool.
    *   **Skill Provider**: Turns Markdown skill files under `.agents/skills/` (or `~/.rad/skills/`) into tools the model can discover on its own — see [CONFIG.md](CONFIG.md) §2.5.
    *   **LLM Connector**: Converts messages and tool definitions into model-specific payloads, and parses the response stream.
    *   **Context Compactor** (a kernel module, configured under `modules` — see CONFIG.md): Decides what history survives once the Orchestrator has assembled it — windowing, stale tool-result clearing, relevance-based retention.
*   **Stateless extensions over a shared DAG**: Extensions keep no session state of their own; each turn they rebuild history from the Core's DAG. An extension that crashes mid-task is respawned and rehydrated from the DAG, so the conversation is not lost.

---

## 2. Quick Start

### 2.1 Prerequisites

*   **A Unix-like OS** — Linux or macOS. `rad` uses POSIX process groups for
    the descendant cleanup described in §1, termios raw mode for `Esc` abort,
    and `pre_exec` when spawning. It does not build on Windows.
*   Rust 1.85 or higher (the workspace is on edition 2024)
*   `cargo`

### 2.2 Build & Installation (One-Command)

```bash
# Clone the repository
git clone https://github.com/akahmys/rad.git
cd rad

# Add WASM target (if not already added)
rustup target add wasm32-wasip2

# Run the unified build & deployment script
./scripts/build_all.sh

# Run rad
rad
```

### 2.3 Running Tests

```bash
cargo test --workspace
```

`--workspace` is required, not optional: a bare `cargo test` only builds the root package and silently skips every test in the extension crates (`ext/*`) and `models/` — roughly a third of the suite.

Tests run in parallel. The integration tests that share process-global environment variables serialize themselves with an in-file `TEST_MUTEX`, so no `--test-threads=1` is needed.

---

## 3. User Guide

### 3.1 CLI Usage & Interaction
Running `rad` opens an interactive shell (REPL) where you talk to the agent.

* **Direct Chat**: Type a request and the agent runs its autonomy loop until it resolves.
* **Shell Escape (`!`)**: Prefix any command with `!` to run it directly on your host shell without LLM involvement.
  ```bash
  ! ls -la
  ```
* **Metadata & Slash Commands**: Use commands prefixed with `/` for CLI control:
  * `/help`: List all available commands.
  * `/quit`: End the session and exit.
  * `/session`: Show the session ID, DAG history nodes, and accumulated token usage.
  * `/rollback <node_id>`: Restore the workspace files and conversation context to the snapshot taken at the given DAG node.
  * `/reload`: Re-read the config, reapply permissions, and drop the cached Wasm runtimes — use this after replacing an extension's `.wasm` so the next task loads the new build.
  * `/new`: Save the current session and start a clean one (rotates the session ID and clears the DAG).
  * `/tree`: Render the history DAG as a tree in the terminal.
  * `/tools`: List active permissions and registered tool definitions.
  * `/llm` (alias `/models`): Manage LLM endpoint profiles (list, switch, test, add, model, delete, context).
  * `/compact`: Compact and persist session history now, instead of waiting for the per-turn compaction that applies only ephemerally.

### 3.2 Capability Mask & Extension Configuration
Filesystem and process operations that an extension requests **through the host RPC surface** are validated against the extension permissions registered in `~/.rad/config.json` (user-global) or a project-local `rad.json`/`config.json` override. If an action is not authorized in this capabilities mask, the API Gateway rejects the operation.

> [!IMPORTANT]
> The mask is a guard rail for cooperating extensions, not a containment boundary. Extensions receive WASI preopens for the working directory and `$HOME`, so one that calls `std::fs` directly bypasses the mask entirely; and tools run inside MCP server processes that hold your full user privileges, which `rad` never mediates. [ARCHITECTURE.md](ARCHITECTURE.md) §1.1 documents both limits and how they were verified. Register only MCP servers you trust.

None of the five extensions `rad` ships provides tools directly. File and shell access comes only from MCP servers registered under `mcp-tool-provider`'s `config.mcp_servers`; skills contribute further tools but not general-purpose file or shell access. Until at least one MCP server is configured, the agent has nothing to act with — see §3.3.

> [!NOTE]
> The full config schema (with a working example), the 5-tier precedence cascade, and the on-disk directory layout are documented in [CONFIG.md](CONFIG.md) — the authoritative reference, kept here as a single source of truth rather than duplicated.

### 3.3 Companion MCP Servers

Any stdio MCP server works — `rad` is not coupled to a particular one. These two are developed alongside `rad` and are what it is dogfooded against day to day:

| Server | Provides | Tools |
| :--- | :--- | :--- |
| **[core-utilities-mcp](https://github.com/akahmys/core-utilities-mcp)** | Filesystem, shell, and structured-data operations. Its `edit_file` is content-addressed — it locates the target by surrounding text rather than line numbers, so an edit does not silently apply to the wrong place after earlier edits shift the file. | 15 |
| **[web-access-mcp](https://github.com/akahmys/web-access-mcp)** | Web fetching and search, using a headless Chromium instance for pages that require JavaScript. Requires Chrome, Chromium, or Edge on the host. | 4 |

```bash
# Build and install both (each exposes a stdio binary)
git clone https://github.com/akahmys/core-utilities-mcp.git && cd core-utilities-mcp && cargo install --path . && cd ..
git clone https://github.com/akahmys/web-access-mcp.git   && cd web-access-mcp   && cargo install --path . && cd ..
```

Then register them under `mcp-tool-provider`'s `config.mcp_servers` in `~/.rad/config.json`:

```json
"config": {
  "mcp_servers": {
    "core-utilities": { "command": "~/.cargo/bin/core-utilities-mcp", "args": [] },
    "web-access":     { "command": "~/.cargo/bin/web-access-mcp",     "args": [] }
  }
}
```

On the next start, `rad` reports what it resolved — with both registered you should see `[OK] Verified 19 tools from extension 'mcp-tool-provider'`.

---

## 4. Developer Guide

### 4.1 Architecture Overview
`rad` is designed with a clean separation of concerns:
* **Core (Rust)**: Manages OS resources, process PGID tracking, DAG state history, snapshots, and the WASM runtime host. It acts as an API Gateway executing RPCs.
* **Extensions (WASM)**: Consist of the six isolated micro-extensions listed in §1 (LLM Orchestrator, Security Guard, Tool/MCP Provider, Skill Provider, LLM Connector, Context Compactor), communicating via WIT-defined interfaces. They are stateless and restore history dynamically from Core's DAG.

### 4.2 Unified Error Handling

`rad` employs a transactional, 3-pillar error handling approach designed to treat errors as state transitions within the DAG to ensure system consistency and enable autonomous recovery.

1. **Pillar 1: Error Normalization (`UnifiedError`)**: Categorizes all host runtime and subsystem errors into three distinct recovery levels:
   * **L1 (Adaptation)**: Transient network drops or tool/command errors. *Strategy*: Log & retry with LLM context feedback.
   * **L2 (Rollback)**: LLM output parsing failures (e.g. truncated tool call arguments from output token limits). *Strategy*: Roll back DAG pointer and physically restore workspace using file snapshots.
   * **L3 (Reset)**: Context window exhaustion — the backend rejected the request as too large. *Strategy*: shrink the context budget by a backoff factor, re-compact via `context-tools`, and re-issue the turn, bounded by a retry cap so a non-budget failure can't loop forever.
2. **Pillar 2: Deterministic State Transition & File Rollback**: Ensures directory states (`.rad/snapshots/`) are synchronized with DAG pointer rewinds during L2 rollbacks.
3. **Pillar 3: Dual-Track Feedback**: Separates technical error traces (sent to the LLM for self-correction) from clean, semantic notification strings (displayed to the user, e.g., `"Error: Model stopped because it reached the maximum output token limit. The response may be incomplete."`).

Two bounded-retry guards complement this, both converting a potentially infinite loop into a bounded one that stops with a clear diagnosis: the L3 context backoff above, and a **tool-failure circuit breaker** that halts the task when the same tool fails repeatedly in a row. See [ARCHITECTURE.md](ARCHITECTURE.md) §5.1.2–5.1.3 for the full specification.

### 4.3 Building WebAssembly Extensions
To rebuild a single Wasm extension (e.g. the LLM orchestrator) during development:
```bash
cargo build --target wasm32-wasip2 --release -p rad-orchestrator
```
This only produces `target/wasm32-wasip2/release/rad_orchestrator.wasm` — it does **not** install it. `rad` loads each extension from the `source` path in its config, so either point `source` at the build output, or run `./scripts/build_all.sh`, which copies all five extensions plus the context module into `~/.rad/wasm/`. After replacing a `.wasm` that a running session already loaded, use `/reload` to drop the cached runtimes so the next task picks up the new binary.

When changing `wit/rad.wit`, mirror it into `templates/rust/wit/` and `templates/go/wit/` — those are standalone copies used by scaffolded extensions, and `build_all.sh` fails the build if they drift.

### 4.4 Running Tests & Compliance
Before contributing code, verify compliance with project standards:
* **Code Size limits**: Ensure functions and files adhere to the length limits described in `CODING.md` (e.g., maximum 300 lines per file).
* **Verify Tests**: Run the whole workspace (see §2.3 for why `--workspace` matters):
  ```bash
  cargo test --workspace
  ```
* **Lint Check**: Warnings must fail the build, matching what `scripts/build_all.sh` enforces:
  ```bash
  cargo clippy --workspace -- -D warnings
  ```
* **Both Targets**: Extensions compile to WebAssembly, so lint and build them for `wasm32-wasip2` as well — a native-only check can miss target-specific breakage:
  ```bash
  cargo clippy --target wasm32-wasip2 -p rad-orchestrator -p llm-connector \
      -p security-guard -p mcp-tool-provider -p skill-tool-provider -p context-tools -- -D warnings
  ```

Running `./scripts/build_all.sh` performs all of the above (plus the WIT sync gate, formatting, license, and secret scans) in one pass.

---

## 5. License

[MIT License](LICENSE)
