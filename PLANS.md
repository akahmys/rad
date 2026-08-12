# Project Work Plan (PLANS.md)
**Last Updated**: 2026-08-09

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 10: Codebase Refactoring & Rule Alignment (v0.15.0)
- [✅] Phase 11: Unified Error Handling Mechanism (v0.16.0)
- [✅] Phase 12: Codebase Verification & Integrity Audit (v0.17.0)
- [✅] Phase 14: Esc Key Task Abort Polish & Robustness (v0.19.0)
- [✅] Phase 15: Release Build, Local Installation & Push (v0.20.0)
- [✅] Phase 16: Separate Core and MCP Tool Providers (v0.21.0)
- [✅] Phase 17: Release Build, Local Installation & Push to Main (v0.22.0)
- [✅] Phase 18: Add Timeouts to Web Access and Fix Hangs (v0.23.0)
- [✅] Phase 19: Remove Built-in Core Tools and Web Access Extensions from rad (v0.24.0)
- [✅] Phase 20: Standardize Global Configuration Directory to ~/.rad/config.json (v0.25.0)
- [✅] Phase 21: Release Build, Local Installation, ~/.rad/config.json Creation & Push to GitHub (v0.26.0)
- [✅] Phase 22: Fix LLM Connection Hangs & Add Graceful Error Handling (v0.27.0)
- [✅] Phase 23: Configurable LLM Endpoints & /llm Slash Command Management (v0.28.0)
- [✅] Phase 24: Model-Agnostic Architecture Refactoring (v0.29.0)
- [✅] Phase 25: Unified Event Tracing & Distributed Logging (v0.30.0)
- [✅] Phase 26: WASM I/O & Streaming Performance Optimization (v0.31.0)
- [✅] Phase 27: One-Command Build & Deployment Automation (v0.32.0)
- [✅] Phase 28: Documentation Update, Config Deployment & Git Main Release (v0.33.0)
- [✅] Phase 29: MCP Host Tilde Expansion & Instant Tool Result Visibility (v0.34.0)
- [✅] Phase 30: Complete Removal of Built-in Shell Fallbacks & Accurate MCP Startup Verification Output (v0.35.0)
- [✅] Phase 31: Stdio Pipe Fallback for PTY Permission Errors in Process Manager (v0.36.0)
- [✅] Phase 32: Fix Tilde Expansion in WASM Permissions & Align MCP Config Discovery (v0.37.0)
- [✅] Phase 33: Fix `read_config_file` Error String Swallowing in WASM MCP Provider (v0.38.0)
- [✅] Phase 34: Fix Tilde Expansion in Host Fs Subsystem (`src/fs.rs`) (v0.39.0)
- [✅] Phase 35: Unified Path Resolution Architecture & Gateway Normalization (v0.40.0)
- [✅] Phase 36: Standardize Stdio Piping for MCP Process Spawning & Fix PTY JSON-RPC Corruptions (v0.41.0)
- [✅] Phase 37: Fast Direct Binary Execution & Robust Read Loop for MCP Server Spawning (v0.42.0)
- [✅] Phase 38: Deep End-to-End Codebase Audit & Verification of MCP Subsystem (v0.43.0)
- [✅] Phase 39: Consolidate Context Compaction Logic into `context-tools` Extension (v0.44.0)
- [✅] Phase 40: Post-Consolidation Audit Fixes — Orphan-Filter Ordering & CallExtension Deadlock Risk (v0.45.0)
- [✅] Phase 41: Second Audit Pass — Unsafe Tool-Call Squashing, File-Size Limit, Production Unwrap (v0.46.0)
- [✅] Phase 42: Full File-Size Limit Compliance — Split All Remaining 300+ Line Files (v0.47.0)
- [✅] Phase 43: Extension Crates Clippy Audit & Hand-Written Allow Cleanup (v0.48.0)
- [✅] Phase 44: Release Build, Verification Audit & Push to Main (v0.49.0)
- [✅] Phase 45: Fix Hardcoded Model Name & Local-LLM Context-Overflow Prevention (v0.50.0)
- [✅] Phase 46: RPC Contract Consolidation & `context-tools` WIT Unification (v0.51.0)
- [✅] Phase 47: Slash Command Registry Redesign (v0.52.0)
- [✅] Phase 48: Security Policy & Extension Config Hygiene (v0.53.0)
- [✅] Phase 49: Context-Overflow Feature Completion (Manual Override & Exact Tokenization) (v0.54.0)
- [✅] Phase 50: Session Storage Operational Hygiene (v0.55.0)
- [✅] Phase 51: Advanced Context Compression Techniques (v0.56.0)
- [✅] Phase 52: Slash Command Composition Review, Role-Based Extension Lookup & Manual Compaction (v0.57.0)
- [✅] Phase 53: File-Size Limit Compliance Cleanup (v0.58.0)
- [✅] Phase 54: Real-World Dogfooding Fixes — MCP/LLM Onboarding, Doc Consolidation & Eager-Load Env-Var Bug (v0.59.0)
- [✅] Phase 55: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider` (v0.60.0)
- [✅] Phase 56: Performance Audit — wasmtime Compile Cache & MCP Tool-List Caching (v0.61.0)
- [✅] Phase 57: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem (v0.62.0)
- [✅] Phase 58: Skills — Autonomously-Discoverable Markdown Tool Definitions (v0.63.0)
- [✅] Phase 59: Documentation Audit & Codebase Synchronization (v0.64.0)
- [✅] Phase 60: Architecture Design & Codebase Alignment Re-verification (v0.65.0)
- [✅] Phase 61: Fix False-Positive [FAILED] Tool Initialization Status for Non-MCP Extensions (v0.66.0)
- [✅] Phase 62: Make LLM Thinking Process Display Independent of RAD_DEBUG (v0.67.0)
- [✅] Phase 63: Release Build, Local Installation & Push to Main (v0.68.0)
- [✅] Phase 64: Edit-Failure & Infinite-Loop Countermeasures — Shell-Quoting Corruption Fix, Consecutive-Failure Circuit Breaker & Content-Addressed `edit_file` (v0.69.0)
- [✅] Phase 64: Fix & Update scripts/build_all.sh Pipeline (v0.69.0)
- [✅] Phase 65: Comprehensive Workspace Test Suite Audit (v0.70.0)
- [✅] Phase 66: Rebuild WASM Components & Local Installation of rad Binary (v0.71.0)
- [✅] Phase 67: Spec-First Architecture Reconciliation — ARCHITECTURE.md & README.md Realigned, L3 Recovery Implemented, FS Watcher Deleted (v0.72.0)
- [✅] Phase 68: Repository Hygiene & Convention Audit — Authorship Rewrite, CI Workspace Fix, Rule Documents Corrected (v0.73.0)
- [✅] Phase 69: Microkernel Migration — Preparation & Stage 0 (v0.74.0)
- [✅] Phase 70: Microkernel Migration — Kernel Surface Alongside the Existing One (v0.75.0) (`ARCHITECTURE-NEXT.md` §9 stages 1–2)
- [🔄] Phase 71: Microkernel Migration — Extensions to Modules, One at a Time (§9 stages 3–8; stages 3–6 done)
- [ ] Phase 73: Windows Support — post-migration, once `proc-spawn` is the single point of process supervision (`ARCHITECTURE-NEXT.md` §8)
- [ ] Phase 72: Microkernel Migration — DAG/UI Extraction and Author Tooling (§9 stages 9–10)

---

## 🛠️ Short-Term Plan: Phase 71 (Extensions to Modules, One at a Time)

Stages 3 through 6 of `ARCHITECTURE-NEXT.md` §9 are done: `context-tools`,
`skill-tool-provider`, `mcp-tool-provider` and `llm-connector` are modules.
**Stage 7 (`security-guard` → `policy`) is next** — start at that section below,
then the state summary under it.

§9.4's invariant holds throughout: rad works at the end of every AWU, and
`wit/rad.wit` is untouched.

### 💡 Current AWU Status (stage 6 — complete)
- [x] AWU 966: `net-open` — the fallible `byte-stream` and the 504 convention
- [x] AWU 967: `modules/llm-openai` — port the dialect table and the SSE parser
- [x] AWU 968: Route `GenerateLlmStream` to the module
- [x] AWU 969: Delete `ext/llm-connector`

Three decisions were taken before the split, and each is recorded where it will
be questioned again:

- **`async_support` stays off.** §3.6 defers it until `net-open` exists, so this
  was the point to decide. `net-open`'s host side is a thread plus a channel,
  exactly like `proc-spawn`'s, so bounded polling gives the same non-blocking
  property without it — while turning it on would make every host import async
  and rewrite every call site, including the two remaining extensions' interop
  path. That is not "one extension moved" (§9.1). **Revisit when `agent-loop`
  becomes a module (stage 8)**, which is where §3.6.4's `post`-driven scheduler
  — the actual beneficiary — arrives.
- **The method names are `llm.generate` / `llm.next`, unprefixed.** The
  `<module>.tools.list` convention exists because tools are plural and
  aggregated (`src/kernel/tools.rs`). Generation is singular: exactly one
  transport serves, and §4.2's second transport is an alternative rather than a
  peer. So the registry routes by method, and two installed transports are a
  startup collision (§3.6.8) — which is the correct answer, not a limitation.
- **URL resolution moves to the host**, as far as it can go. `RAD_TEST_PORT`,
  `normalize_base_url`, the `https://api.openai.com` default and the "No LLM
  endpoint configured" error become host-side; `dialect.url()` and
  `dialect.headers()` stay in the module, because moving those would put the
  dialect table back in the host and dissolve §4.2's reason for the module to
  exist. The module then reads no environment at all.
  - One behavioural difference, invisible today: `RAD_TEST_PORT` currently
    hard-codes `/v1/chat/completions` and ignores the dialect, so a non-default
    dialect combined with the test port will produce a different URL afterwards.
    No test does that — `dialect` is set in exactly one test file, to `None`.
  - `tests/llm_connector_eager_load_tests.rs:167` asserts the **absence** of
    "No LLM endpoint configured", so it passes vacuously if the wording drifts.
    Moving the message host-side is exactly when that would happen: AWU 968 owns
    both the move and a test that fails when the wording changes.

#### AWU 966: `net-open` — the fallible `byte-stream`
- **Objective**: The second syscall, and the last one. `proc-spawn` already
  returns a `byte-stream`; what was missing was the HTTP side.
- **Done**. `src/kernel/net.rs` (the request machinery, ported from
  `src/wasm/imports_http.rs` rather than rewritten) and `src/kernel/stream.rs`
  (the resource, moved out of `proc.rs` because both syscalls hand it back and
  neither file has room for both under the 300-line limit). Driven from
  `modules/net` — a fixture, `ship = false` — through `tests/kernel_net_tests.rs`.
- **An empty read cannot mean two things.** This is the whole design of the
  file. `proc.rs` maps both "nothing yet" and "the pipe closed" to an empty
  read, which is honest because `process.wait` distinguishes them. HTTP has no
  `wait`, and the SSE parser already treats an empty read as end-of-response —
  so porting that mapping unchanged would have ended a response 100ms into a
  slow first token. "Nothing yet" is therefore a `504` error, reusing the
  "call again" convention `WAIT_PENDING` already established, and empty keeps
  meaning "the body is over".
- **A transport failure must not arrive as bytes or as silence.** Non-2xx, a
  refused connection, a stalled peer: all reach the guest as an error, which is
  why `KernelStream` grew `Fallible` alongside `Reader`. The extension host
  reached the same split (`PipeReader` / `PipeReaderFallible`) for the same
  reason. Confirmed by removing the error arm: the 404 test then reports a
  *successful* call with an empty body — a truncated answer presented as a
  complete one.
- **A latent data-loss bug in `proc-spawn`'s reader, fixed here.** `read(max)`
  truncated any chunk larger than `max` and discarded the remainder. Harmless so
  far only because the process reader thread chunks at 1024 bytes while every
  caller asks for 4096 — a coupling nothing enforced. Off a socket, chunks
  routinely exceed 4096. Both variants now hold the remainder (`Incoming`), and
  `read(0)` is rejected rather than answered with an empty slice that a
  `Fallible` reader would read as end-of-response.
- **Every assertion was checked by breaking the thing it guards**: swallowing
  the error (404 → empty success), collapsing `PENDING` into empty (body
  truncated to its first half), removing the leftover buffer (4,000 bytes → 64),
  and dropping the headers before sending (the dialect's `Authorization` line
  going nowhere).
- **The `security-guard` gap now has a second occurrence.** The extension host
  runs `verify_rpc_exclude` before every request; the kernel cannot, holding no
  orchestrator handle. Same shape as `proc-spawn`'s, same owner: the `policy`
  module (§3.4.3, stage 7). Two occurrences rather than one strengthens the case
  for doing it there rather than special-casing either syscall.
- `boot` now takes the whole `Config`. The settings the kernel reads are spread
  across `modules`, `core` and `default_timeout`, and a positional list of a
  `&str`, a `bool` and a `u64` is a transposition waiting to happen.
- `src/kernel/host.rs`'s `unimplemented` helper is gone: with `net-open` landed,
  all three syscalls are implemented and the surface is closed at three (§3.1).

#### AWU 967: `modules/llm-openai` — the dialect table and the SSE parser
- **Objective**: `llm-transport-openai` (§4.1) as a module. The old extension
  stays live and serving; AWU 968 switches the caller over.
- **Done**. `modules/llm-openai/`, driven by `tests/llm_module_tests.rs` (the
  response) and `tests/llm_module_request_tests.rs` (what goes out).
- **`dialect.rs` and `dialect/tests.rs` moved byte-identical**, proven with
  `diff`. That table is §4.2's whole reason for this module to exist, so it is
  the one part that must be seen not to have changed.
- **Measured**: 574 lines become 845, or 455 against 575 ignoring comments and
  blanks. It grew, and the growth is all test: the extension had 111 lines of
  tests and the module has 264. Production code is 463 against 344 — and the
  extension's 344 excludes the ~90 lines of host-side conversion
  (`RemoteMessage`, `RemoteTool` in `rpc_meta_llm_connector.rs`) that AWU 968
  deletes.
- **The wire types collapse from three sets to one.** A WIT record, a serde
  struct, and the host's parse target all described the same OpenAI shape, with
  `connector.rs` converting between the first two field by field on every
  request. A module receives JSON, so the wire shape is the only shape — the
  same collapse `context` saw in AWU 956. `parameters` stops being a string
  containing JSON, which was a double encode that existed only to cross the
  boundary.
- **Two bugs found in the extension while porting, both fixed here.**
  - **A multi-byte character split across chunks failed the whole response.**
    `String::from_utf8` per chunk, and nothing aligns TCP segments to UTF-8 —
    so a model answering in Japanese could fail for no reason but where the
    packet split. `Session::decode` keeps the incomplete tail. Verified: with
    the per-chunk decode restored, the test reports the extension's own
    "Invalid UTF-8 chunk received".
  - **A final `data:` line with no trailing newline was dropped silently.** The
    parser only ever consumed up to a `\n`. `pump` feeds a terminator at
    end-of-body. Verified: without it the event never arrives.
- **The event JSON is unchanged and now asserted.** `{"ContentChunk": "..."}`,
  externally tagged, snake_case fields — what
  `ext/rad-orchestrator/src/orchestrator/reasoning.rs` deserializes. A unit test
  pins the exact bytes, because the consumer stays an extension until stage 8.
- **No `unsafe`.** The session lives in a `thread_local`, not a
  `static Mutex`, so the non-`Send` stream handle needs no `unsafe impl Send`.
  `modules/mcp` has one; CODING.md §4 prohibits `unsafe` outright, and a
  thread-local is the version that does not need the exemption. Worth applying
  to `mcp` separately.
- The scripted split of the integration test file removed a neighbouring
  function's body on the first attempt — the same failure as stage 3's
  timeout-assertion edit and stage 5's seven test files. **Three occurrences
  now.** Redone by hand.

#### AWU 968: Route `GenerateLlmStream` to the module
- **DoD**: A turn completes with no `llm-connector` extension configured, and
  the same run without the module produces no answer.
- **Done**. `src/wasm/rpc_meta_llm_module.rs`; the module answers first when one
  is loaded, the same bridge `CallExtension` uses, so removing `llm-openai` from
  `modules` falls straight back to the extension.
  `tests/llm_module_e2e_tests.rs` is the pair. Verified by making
  `is_available` return `false`: the answer never reaches the conversation.
- **Endpoint resolution moved to the host**, as agreed: `RAD_TEST_PORT`,
  `normalize_base_url`, the `api.openai.com` default and the "No LLM endpoint
  configured" message. The dialect stayed in the module. The module now reads no
  environment at all.
- **The eager-load test's blind spot is closed.**
  `tests/llm_connector_eager_load_tests.rs:167` asserts that message is
  *absent*, which passes vacuously the moment the wording drifts — and moving
  the message was exactly when that would happen. A unit test now pins the
  string; both fail if it changes.
- **A kernel bug that this AWU would have shipped.** `call_stack` was one
  `Mutex<Vec<String>>` for the whole kernel, so two *threads* calling one module
  looked like re-entrancy: the polling loop holds `llm-openai` while anything on
  the main thread calling it gets `dispatch cycle: llm-openai -> llm-openai`.
  §3.6.3's hazard is A→B→A, which is one thread by construction — `dispatch.call`
  runs the target on the caller's thread and suspends the caller — so the stack
  is now thread-local. Nothing drove a module from a background thread before
  this AWU, which is why it had never fired.
  `two_threads_calling_one_module_is_not_a_cycle` fails against the old shape.
  - **Not fixed, and recorded**: two threads each holding one module's lock and
    calling into the other's would deadlock on the locks, below where this check
    sits. No module pair can do that today — the transport calls nothing — and
    the answer is §3.6.1's single scheduler, which arrives with `agent-loop`.
- The host still models nothing about the request: `messages_json` and
  `tools_json` are spliced in as parsed JSON. `RemoteMessage`/`RemoteTool` and
  the WIT conversion still exist for the extension path, and go with it in 969.

#### AWU 969: Delete `ext/llm-connector`
- **DoD**: Extension gone, suite green, generation still works.
- **Done. Stage 6 complete: 2 extensions, 4 modules.** Crate, `wit/connector/`,
  `bindings::rad_llm_connector`, `WasmRuntime::llm_connector`, the loader's
  `"llm-connector"` role branches, the connector-only host impls in
  `imports_resources_exec.rs`, and `rpc_meta_llm_connector.rs` are all gone.
- **Every test moved before the extension did**, per the rule stage 5 set. Ten
  files swapped their `llm-connector` extension entry for an `llm-openai`
  module entry, and `llm_connector_eager_load_tests.rs` — the only test that
  drives a *real* `llm.endpoints` config rather than the `RAD_TEST_PORT` bypass
  — was migrated and renamed `llm_endpoint_config_tests.rs` rather than deleted
  with its subject. The module cannot have the bug it guards (it reads no
  environment), but that is a property worth holding in place.
- **The edit was scripted, and the script's assertions caught two things** —
  which is the whole reason for asserting rather than trusting a regex. A
  `rad::config::ExtensionConfig` path prefix broke the brace match on the second
  file, and two stale doc comments naming `mcp-tool-provider` (already gone
  since stage 5) surfaced as "leftover reference". Neither was the failure mode
  that bit three times before; the guard is what made the difference.
- 256 passed, down exactly 8 from 264: the extension's own `dialect/tests.rs`,
  whose copies live in the module and were counted twice while both existed.
- With the extension gone there is no fall-back branch left, so an orchestrator
  with no transport module now gets a named error rather than the
  no-orchestrator path's hardcoded endpoint, which would have surfaced as a
  connection failure explaining nothing.

### 🚧 In progress: stage 7 (`security-guard` → `policy`)

- [x] AWU 970: Delete the three verification sites no guest reaches
- [x] AWU 971: `modules/policy` — the blocklist as a module
- [x] AWU 972: `mcp` asks `policy`, and `verify_tool_call` goes in the same AWU
- [x] AWU 973: Delete `ext/security-guard` and the host's verification machinery
- [x] AWU 974: §3.4.4 — write down what is not defended
- [x] AWU 975: remove `modules/mcp`'s unsafe `Send` claim
- [x] AWU 976: bring `~/.rad/config.json` to the current architecture

**The open design question is settled: one hook, inside `modules/mcp`.**
Moving from five call sites to one is not a narrowing of what is enforced —
four of the five enforce nothing today, and the narrowing already happened in
stages 5 and 6 as a side effect of deleting their callers. Measured before
deciding:

| site | guest that calls it | branch in `policy::evaluate` | live |
|---|---|---|---|
| `imports_tool.rs:25` `execute_tool` | `rad-orchestrator` | `ExecuteTool { arguments }` | **yes — the only one that ever blocks** |
| `imports_rpc.rs:32` `host_rpc` | `rad-orchestrator`, 13 commands | none of them hit a branch | yes, always allows |
| `imports_rpc.rs:92` `open_file` | nobody | `_ => true` | no |
| `imports_http.rs:43` `open_http_stream` | nobody (AWU 969) | `_ => true` | no |
| `imports_process.rs:69` `open_process` | nobody (AWU 965) | `SpawnBashProcess` | no |

**`proc-spawn`/`net-open` do not get a check, and the recorded "gap" is
closed upstream rather than filled.** §3.4.2 discarded the `syscall-gate` role
and its reasoning still holds in the code: model-derived data reaches neither
`argv` nor the URL. `mcp` spawns from `kernel.config` and sends the model's
tool calls over an already-running server's *stdin*; `llm-openai`'s URL is
config-derived. The one exception is `testmode`, where `bash -c <model's
command>` does put model text in `argv` — and the tool-execution hook sees the
same string first. The one hook is upstream of both kernel sites on every path
where model-controlled data exists. AWU 972 tests that rather than asserting it.

**`block_path_patterns` is already dead.** The `write` tool is served by
`modules/mcp/src/testmode.rs` through `bash`, not through `FileWrite`, and
`rad-orchestrator` issues no `FileWrite` at all. The only thing still executing
that branch is `src/wasm/tests.rs:119`, which drives `verify_rpc` directly.
AWU 971 measures this before removing it.

Read before planning: `ARCHITECTURE-NEXT.md` §3.4.3 (policy is cooperative),
§3.4.4 (the limit to state plainly), §3.4.5. What is already known from reading
the code, so it is not re-derived:

- **The extension is 148 lines and almost pure.** `ext/security-guard/src/`:
  `lib.rs` 76, `policy.rs` 72. `verify_rpc(command) -> bool` plus a blocklist
  fetched once via `GetExtensionConfig` and cached in a `thread_local`. It makes
  no other host call. Of the 76 lines in `lib.rs`, roughly 40 are WIT conversion
  macros that exist only to cross the extension boundary and go with it.
- **The host has five call sites, and they are all `src/wasm/`** —
  `imports_tool.rs:25`, `imports_http.rs:43`, `imports_process.rs:69`,
  `imports_rpc.rs:32` and `:92` — all reaching
  `Orchestrator::verify_rpc_exclude` (`src/orchestrator/runner/events.rs`).
  That function walks every *extension* and asks each one. With two extensions
  left, and `rad-orchestrator` the only non-guard among them, most of that
  fan-out is now vestigial.
- **The kernel has no equivalent, and that is the recorded gap.** `proc-spawn`
  (AWU 965) and `net-open` (AWU 966) both skip the check because the kernel
  holds no orchestrator handle. Two occurrences, both pointing here.
- **A syscall asking `policy` is a plain module-to-module `call`, and the cycle
  check already covers it.** A module reached through `call` is on the
  thread's stack, so `proc-spawn` → `policy` pushes a second frame and returns;
  a `policy` that tried to spawn something itself would be refused by name
  rather than deadlocking. Worth a test rather than an assumption.
- **`GetExtensionConfig` has no module equivalent and does not need one.** A
  module's config comes from `kernel.config` (`KernelShared::handle_kernel`),
  which is how `mcp` and `skills` already read theirs.
- **Four test files register `security-guard`**: `multi_extension_tests.rs`,
  `multi_extension_isolated_roles_tests.rs`, `security_guard_policy_tests.rs`,
  `security_hook_tests.rs`. Plus `src/wasm/tests.rs` drives `verify_rpc`
  directly against a real component. All five move before the extension does.
- **§3.4.4 is part of the work, not commentary.** `ARCHITECTURE.md` §1.3 claims
  the security guard prevents prompt-injection damage; §3.4.4 calls that an
  overclaim and says not to pretend. Stage 7 is when that sentence gets fixed.
- ~~**Open design question, not yet decided**~~ — settled above: one hook, in
  `modules/mcp`. Mostly deletion, not a redesign.

#### AWU 970: Delete the three verification sites no guest reaches
- **Objective**: Remove `verify_rpc_exclude` from `open_file`,
  `open_http_stream` and `open_process`, which nothing has called since AWU 965
  and 969 deleted their only callers.
- **DoD**: Suite unchanged at 256, and the removal justified by measurement
  rather than by reading the call graph.
- **Done**. 43 lines out, 17 in — the difference is the comments recording why.
  The WIT imports stay: `wit/rad.wit`'s existing functions do not change type
  during the migration, so the host impls remain and only the policy call goes.
- **The probe needed its own positive control, and that is the point.** Each of
  the three functions was made to `panic!` on entry; the full suite then ran
  256/0, never firing one. That alone proves nothing — an unfired probe and a
  probe that cannot fire look identical. The same probe placed on
  `execute_tool_text` failed `security_guard_policy_tests` immediately
  (`AWU970-PROBE: execute_tool_text reached`, then the test's own
  "SHOULD exist" assertion), which is what makes the three silences evidence.
- Two `RasRpcRequest` imports became unused with the blocks, which is the
  compiler confirming the bindings existed for the check and nothing else.

#### AWU 971: `modules/policy`
- **Objective**: The blocklist as a module, config from `kernel.config`.
- **DoD**: `policy.check` answers through a real kernel; the patterns arrive
  from config rather than from a literal.
- **Done**. `modules/policy/{lib.rs,rules.rs,rules/tests.rs}` and
  `tests/policy_module_tests.rs`. 269 passed, up 13: 8 unit + 5 integration.
- **`block_path_patterns` is gone, and the measurement is why.** Three runs:
  both keys configured — blocks, green; `block_command_patterns` alone —
  blocks, green; neither — the test fails. So the path list changed no outcome
  on any end-to-end path. It is not a config-cleanup exception either: the new
  module defines its own schema, and `block_command_patterns` keeps its name so
  a config moved from `extensions` to `modules` keeps working.
- **`policy.check` returns `Ok(allow: false)` to refuse; `Err` means the module
  failed.** The distinction is what lets AWU 972 fail *closed* on a missing or
  crashed policy without a crashed policy being indistinguishable from a
  refusal.
- **The two test layers were shown to cover different things, not the same
  thing twice.** Breaking `refuse` fails 3 of the 8 unit tests and none of the
  integration tests; misspelling `kernel.config` fails 2 integration tests and
  *none* of the unit tests. The unit tests deliberately never call `fetch` —
  the generated dispatch bindings have no host behind them in a native test
  binary — so without that second layer the whole config path would have been
  unverified.

#### AWU 972: `mcp` asks `policy`; the host's tool gate goes
- **Objective**: The one live hook moves into `modules/mcp`, and the host stops
  modelling policy. Added and removed in one AWU so no commit has both live.
- **DoD**: Blocking works end to end through the module path; removing the gate
  makes the blocking tests fail.
- **Done**. `modules/mcp/src/gate.rs`, `tests/policy_gate_tests.rs`.
  272 passed / 0 failed. Clippy clean on native and `wasm32-wasip2`.
- **`verify_tool_call` had three call sites, not one**, and
  `execute_tool_unverified`'s doc comment claimed its callers had already run
  it — so the extension path verified twice. Probing both extension-path sites
  found them dead (`execute_tool` the handle-returning one, and
  `execute_tool_unverified`): 269/0 with panics on entry. Only
  `execute_tool_text` was live, which is the single hook §3.4.3 describes.
- **`kernel.modules` decides whether a policy exists**, rather than matching on
  a dispatch error string. The two answers have to go opposite ways: no policy
  configured allows (opt-in, as the extension was), a policy present but
  unreachable refuses. A crashed policy that read as approval would be a gate
  that vanishes exactly when something is wrong.
- **`security_hook_tests` had no `mcp` module, and that was hiding something.**
  The extension rejected `write` at the host before anything looked for a
  provider, so the test asserted a refusal for a tool that did not exist in its
  own config. The gate now sits inside the provider, so the tool has to be real
  — `mcp` was added and the test means more than it did.
- **A test in the new file passed for the wrong reason first.**
  `RAD_TEST_PORT` was set after the component was instantiated, leaving `mcp`
  in real-MCP mode where the synthetic `execute` does not exist — and
  `a_blocked_command_never_reaches_proc_spawn` passed anyway, because the gate
  refuses above the point where that matters. Caught by the two allow-path
  tests failing; the env var now moves before `load`. Both allow-path tests
  exist precisely so a refusal cannot be confused with a tool that never works.
- **`tests/multi_extension_tests.rs` is deleted, not ported.** Its subject was
  `verify_rpc_exclude`'s fan-out over two `security-guard` instances, and one
  method routes to exactly one module, so "two policies" is not expressible.
  Confirmed it was really testing that mechanism before deleting it: with the
  host gate gone it failed at `blocked.txt should NOT exist`, meaning the
  extensions had been the enforcing party. `multi_extension_isolated_roles_tests`
  keeps the part that outlived it.
- Removing `gate::check` fails exactly the three blocking tests and none of the
  opt-in or allow-path ones.

#### AWU 973: Delete `ext/security-guard` and the verification machinery
- **Objective**: The extension, the host's fan-out, and the WIT surface behind
  them.
- **DoD**: 1 extension + 5 modules, suite green, both clippy targets clean.
- **Done**. Gone: the crate and its workspace member; `verify_rpc_exclude` and
  `verify_with` (`orchestrator/runner/events.rs`); `WasmRuntime::verify_rpc`
  and its `security_guard` binding field; the `host_rpc` call site — the last
  live one, and always an approval, since none of `rad-orchestrator`'s 13
  commands hit a policy branch; the loader's `"security"` linker and
  instantiation branches; `bindings::rad_security_guard`; the
  `delegate_extension_imports!` `rpc_only` arm, whose only consumer it was;
  and `world rad-security-guard` plus **both** `export verify-rpc` declarations
  in `wit/rad.wit` — the one on `rad-extension` too, since nothing could invoke
  it once the host's caller was gone.
- 269 passed / 0 failed, down exactly 3: the `verify_rpc` tests in
  `src/wasm/tests.rs`, whose subjects had already moved in AWU 971 and 972.
  `test_verify_rpc_blocked_file` has no successor on purpose — it drove
  `block_path_patterns`, and it was the only thing still executing that branch.
- **The templates are part of the contract, not decoration.** All three WIT
  copies (`wit/`, `templates/rust/`, `templates/go/`) changed together — the
  sync gate in `build_all.sh` exists because they have drifted before — and
  `templates/rust/src/lib.rs` lost its `verify_rpc`, which would otherwise
  teach a new author to implement a hook the host cannot call. Verified by
  building `rad-extension-template` against the new WIT.
- `tests/security_guard_policy_tests.rs` is now `tests/policy_optin_tests.rs`;
  a test file named after a deleted component is exactly the stale reference
  this repo keeps catching late.
- **The installed config carries three stale entries.** `~/.rad/config.json`
  still lists `security-guard`, `mcp-tool-provider` and `llm-connector`, the
  last two deleted in stages 5 and 6. Config cleanup stays deferred by
  decision, so nothing on the machine was changed here. **Corrected in AWU 976
  below** — the failure mode recorded at the time was probed against
  `~/.rad/wasm/`, and that is not the path this config uses.

#### AWU 974: §3.4.4 — write down what is not defended
- **Objective**: The sentence `ARCHITECTURE.md` §1.3 was making about
  prompt-injection, and every doc still describing a hook that no longer fires.
- **DoD**: No document describes `verify-rpc` as live, and the limit §3.4.4
  names is stated where a reader meets the claim rather than in a design doc.
- **Done**. `ARCHITECTURE.md` (diagram, §1.1's gateway checks, §1.3's entry 2,
  the config example), `CONFIG.md`, `EXTENSIONS.md`, `src/kernel/net.rs`,
  `src/kernel/proc.rs`, `ARCHITECTURE-NEXT.md` §9.3/§9.4.
- **The overclaim is now stated as a limit, next to the thing it limits.** §1.3
  carries a note saying plainly that once a tool call reaches an MCP server rad
  constrains nothing, that the effective defence is which servers a user
  registers, and — the part §3.4.4 asks for — that **no component here prevents
  prompt-injection damage** and earlier revisions said otherwise.
- **The two kernel headers said the opposite of the decision.**
  `net.rs` described the missing check as "the *second* occurrence of that gap"
  belonging to `policy` at stage 7, which reads as "a check will be added
  here". It will not. Both headers now carry §3.4.2's argument — a syscall gate
  defends nothing — plus the reason it is safe: no model-chosen text reaches
  `argv` or a URL, except `mcp`'s testmode `bash -c`, which the gate sits above
  and `tests/policy_gate_tests.rs` proves it sits above.
- **`EXTENSIONS.md` was teaching new authors to implement `verify_rpc`.** The
  skeleton, the world listing, the bullet, and the diagram node are gone, with
  a note pointing at the module world instead.
- **§9.4's invariant needed narrowing a third time.** Removing
  `export verify-rpc` from `rad-extension` is not a type change: a component
  carrying an export the world no longer requires still instantiates, so the
  breakage runs the opposite way from the one the invariant guards against.
  Recorded there with the evidence, not left as a silent exception.

#### AWU 975: `modules/mcp`'s unsafe `Send` claim
- **Objective**: CODING.md §4 prohibits `unsafe` outright, and `client.rs`
  carried one. Not a preference — a violation.
- **Done**. `SERVERS` becomes a `thread_local` `RefCell`, the shape
  `modules/llm-openai/src/session.rs` already used. `modules/` now contains no
  `unsafe` at all.
- **The claim was never needed.** It existed because `Mutex<T>` requires `T:
  Send` and `Process`/`ByteStream` are guest resource handles. A module's store
  is entered by one caller at a time by construction, so there was no second
  thread for `Send` to be about; the `Mutex` was buying nothing and charging an
  `unsafe` for it.
- `TOOL_MAPPING` and `TOOLS_CACHE` stay `static Mutex`: they hold plain data,
  need no claim, and changing them would be churn.
- **`init_servers` had to be restructured, not just retyped.** It held the lock
  across the spawn loop, which reaches back into this module. Under a `Mutex`
  that was a latent deadlock; under a `RefCell` it would be a panic. The borrow
  is now scoped to the liveness probe and to the final store.
- Verified by neutering `server_names`: 5 of `mcp_module_tests`' 6 fail, so the
  suite does reach the converted path against a real server rather than only
  through `testmode`.
- **Still outstanding, and out of scope here**: `ext/rad-orchestrator` has two
  `unsafe` blocks (`orchestrator/reasoning.rs`). That extension goes in stage 8.

#### AWU 976: the deferred config cleanup, and a correction
- **Objective**: Bring `~/.rad/config.json` to the current architecture. Asked
  for explicitly, which is what lifted the deferral.
- **The AWU 973 note was wrong about which entry breaks, and how.** It probed
  `~/.rad/wasm/security_guard.wasm` and reported
  `no function export 'on-event' found`. The config does not point there — every
  entry names `target/wasm32-wasip2/debug/`, where that component had already
  been removed. Re-probed at the paths the config actually declares:
  - `security-guard` — **file absent, skipped silently.** Harmless.
  - `mcp-tool-provider` — **loads.** Role `tool-provider` still has a world, so
    the deleted extension's binary was still serving this machine's tools. Not
    a failure, which is worse: it was working, from a crate that no longer
    exists in the tree.
  - `llm-connector` — **fails**: `component imports instance
    `radcomp:connector/types`, but a matching implementation was not found in
    the linker`. Its world went in AWU 969, so this config has been failing to
    boot since **stage 6**, not stage 7.
- **What changed.** The three extensions are gone, leaving `rad-orchestrator`.
  `mcp`, `llm-openai` and `policy` were added to `modules`. The MCP server
  definitions (`core-utilities`, `web-access`) were carried across from the
  deleted extension's `config` rather than retyped. `policy` gets an empty
  config because `security-guard` had one — it blocked nothing, and inventing
  patterns would be a behaviour change smuggled in as cleanup.
- **Verified by booting it**, not by reading it: `load_config` +
  `Orchestrator::new` against the real file reports all five modules loaded.
- The stale copies in `~/.rad/wasm/` are untouched and unused by this config.

### 🚧 In progress: stage 9 (`dag` / `ui-repl` → modules)

Decision **(A)**: the `dag` module owns the graph and persists it itself, rather
than being a window onto storage the host keeps.

**The crash risk was weighed down, not waved away.** The first framing made
"prove the conversation survives a trap" a gate on the whole stage. Measured:
`save_session` already runs after every completed task, so the host's exposure
was one in-flight turn — and today an extension crash mid-turn already costs
that turn. The module saves on *every* mutation, which is strictly better, and
the gate came off.

- [x] AWU 985: `modules/dag` — the graph and its persistence
- [x] AWU 986: route the host's readers and writers through it
- [x] AWU 987: the terminal half — `modules/ui`
- [x] AWU 988: rollback/reset through the module, and dedup the graph type
- [x] AWU 989: one read path, one write path — every host reader and writer of
      the conversation now goes through `Orchestrator::conversation()` or the
      module
- [ ] AWU 990: delete `Orchestrator::dag` and make the module required
      (25 `Orchestrator::new` sites, 9 `DagSubsystemImpl` sites)

#### AWU 988: two divergences AWU 986 shipped
- **Found while sizing the next unit**, not by a failing test: `reset_session`
  and `rollback` both write the host's `Arc` *directly*, and AWU 986's bridge
  tests only ever exercised create/set/get. 352 passed / 0 failed.
- **Rollback was silently undone one turn later.** The pointer moved in the
  host's cache; the module kept pointing at the old tip, so the next
  `create_node` parented off *that* and the refresh copied the result back over
  the cache. The failure would have surfaced a turn after the rollback, looking
  like nothing to do with it. Fixed with `dag.set_current`.
- **A reset left the module holding the old conversation**, which the next
  mutation copied straight back. Fixed by opening the new session on the module.
- **The first fix for that was worse than the bug.** A `dag.reset` that cleared
  the graph in place *saved through the still-open handle*, overwriting the
  session that had just been archived. It passed the reset test — clearing does
  clear — and only failed once a test asked which *file* the module was writing
  to afterwards. `dag.open(new_id)` does both halves correctly, so `dag.reset`
  is gone from the host, the module and the store, with the reason recorded on
  `store::open` where someone would go looking for one.
- Each fix was controlled separately: removing `dag.set_current` fails only the
  rollback test, removing `dag.open` fails only the two reset tests.
- **The pattern is the same one three times now** — AWU 986's absolute path,
  AWU 987's unloaded module, and this. A bridge is only as good as the paths
  the tests take across it, and "the operations I thought of" is not the same
  set as "the operations that exist".

#### AWU 989: one read path, one write path
- **Objective**: Stop the host touching the conversation directly, so that
  deleting its copy is a change to one function rather than to every reader.
  348 passed / 0 failed.
- **`Orchestrator::conversation()`** is now the single read path: it asks the
  module and falls back to the cache. `command/handlers.rs` (×2),
  `command/compact.rs` and `rollback` were locking the `Arc` directly, which was
  correct only while the host owned the graph.
- **`/compact`'s merge was a write nobody had noticed.** It called
  `merge_nodes` on the host's copy, which the next refresh would have undone —
  the same shape as AWU 988's rollback, a fourth instance of the same class. It
  goes through `dag.merge_nodes` now.
- Two tests added for paths that had none: the merge, and that
  `conversation()` returns the module's graph rather than the cache — asserted
  against a graph written *straight to the module*, so a cache that happened to
  agree could not pass it.
- Verified by making `conversation()` read the cache only: exactly those two
  fail, the other eight stay green.
- **A test wanted `try_dag_module`, which is `pub(crate)`.** It drives the
  kernel directly instead. Widening visibility to suit a test is the wrong
  direction — the same call AWU 986 made when it put the drain's tests in a
  companion file rather than exposing the loop.
- **Left for AWU 990**, which is the churn: `Orchestrator::dag` and
  `DagSubsystemImpl::dag` still exist as the fallback, and 25 construction sites
  plus 9 subsystem sites depend on them. Removing them is what makes the module
  required and finishes §9.3's "the Core that remains is the kernel".

#### AWU 988 (second half): the module shares the host's graph
- **`modules/dag` no longer carries a copy of `Dag`.** It depends on
  `rad-models` and uses `rad_models::Dag`, the same type the host uses.
  346 passed / 0 failed.
- **AWU 985 copied it for no reason.** `rad-models` depends on serde and nothing
  else, and `ext/rad-orchestrator` had already been building against it for
  `wasm32-wasip2` for stages — so a module could always have simply used it.
  171 lines of graph and 107 of duplicated tests are gone; the six graph tests
  now run once, in the host, instead of twice against two copies.
- **Two copies of the one structure a session cannot lose is two things to keep
  in step**, and nothing was keeping them: `dag.get`'s shape agreeing with the
  host's was a *test result* rather than a fact about the code. It is now true
  by construction.
- **The first control for that did not discriminate.** Adding a field to `Dag`
  left the module building fine, because it only ever constructs one through
  `Dag::new()`. Renaming `create_node` — a method the module actually calls —
  fails its build with 2 errors, which is what shows the type is genuinely
  shared.
- Still duplicated, and next: `models/src/dag.rs` remains the host's
  implementation *and* the module's, which is correct. What is still doubled is
  the *graph state* — the host's `Arc` cache beside the module's copy — and that
  is what the last unit of stage 9 removes.

#### AWU 987: `modules/ui`
- **Objective**: The terminal's output half as a module. 349 passed / 0 failed.
- **Done**. `modules/ui/{lib.rs,screen.rs,screen/tests.rs}`, `src/terminal.rs`
  routed through it, and `tests/ui_module_tests.rs` for the seam.
- **The whole state machine moved, not just the printing.** `write_log` defers
  while a response is streaming and flushes when it stops, so a host that kept
  the state while a module printed the tokens would have two halves of one
  decision — the divergence AWU 986 spent its length avoiding.
- **Input stays in the host.** Reading a line blocks, and suspending only the
  caller is what §3.6.1's async is for, which stage 8 deferred. `ui-repl` is
  therefore half a module for now, and the REPL loop is the other half.
- **`write_raw` is deleted rather than routed.** It was a fourth reader of the
  state, and with a module loaded the host's copy freezes at `Idle` —
  `set_state` delegates and returns before touching it — so raw bytes would
  have printed straight through a streamed response. It had no callers at all:
  `route_event_to_terminal` stopped feeding it when it became a no-op. A latent
  trap rather than a live bug, and the reason is recorded on the struct so the
  next person adds a `ui` method instead of a fifth reader.
- **`get_terminal()` is a process-wide singleton**, so the tests serialise on a
  mutex — `attach_kernel` replaces what it points at, and parallel tests would
  fight over it. The same reason `llm_command_tests.rs` serialises.
- Verified by neutering the host's routing: three of the five tests fail and the
  two fallback ones stay green, so each is about something different.
- **Reviewed before finishing, and two things came out of it.** Clippy was
  failing with four `needless_pass_by_value` errors on both targets — the same
  lint `agent-loop` and `dag` each hit. And the module had no consumer at all:
  no config entry, no test, nothing loading it. That is exactly the state AWU
  984 called out as the thing that rots, one step short of the bridge.
- `CONFIG.md`'s module example was three behind — `dag`, `agent-loop` and `ui`
  are now documented, and the example is checked to still parse as JSON.

#### AWU 986: the host writes through the module
- **Objective**: One source of truth. 337 passed / 0 failed.
- **Done**. `DagSubsystemImpl`'s five operations go to the module when one is
  loaded; `kernel.dag` asks it too; `Orchestrator::new` opens the session on it.
- **The host's `Arc` is demoted to a cache, not kept as a second owner.** Every
  mutation refreshes it from the module's reply. That is what lets the readers
  still holding it directly — `src/main.rs`'s auto-save, `command/tree.rs`,
  `command/compact.rs` — keep working untouched. They move in AWU 988 and the
  field goes with them. Dual *ownership* was rejected: two writable copies
  diverge, and nothing on either side can see it.
- **`kernel.dag` routes through `call`, not `deliver`.** `handle_kernel` runs
  before `call` pushes anything onto the stack, so reaching `deliver` directly
  would let a module re-enter itself through the kernel with the cycle check
  looking on.
- **A real bug, found only by reading the way the host reads.** The module was
  writing to a *host-absolute* workspace path. The kernel preopens the workspace
  as the guest's `.`, so WASI resolved `/tmp/ws/.rad/...` **under** the preopen:
  the file landed at `<workspace>/tmp/ws/.rad/sessions/…`. Every module-side test
  passed, because writes and reads went through the same mangled path and agreed
  with each other. It surfaced the moment a test read with
  `rad::session::load_session`. `dag.open` now takes only a session id and the
  module resolves relative to `.`.
- **Self-consistency is not verification.** That is the second time this stage:
  a test can confirm a component agrees with itself while the thing it has to
  agree with is somewhere else entirely. The cross-check now lives in
  `dag_module_bridge_tests`, and `dag_module_tests` points at it.
- **A second self-inflicted one, caught by running the whole suite.** The module
  writes relative to the kernel's workspace, and the test kernels used the
  default — the repo root — so every test shared one session file and collided
  in parallel. Each test kernel now has its own workspace. It passed in
  isolation and failed together, which is the only way that shows up.

#### AWU 985: `modules/dag`
- **Objective**: The graph as a module, owning its own storage.
  332 passed / 0 failed.
- **Done**. `modules/dag/{lib.rs,graph.rs,store.rs}`, `src/dag/tests.rs` copied
  across unchanged as `graph/tests.rs`, plus `store/tests.rs` and
  `tests/dag_module_tests.rs`.
  **Corrected below (AWU 988):** this entry originally said the tests *moved*.
  They were copied — the originals stayed, because `models/src/dag.rs` was still
  the host's live implementation. The copy itself was unnecessary.
- **The graph is copied operation for operation.** It is the one piece of state
  a session cannot lose, so the port is mechanical — including the two
  conditional `current_node_id` rules in `merge_nodes` and `delete_node` that
  each took a bug to find.
- **One method did not come across.** `set_node_semantic_references` has no
  production caller in the host either: `tests/repo_map_tests.rs` is the only
  thing that calls it, so the host's copy is kept alive by a test alone. The
  *field* stays, because it is in the on-disk shape. Worth deleting from
  `models/src/dag.rs` when that file goes.
- **Saving is part of mutating, not a separate call.** The only way to change
  the graph from outside `store.rs` is through `mutate`, which saves. A caller
  cannot forget.
- **The file is byte-compatible with `src/session.rs`** — same path, same
  shape — which is what lets AWU 986 swap the producer with no migration step.
- Verified by commenting out the save: three `store` tests fail and so does
  `a_second_kernel_attaching_to_the_session_sees_the_same_conversation`, which
  is the property decision (A) rests on.

### 📌 State at the end of stage 7

- **1 extension** (`rad-orchestrator`) + **5 modules** (`context`, `skills`,
  `mcp`, `llm-openai`, `policy`), plus four `ship = false` test fixtures
  (`echo`, `relay`, `spawn`, `net`).
- 269 passed / 0 failed. Clippy clean on native and `wasm32-wasip2`.
- **Policy is one hook, inside `modules/mcp`.** The host models no policy at
  all: no `verify_rpc_exclude`, no `verify_rpc`, no `security` role, no
  `verify-rpc` anywhere in the WIT.
- `modules/` contains no `unsafe`.

Carried forward, none of it blocking:

- **The known flake below is still unreproduced.**
- **Cross-thread lock ordering is unaddressed** (AWU 968), and as of AWU 977 it
  is **demonstrated rather than theoretical** — see
  `tests/kernel_lock_order_tests.rs`. The old note said no module pair could do
  it; that was true only because no second forwarding fixture existed.
- **`ext/rad-orchestrator` has two `unsafe` blocks**
  (`orchestrator/reasoning.rs`). That extension goes in stage 8.
- **`report_tool_inventory`** (`src/orchestrator/runner/runtimes.rs`) is still
  uncovered.
- **`runtimes/tests.rs`'s 53-line test function** is still over CODING.md §2's
  40-line rule, still left alone deliberately.
- **`execute_tool` and `execute_tool_unverified`** (`src/wasm/imports_tool.rs`)
  are dead — probed in AWU 972 — and stay only because `wit/rad.wit` declares
  `execute-tool`. They go with the extension world in stage 8.

### 📌 State at the end of stage 6

- **2 extensions** (`rad-orchestrator`, `security-guard`) + **4 modules**
  (`context`, `skills`, `mcp`, `llm-openai`), plus four `ship = false` test
  fixtures (`echo`, `relay`, `spawn`, `net`).
- 256 passed / 0 failed. Clippy clean on native and `wasm32-wasip2`.
- All three syscalls are implemented; the surface is closed at three (§3.1).
- Everything through AWU 969 is committed and pushed to `main`.

Carried forward, none of it blocking:

- **The known flake below is still unreproduced.**
- **Cross-thread lock ordering is unaddressed** (AWU 968). Two threads each
  holding one module's lock and calling into the other's would deadlock below
  the cycle check. No module pair can do it today; §3.6.1's scheduler is the
  answer, at stage 8.
- **`modules/mcp` carries an `unsafe impl Send`** that a `thread_local` would
  remove, the way `modules/llm-openai/src/session.rs` does. CODING.md §4
  prohibits `unsafe` outright, so this is a real violation, not a preference.
- **`report_tool_inventory`** (`src/orchestrator/runner/runtimes.rs`) is
  uncovered — it only prints, and nothing asserts on its `[OK]`/`[FAILED]`
  wording, which Phase 61 was about.
- **`runtimes/tests.rs`'s 53-line test function** is over CODING.md §2's 40-line
  rule, left alone deliberately: splitting one scenario is the fragmentation the
  same rule warns against.
- **Config-file cleanup stays deferred** until the migration finishes.

### 🚧 In progress: stage 8 (`rad-orchestrator` → `agent-loop`)

Approach **(C)**, chosen before any code: drive `drain_posts` in production and
route `agent-loop`'s events through `post`, but leave §3.6.1's async wasmtime
and per-module Stores for later. What removes the lock-order hazard is the
`post` path, not the async runtime, and §3.6.9 admits per-Store memory is an
unmeasured unknown — pairing that with a 2,061-line port would make a failure
impossible to attribute.

- [x] AWU 977: Demonstrate the lock-order deadlock
- [x] AWU 978: Drive `drain_posts` in production
- [ ] ~~AWU 979: `llm-openai` from pull to push~~ — **premature, see below**
- [x] AWU 979 (revised): `modules/agent-loop` — the event intake
- [x] AWU 980: `llm.rs` — the pure core (DAG walk, orphan filter, system prompt)
- [x] AWU 981: verify AWU 980's port against the extension (differential)
- [x] AWU 982: `kernel.dag` — decision (A), scaffolding with a stage-9 expiry
- [ ] AWU 983: `orchestrator.rs` — the event state machine (282)
- [ ] AWU 982: `runner/done.rs` + `inline_tool_calls.rs` (565)
- [x] AWU 983: `digest`, and the port gap the differential had missed
- [x] AWU 984: route the extension's message assembly through the module
- [ ] **Blocked on stage 9** — see AWU 983. `orchestrator.rs`, `runner/done.rs`
      and `reasoning.rs` cannot move until the DAG and the terminal do.
- [ ] AWU final: delete the extension, the old world, the old RPC surface, and
      `models/`'s conversion macros

#### AWU 984: the module serves the extension
- **Objective**: Stage 8 pauses here (the fork AWU 983 recorded), but not with
  `agent-loop` as an unused parallel implementation. What was ported is now
  what runs. 315 passed / 0 failed.
- **Done**. `ext/rad-orchestrator/src/llm.rs`'s `load_messages_from_dag` asks
  `agent-loop` for the raw message list and falls back to its own copy when no
  module answers — the same bridge stages 3-7 used, and no new kernel surface:
  `agent.messages` already reads the conversation through `kernel.dag`.
- **Why not simply stop.** A parallel implementation nothing calls is the thing
  that rots: the extension's copy is what would keep working, and the module's
  would drift until the day it was switched on. Making it load-bearing now means
  every real turn exercises it.
- **The seam is compaction.** The module builds system-with-digest plus the
  filtered conversation; `context-tools` windowing and the second orphan filter
  stay in the extension, because they need `GetActiveLlmProfile` and the turn's
  retry state. Cutting there took no new methods.
- `extension_id` is `"agent"`, not `"agent-loop"`: the host's bridge routes on
  `<extension_id>.<method>`, and the module provides `agent.messages`.
- **The parity test had to change shape or become meaningless.** It compared
  the extension's request against an `agent.messages` call made from the test —
  which, once the extension started asking the module, would have compared the
  module against itself. It now runs the same turn twice, with and without
  `agent-loop` loaded, and compares what reaches the wire. That is a real
  differential of the two implementations and stays one.
- **Verified the module is the one serving**, by marking its system prompt and
  watching the test fail. Without that check, "the requests match" would be
  equally consistent with the bridge never firing and both runs using the
  fallback.

#### AWU 983: `digest`, and a gap the differential had missed
- **Objective**: Move `orchestrator.rs`'s state machine. **It cannot move yet**,
  and measuring why is most of what this AWU produced. 315 passed / 0 failed.
- **The remaining extension's host-RPC inventory, counted rather than guessed**:
  `WriteStdout` 22, DAG writes (`CreateNode` 4, `SetNodeText` 4, `TakeSnapshot`
  1, `CheckoutSnapshot` 1) 10, `GetDag` 5, `CompleteTask` 5,
  `GetActiveLlmProfile` 2, and one each of `ReportTokenUsage`, `GetTools`,
  `GenerateLlmStream`, `FileRead`, `CallExtension`.
- **The terminal and DAG *writes* are stage 9's, not stage 8's.** §9.3 puts
  `dag` and `ui-repl` in stage 9, and `orchestrator.rs` (16 RPCs),
  `runner/done.rs` (16) and `reasoning.rs` (9) are made of them. Moving them now
  means adding a batch of kernel methods that stage 9 then deletes — far more
  scaffolding than `kernel.dag` was. **The dependency runs the other way from
  the plan's ordering, for these three files.** Recorded as a decision for the
  next session, not taken here.
- **What did move**: `digest.rs` and its tests, verbatim but for one import.
  `context_recovery.rs` and `inline_tool_calls.rs` were copied and then put
  back — their consumers are in the blocked files, and a module carrying
  unreachable code is what CODING.md §3 forbids and clippy rejects. A thing
  moves with its consumer.
- **`digest` was already missing from AWU 980's port, and AWU 981 did not
  catch it.** `load_messages_from_dag` appends the digest to the system prompt;
  `agent.messages` did not. The differential passed anyway because its fixture's
  tool call carried `arguments: "{}"` — no `path`, no `command`, so the digest
  was empty and the comparison never exercised it. The fixture now carries a
  real path. **Confirmed in that order**: strengthened fixture fails, digest
  wired, fixture passes.
- The lesson is about the fixture, not the port: a differential is only as good
  as the behaviour its input provokes, and "they agree" over an input that
  reaches neither implementation's interesting path means nothing.

#### AWU 982: `kernel.dag`
- **Objective**: Decision (A) from AWU 980 — host-owned runtime state reaches a
  module as a kernel method. 308 passed / 0 failed.
- **Done**. `KernelShared::dag`, the `kernel.dag` arm beside `kernel.config` and
  `kernel.modules`, wired from `Orchestrator::new`, and `agent.messages` asking
  for it when no DAG is handed in.
- **Scaffolding, and the expiry is written where the field is.** Stage 9 makes
  `dag` a module and both this field and its method go with it. Recorded in the
  code rather than only here, because that is where someone will find it.
- **`kernel.llm-profile` is not in this AWU.** The DAG arrives as an `Arc` that
  `Orchestrator::new` already holds; the config does not — it is a plain
  `Mutex<Config>` inside the orchestrator, so exposing it means either changing
  its ownership or giving the kernel a `Weak<Orchestrator>`. Neither is a
  decision to make in passing, and nothing needs the profile until compaction
  budgeting moves.
- **Read-only on purpose.** A module that could write the DAG would be
  reimplementing snapshots and rollback through a keyhole.
- **The unit tests could not have caught the production wiring.** They attach a
  DAG to a bare kernel by hand, so they pass whether or not `Orchestrator::new`
  ever hands one over. The parity test now asks `agent.messages` *without* an
  explicit DAG, which only works through `kernel.dag`. Verified by deleting the
  wiring line: the parity test fails, the ten module tests stay green.
- Both keeps its explicit-DAG form too, because the differential needs a
  conversation as it was at a past instant rather than as it is now.

#### AWU 981: the differential AWU 980 asked for
- **Objective**: Find out whether the module's copy actually matches the
  extension, rather than only matching its own tests. 305 passed / 0 failed.
- **Done**. `tests/agent_loop_parity_tests.rs`. **They match.**
- **The comparison point is the wire.** The extension's message list is private,
  inside a component, built from host RPCs — unreachable from a test. But every
  turn sends it to the backend, so a mock server that keeps the request body has
  the real answer. The fixture DAG carries a user turn, an assistant turn with a
  tool call, its matching reply, and an **orphan** reply whose call was never
  made.
- **The first run reported a difference that was the test's own doing**, and it
  is worth keeping: the module's list had one extra `assistant` message. The
  DAG was read *after* the turn, by which time the assistant's reply was a node;
  the extension's request had been built before it existed. The server now
  snapshots the DAG at the moment the request arrives. A differential that
  compares two different inputs finds differences that mean nothing, and would
  have sent me looking for a porting bug that was not there.
- **The second failure was the fixture's sanity check**, which expected four
  roles where there are five — the task instruction is itself a user turn. The
  parity assertion had already passed by then.
- **Verified it can detect divergence, twice.** Neutering the module's orphan
  filter fails it, and rewording one word of the system prompt fails it. Without
  those, "they match" would be indistinguishable from "the test compares
  nothing".
- The orphan is asserted absent from the *extension's* list separately, so a
  shared bug — both keeping it, or both dropping everything — cannot pass as
  agreement.

#### AWU 980: `llm.rs`'s pure core
- **Objective**: Move the parts of message assembly that are functions of their
  input. 304 passed / 0 failed.
- **Done**. `modules/agent-loop/src/messages.rs` and its 12 unit tests, plus
  `agent.messages` across dispatch.
- **`llm.rs` did not move whole, and the reason is a decision still open.** Its
  other half asks the host questions — `GetDag`, `GetActiveLlmProfile`,
  `CallExtension` to `context-tools`, `GenerateLlmStream`, `WriteStdout` — and a
  module has no way to ask most of those. `CallExtension` and
  `GenerateLlmStream` already have module answers (`context-tools.optimize`,
  `llm.generate`); **`GetDag` and `GetActiveLlmProfile` do not.**
- **The open question, which decides AWU 981's shape**: how does a module reach
  host-owned runtime state?
  - **(A) kernel methods** — `kernel.dag`, `kernel.llm-profile`, beside the
    existing `kernel.config` and `kernel.modules`. §3.6.7 already registers the
    kernel as a dispatch target, so this is not a new concept. Cost: stage 9
    makes `dag` its own module, so `kernel.dag` is scaffolding with a known
    expiry.
  - **(B) the host pushes state in** with the turn. Keeps the traversal in the
    module but leaves turn orchestration in the host, which is what stage 8
    exists to remove.
  - **(C) pull stage 9's `dag` module forward.** Cleanest end state; the DAG is
    wired into snapshots and rollback, so it is a large reordering.
  - Recommendation: **(A)**, with the expiry recorded rather than discovered.
- `read_rule_file` shrinks: a `FileRead` RPC that decoded a `Vec<u8>` out of
  JSON becomes `std::fs::read_to_string`, because §3.1 puts the filesystem on
  WASI.
- **The base system prompt is copied verbatim.** It is what every transcript so
  far has been produced against; rewording it would be a behaviour change
  wearing a port's clothes.
- Both halves were shown to be load-bearing: neutering the orphan filter fails
  3 tests, removing the walk's `reverse()` fails 1, and they are different tests.
- **Not yet verified against the extension.** Nothing compares the two
  implementations on the same DAG — the extension's copy is still what runs. The
  differential only becomes possible when AWU 981 can drive both, and that is
  the first thing it should do.

#### AWU 979 (revised): `modules/agent-loop` — the event intake
- **Objective**: The module exists, and the transport's events reach it, before
  any decision-making moves onto it.
- **DoD**: A real turn through the transport is visible in the module.
  290 passed / 0 failed.
- **Done**. `modules/agent-loop/{lib.rs,intake.rs,intake/tests.rs}`,
  `tests/agent_loop_tests.rs`, and the relay in
  `src/wasm/rpc_meta_llm_module.rs` now posts each event to `agent-loop` when
  one is loaded.
- **`RawEvent` is copied field for field**, not redesigned. It is the contract
  with `llm-openai`, which is still emitting exactly those bytes; changing the
  shape and the consumer together would leave no way to tell which broke a turn.
- **Both consumers are fed during the migration.** The extension still runs the
  turn off the `RasCoreEvent` bus; the module only accumulates. A doubled event
  costs nothing while that is true, and it means the new path is exercised by
  every real turn long before anything depends on it.
- **Events are posted, never called.** From the relay thread a `call` would put
  a second thread inside a module while the event-loop thread calls out of one —
  AWU 977's deadlock exactly. `post` touches only the queue, and AWU 978's drain
  delivers it on the one thread allowed to hold two module locks.
- **Three test layers, each catching what the others cannot.** The unit tests
  (11) cover the fold; `tests/agent_loop_tests.rs` (5) covers what only exists
  across dispatch — method names resolving, the payload envelope matching what
  the host builds; and the extended `llm_module_e2e_tests` covers the relay
  actually producing posts from a real stream. Verified by neutering
  `post_to_agent`: only the e2e test fails, the other 16 pass.
- `the_module_declares_the_methods_the_host_posts_to` exists because a drift
  between the method names here and in `rpc_meta_llm_module.rs` would make the
  host silently stop posting, with every other test still green.

#### AWU 979 was mis-scoped, and the reason is worth keeping
Planned as "fold the polling thread away so the transport pushes chunks".
Two findings while starting it say otherwise.

- **The polling thread is not part of the hazard.** AWU 977's deadlock needs
  *both* threads to hold two module locks. `modules/llm-openai` calls nobody —
  checked, not assumed: the only `dispatch::call`s in any module are `mcp` →
  `policy` and three `kernel.*` lookups, and a `kernel.*` target takes no module
  lock at all (`handle_kernel` is pure host). So the polling thread holds
  exactly one lock, always. The invariant AWU 978 wrote down is not violated by
  it, and removing it buys no safety.
- **There is no `post` target yet.** `post` resolves through the registry to a
  *module*. The consumer of LLM events is still the `rad-orchestrator`
  extension, reached over the host's `RasCoreEvent` bus. Until `agent-loop` is a
  module there is nothing for the transport to push *to*, so the conversion has
  no destination.

A third finding constrains whatever replaces it: **`llm.next` can block for
~15 seconds.** `Session::pump` loops until it has events or the stream ends, and
each `read` waits `READ_POLL` (100ms) in the kernel, up to `MAX_PENDING` (150).
So it can never be pumped from the event-loop thread — that thread also handles
aborts and drains posts. Either the transport keeps a thread of its own, or
§3.6.1's async lands. Recorded here because "just call it from the loop" is the
obvious idea and it is wrong.

The conversion therefore belongs *inside* the `agent-loop` port, as the wiring
of its event intake, not before it.

#### AWU 977: Demonstrate the lock-order deadlock
- **Objective**: Decide whether `post` is load-bearing or merely tidy, by
  reproducing the hazard rather than reasoning about it.
- **Done**. `modules/pong` (`ship = false`) and
  `tests/kernel_lock_order_tests.rs`. 271 passed / 0 failed.
- **It reproduces, deterministically.** Two threads taking `relay` and `pong`
  in opposite orders both wedge; `deliver` holds a module's lock across the
  nested guest call, and the cycle check cannot see it because its stack is
  thread-local by design. Epoch interruption does not help: both threads are
  blocked in a *host* call on a `Mutex`, and epochs preempt guest code only —
  the same reason `src/kernel/proc.rs` bounds its `wait` by hand.
- **A new fixture was unavoidable.** `relay` was the only module that forwards,
  and one method may be claimed by one module (§3.6.8), so it cannot be loaded
  twice to face itself. `pong` forwards under its own method name and can be
  told to hold its store first — without that lever the interleaving is a race,
  and a deadlock test that only sometimes reproduces cannot be trusted when it
  passes.
- **The control is what makes the result mean anything.** The same pair, the
  same holds, the same harness, but both threads taking the locks in the *same*
  order: they serialise and complete. A harness that reported a timeout
  unconditionally would be indistinguishable from the deadlock test otherwise.
- The test asserts the deadlock *occurs*, so it fails the day the kernel gains
  lock ordering or §3.6.1's scheduler — which is the point at which the `post`
  routing stops being load-bearing and the note here should go.
- **These threads stay wedged for the life of the process**, which is why the
  file is its own test binary with nothing else in it.

#### AWU 978: Drive `drain_posts` in production
- **Objective**: `post` has existed since AWU 955 and queued into a queue
  nothing emptied outside a test. A running rad never delivered one.
- **Done**. `process_event_loop` trades its blocking `recv()` for
  `recv_timeout(20ms)` and drains at two points.
  274 passed / 0 failed.
- **Which thread drains is the whole design decision, not a convenience.**
  Delivering a post takes the target's lock, and a module handling one may call
  onward — two locks, nested. AWU 977 showed two threads doing that in opposite
  orders deadlock. The invariant is now stated in the code: **only the
  event-loop thread ever holds more than one module lock.** Other threads may
  `post` (which touches the queue and nothing else) but must not `call` into a
  module that calls onward. A second draining thread would break it, which is
  why there is not one.
- **Two drain sites, and the second is not a latency trim.** If events arrive
  faster than the tick — which is exactly what an LLM turn is — `recv_timeout`
  returns `Ok` every iteration and the timeout branch never runs. Without the
  drain after the handlers the queue starves for the length of the stream.
  Found by removing each site separately: dropping the tick drain fails two
  tests, dropping the handler drain fails only
  `a_steady_event_stream_does_not_starve_the_post_queue`. Each site has a test
  that is about *it*.
- **The first version of the test poisoned four unrelated ones.** It used `mcp`
  in testmode, which needs `RAD_TEST_PORT` — process-global, and the lib tests
  run in parallel threads, so `rpc_meta_llm_module`'s four endpoint tests
  started failing. `tests/llm_command_tests.rs` carries a `TEST_MUTEX` for
  exactly this. A mutex would have worked; using `modules/spawn`, which reads no
  environment at all and touches a file, needs no shared global.
- Failed deliveries are logged, not propagated: `post` is fire-and-forget by
  definition (§3.6.2), and a task must not die because an event had nowhere to
  go.
- **Still not covered**: posts queued while no task is running. `run_attempt`
  owns the loop, so the queue only moves inside a task. Nothing posts outside
  one today; the kernel owning its own loop is §3.6.7's job.

### 📜 Stage 6 — what was known before it started (historical)

Kept because two entries outlived the stage. The rest is settled.

- ~~**`net-open` is half-built.**~~ Landed in AWU 966, along with the fallible
  reader `KernelStream` needed and the decision on async. All three are recorded
  above.
- **WASI 0.3 is not a prerequisite** (§8): it deleted `wasi:io` and needs
  Wasmtime 43+. Staying on 29 is a deliberate choice, not an oversight.
- **The consumer of LLM events is still an extension.** `rad-orchestrator` reads
  `RasCoreEvent::LlmConnectorEvent` through `src/wasm/bindings_event.rs`, so the
  module has to feed that same bus with byte-identical JSON. §3.6.4's `post`
  shape cannot be used yet: `agent-loop` does not exist until stage 8, and
  nothing drives `drain_posts()` in production — only a test calls it. Stage 6
  is therefore pull-based, and `rpc_meta_llm_connector.rs`'s existing polling
  thread is the loop that stays.
- **`ext/llm-connector` has no test that ever speaks to a server**, the same gap
  `mcp-tool-provider` had in stage 5. `dialect/tests.rs` is its only test file.

### ⚠️ Known flake (unresolved)
One `cargo test --workspace` run during AWU 963 reported `122 passed / 1 failed`;
cargo's fail-fast truncated the run, so 122 is a partial count. It did not
reproduce across five subsequent full runs, and the failing test's name was not
captured. Recorded so a recurrence is recognised rather than investigated from
scratch.

### 💡 Previous AWU Status (stage 5)
- [x] AWU 963: Implement `proc-spawn`, `process`, and `byte-stream` in the kernel
- [x] AWU 964: `modules/mcp` — port the MCP client
- [x] AWU 965: Route tools to the module and delete `ext/mcp-tool-provider`

#### AWU 963: Implement `proc-spawn`, `process`, and `byte-stream`
- **Objective**: The first real syscall. Every host implementation in
  `src/kernel/host.rs` is currently a stub returning 501.
- **Context**: `mcp-bridge` cannot exist without it — an MCP server is a
  long-lived child process spoken to over stdio, and nothing in `std` reaches a
  child from inside Wasm (§3.1.2's rule: WASI where std insulates, a syscall
  where it does not).
- **Shape**: `proc-spawn(argv: list<string>)`, not a command string. The
  extension host takes a string and has to decide whether it needs a shell —
  `src/process.rs` carries a comment about quoted arguments being mangled by the
  direct-exec fast path, a bug that corrupted every tool result until it was
  found. An argv list has no such question to answer.
- **DoD**: A module spawns a process, writes to its stdin, reads its stdout, and
  waits for it. The process is reaped, and killed if the module goes away.
- **Done**. `src/kernel/proc.rs`, driven from `modules/spawn` (a test fixture,
  `ship = false`) through `tests/kernel_proc_tests.rs`.
- **`read` waits at most 100ms and then reports empty.** It must be bounded:
  epoch interruption preempts *guest* code only, so a host call blocked in
  `recv()` cannot be interrupted at all, and a module reading from a live but
  silent child would hang the kernel with the deadline machinery looking on.
  The extension host does block there. `wait` is bounded for the same reason —
  25s against a 30s deadline — and returns 504 meaning "call again".
- **The first version of the kill-on-drop test was vacuous.** It used `cat`,
  which exits by itself when its stdin pipe closes as the module's resources
  drop, so it passed with the kill removed. Rewritten with `sleep`, which has
  to be killed to go away; verified to fail without the kill.
- `spawn_argv` shares `finish_spawn` with `spawn_bash_process` rather than
  duplicating it: the process group is what `kill_group` and `ProcessManager`'s
  `Drop` act on, and a second spawner that forgot it would leak children.

#### AWU 964: `modules/mcp` — port the MCP client
- **Objective**: JSON-RPC over an MCP server's stdio, as a module.
- **Context**: 770 lines across 6 files, of which the host-facing part is 5
  `open_process` calls plus one `FileRead` and one `WriteStdout`. Config comes
  from `kernel.config` rather than `GetExtensionConfig`.
- **DoD**: Ported tests pass; `mcp.tools.list` / `mcp.tools.call` answer.
- **Done**. Measured: 770 lines become 732 (672 excluding the new unit tests);
  ignoring comments and blanks, 645 becomes 530. Less of a saving than the
  deletions suggest, because the module carries heavier documentation and the
  `RAD_TEST_PORT` scaffolding grew when it was pulled into its own file.
- **`mcp_config.rs` is the whole saving.** 175 lines existed to answer "what is
  my config?" — six candidate paths, a hand-written JSON comment stripper, and
  three successive guesses at what shape `FileRead` had returned the file in.
  `kernel.config` answers it in one call.
- `conv.rs` goes entirely (no host RPC), as does the `echo -n '<result>'`
  round trip with its quote escaping.
- **The extension had no test that ever spoke to a server.**
  `tests/mcp_module_tests.rs` spawns a real one — a small script over real pipes
  — so the syscall, the resources, the handshake, and both JSON-RPC calls are
  exercised together. The first version of its failure test was unreachable
  (it called a tool the mapping did not contain); fixed to fail through
  `isError` on a tool that exists.
- **`RAD_TEST_PORT` synthetic tools are carried over**, in `testmode.rs` rather
  than threaded through the handlers. Seven files in `tests/` drive the agent
  loop against those three tool names; removing them is its own change, not
  part of a port.

#### AWU 965: Route tools to the module and delete `ext/mcp-tool-provider`
- **Context**: Unlike stage 4 this needs no host change — `src/kernel/tools.rs`
  already aggregates any module offering `<module>.tools.list`.
- **DoD**: Tools resolve with no tool-provider extension configured; extension
  and its tests deleted; suite green.
- **Done**. Stage 5 complete: 3 extensions, 3 modules.
- **Three host bugs surfaced, all of which would have shipped silently.** Each
  was found by an existing test, which is the argument for migrating the seven
  files rather than writing new ones:
  - **`proc-spawn` ignored the workspace.** The extension host spawns with
    `cwd = workspace`; the kernel inherited rad's own working directory, so a
    module's children resolved relative paths in the wrong place. Both the
    spawn cwd and the WASI preopen are now rooted at `core.workspace`.
  - **`proc-spawn` had no HITL gate.** `src/wasm/imports_process.rs` asks before
    spawning; the kernel did not, so moving a tool provider to a module removed
    human approval from every process it starts. `tests/hitl_tests.rs` caught
    it. The rejection is worded exactly as the extension words it, because the
    DAG and the model both carry that phrase.
  - **`Orchestrator::new` did not boot the kernel.** `main` did, so anything
    else built from a `Config` — every integration test — declared modules and
    silently got none. Boot moved into `Orchestrator::new`, which is also what
    made migrating the seven test files a one-line change each.
- **Still missing: the security-guard check on `proc-spawn`.** The extension
  host runs `verify_rpc_exclude` before spawning; the kernel does not, because
  it holds no orchestrator handle. Exposure today is limited — a module's argv
  comes from its config, not from the model — but the `execute` path under
  `RAD_TEST_PORT` shows the shape of the gap. This belongs to the `policy`
  module (§3.4.3, stage 7); recorded here so it is not rediscovered as a
  surprise.
- The regex-based edit of the seven test files removed neighbouring extension
  entries on the first attempt (non-greedy matching across whole blocks, the
  same failure as the timeout-assertion edit in stage 3). Reverted and redone by
  brace matching, asserting one `name:` per removed block.

### 💡 Previous AWU Status (stage 4)
- [✅] AWU 959: `modules/skills` — port discovery and execution (Result: Success — 10 ported tests plus 3 against a real skill tree; the SDK gained an `infallible` adapter on its third occurrence)
- [x] AWU 960: Consult modules from `GetTools` and `execute_tool`
- [x] AWU 961: Delete `ext/skill-tool-provider`
- [x] AWU 962: Collapse per-skill tools into one `skill` tool plus an index (§4.5 ③)

#### AWU 959: `modules/skills` — port discovery and execution
- **Objective**: SKILL.md discovery and inline execution, as a module.
- **Context**: Two changes are inherent to the port rather than chosen. The
  extension returned skill bodies by shelling out to
  `open_process("echo -n '...'")`, because its WIT return type was an
  `execution-handle` — which also meant a module that only reads files needed
  bash execution permission. `handle()` returns a string, so both the hack and
  the permission requirement vanish. And `mode: subagent` goes: it only ever
  returned "not implemented", and subagents were dropped in §1.2.
  Tool-per-skill parity is kept here on purpose; changing what the model sees is
  AWU 962's job, so a behaviour difference cannot be mistaken for a porting bug.
- **Result**: All 10 of the extension's tests ported, plus 3 integration tests
  against real files on disk — necessary because discovery moved from
  `ListDir`/`FileRead` host RPCs to `std::fs`, so what the module can reach is
  now the host's preopens rather than a permission mask.
  Both inherent changes landed: the `echo -n` shell-out is gone (and with it the
  bash permission a Markdown reader never should have needed), and `mode:
  subagent` no longer blocks execution — a legacy line is ignored rather than
  rejected, so existing `SKILL.md` files keep working.
  **The SDK gained `rad_sdk::infallible`.** `list` has no honest failure case,
  and this was the third module to hit clippy's `unnecessary_wraps` — the point
  AWU 956 named as the trigger for fixing the SDK instead of the module.
  Handlers are now `expr` rather than `path`, so an adapter can wrap one. The
  invented "target must not be empty" check added to `relay` in AWU 953 as a
  workaround is retracted.
  Two test hazards were caught and fixed before they shipped: the fixture wiped
  all of `.agents/skills` on drop, which would delete a developer's real skills,
  and the tests raced each other over one shared directory. It now removes only
  what it created and serialises through `TEST_MUTEX`.

#### AWU 960: Consult modules from `GetTools` and `execute_tool`
- **Objective**: The bridge for tool providers, which differs from stage 3's.
- **Context**: `CallExtension` names one role; tools are inherently plural and
  aggregated across providers (`src/wasm/rpc_meta.rs` for listing,
  `src/wasm/imports_tool.rs` for execution). The registry maps a method to
  exactly one module, so tool-providing modules cannot all claim `tools.list`.
  Each provides `<module>.tools.list` / `<module>.tools.call` instead, and the
  host iterates modules rather than resolving a single method — the same
  `role.method` concatenation rule stage 3 established.
- **DoD**: Skills appear in `/tools` and execute, with the extension disabled.
- **Done**. `src/kernel/tools.rs` aggregates `<module>.tools.list` and routes
  `<module>.tools.call` by tool name; `GetTools` consults it first
  (`src/wasm/rpc_meta.rs`) and a new `execute-tool-text` import carries the
  result (`src/wasm/imports_tool.rs`). `tests/skills_module_e2e_tests.rs` runs a
  skill through a real Orchestrator with **no tool-provider extension
  configured**, and fails with "no registered tool provider handled it" when the
  module is disabled — the negative control stage 3 lacked.
- **Two things this turned up**, both invisible to the tests that existed:
  - `execute-tool` returns an `execution-handle`, so a provider holding a plain
    answer had to manufacture a process to carry it — the reason
    `skill-tool-provider` shelled out to `echo -n`. Added `execute-tool-text`
    rather than changing the existing function: §2.2's experiment says a new
    import is safe, and this confirmed it a third time (all 8 components loaded
    unrebuilt). §9.4's invariant is narrowed to match the evidence.
  - The kernel loader preopened only `.` and passed no environment, so
    `~/.rad/skills` was unreachable from `modules/skills` — a regression from
    AWU 959 that every test missed by using `.agents/skills` alone. Loader now
    matches `src/wasm/loader.rs`; `user_global_skills_under_home_are_discoverable`
    fails without it.

#### AWU 961: Delete `ext/skill-tool-provider`
- **DoD**: Extension gone, suite green, skills still work.
- **Done**. Crate, both test files, and the workspace member are gone.
- CONFIG.md and ARCHITECTURE.md described it as a live extension, and CONFIG.md
  still listed `context-tools` as one too — the config reference was a stage
  behind. Both now document the `modules` array instead, including that the
  `allow_bash` grant disappeared with the `echo -n` hack.
- CI's wasm package list is now derived from `cargo metadata` rather than
  written by hand. It had drifted three times; a package that is never built
  looks exactly like one that passes.

#### AWU 962: One `skill` tool plus an index
- **Objective**: §4.5 ③. Deliberately separate from the port.
- **Context**: One tool schema averages 468 characters (§4.4), so a tool per
  skill costs context linearly. A single `skill(name, args)` whose description
  lists what is available is roughly a quarter of that at ten skills, and the
  body still loads only on invocation.
- **DoD**: Ten skills cost one schema; invocation still resolves by name.
- **Done**. `ten_skills_cost_one_schema` measures it: 935 bytes for ten, against
  roughly 4,700 for the extension's ten schemas. The budget in that test is a
  slope check, not a golden file — it fails if per-skill schemas come back, not
  when the description text is edited.
- The name is constrained by a JSON-Schema `enum` as well as listed in the
  description. The index is prose a model may paraphrase; the enum is what it
  can actually emit.
- Zero skills produces zero tools rather than a tool with an empty index, which
  would spend schema on an offer that cannot be taken.
- User-visible, so CONFIG.md §2.5 says so plainly, and `mode` is now documented
  as removed rather than reserved (a `SKILL.md` still carrying the line runs).

### 💡 Previous AWU Status (stage 3)
- [✅] AWU 956: `modules/context` — port the optimize logic (Result: Success — 13 ported tests pass, `windowing.rs` logic byte-identical to the extension's)
- [✅] AWU 957: Route `CallExtension` to a kernel module when one provides the method (Result: Success — proven by disabling the extension and watching compaction keep working)
- [✅] AWU 958: Delete `ext/context-tools` and its WIT (Result: Success — stage 3 complete; compaction runs entirely through the module)

### 📝 AWU Details

#### AWU 956: `modules/context` — port the optimize logic
- **Objective**: The windowing logic, running as a module.
- **Scope**: `modules/context/` (new).
- **Context**: Logic is copied rather than moved. The old extension stays live
  and serving until AWU 957 switches the caller over, because §9.4 requires rad
  to work at the end of *this* AWU, not only at the end of the stage. The
  duplication is deliberate and lasts two AWUs.
- **Result**: All 13 of the extension's tests ported and passing, and a diff of
  `windowing.rs` against the original shows the logic is byte-identical — the
  only change is one `use` line, where `Message` stops being a WIT record and
  becomes a plain serde struct. That single line is the difference between the
  two architectures: adding a field to it now breaks nothing and rebuilds
  nothing else.
  One addition beyond a straight port: a zero `max_history`/`max_content_chars`
  is now rejected. The extension had no way to report it, so a caller passing
  zero got an empty history back and discovered it much later as a model that
  had forgotten the conversation. This also resolved, legitimately, the SDK
  friction recorded in AWU 953 — a handler must return `Result` even when it
  cannot fail, and clippy's `unnecessary_wraps` then fires. **Second occurrence;
  the next module that has no honest failure case should get an SDK fix rather
  than an invented one.**

#### AWU 957: Route `CallExtension` to a kernel module when one provides it
- **Objective**: Move the live traffic without changing the caller.
- **Scope**: `src/wasm/rpc_meta*.rs`, kernel wiring in `src/main.rs`.
- **Context**: The orchestrator asks for `context-tools.optimize` over
  `CallExtension`. The host resolves that against the kernel registry first and
  falls back to the extension path, so the orchestrator — still a Wasm extension
  itself — needs no change. This is the shape every later stage reuses.
- **Result**: Proven the only way that actually proves it. Both paths produce
  byte-identical output, so a matching summary line shows nothing; the extension
  was disabled in config instead, and compaction kept working — which it could
  only do through the module. Removing the module from config falls straight
  back to the extension, also verified.
  The first attempt silently did nothing, and the reason is worth keeping: the
  legacy call names a role and a bare method (`context-tools`, `optimize`) while
  module methods are namespaced (`context.optimize`), so nothing resolved and it
  fell through to the extension. **A fallback that works makes a failure to
  route look like success** — only disabling the fallback exposed it. The bridge
  now maps `(role, method)` to `role.method` and nothing cleverer, and the
  module is named for the role the orchestrator still asks for. It drops the
  `-tools` at stage 8, when the orchestrator becomes a module and can call
  `context.optimize` directly.
  The orchestrator needed no change at all: the JSON wire format it already
  sends matches the module's serde structs field for field.

#### AWU 958: Delete `ext/context-tools` and its WIT
- **Objective**: Remove the old copy once nothing routes to it.
- **Scope**: `ext/context-tools/`, `wit/context-tools.wit`, `src/wasm/bindings.rs`.
- **Context**: `get-repo-map` goes with it — dead, and the kernel has no syscall
  for it by design.
- **Result**: Extension, WIT, bindings, linker arm and dispatch all removed, and
  a real task still compacts — now only through the module.
  Test count reads 195, down from 206, which is arithmetic rather than lost
  coverage: the extension's 13 unit tests went with it (already reproduced in
  the module), the 2 old integration tests were replaced by 4 new ones covering
  the same ground across the dispatch boundary instead of the WIT one. That
  original test existed because a serialization mismatch once made `optimize`
  silently window nothing while still returning a plausible summary; the
  replacement asserts the message count actually changed, so the same class of
  failure is still caught.
  `context-tools` turned out to be the only role that ever implemented
  `call_extension_method`. Rather than delete the function with its last user,
  it now returns an explicit error naming the method — it is the seam every
  remaining extension passes through as it moves, and a named error is what
  makes a missing module obvious rather than silent.
  **Live config migrated** (`~/.rad/config.json`, backed up to
  `.bak-awu958`): the `context-tools` entry moved from `extensions` to
  `modules`. Without that the deletion would have removed compaction outright.


---

## 🛠️ Short-Term Plan: Phase 70 (Kernel Surface Alongside the Existing One)

**Stages 1–2 of `ARCHITECTURE-NEXT.md` §9.** Adds the kernel's WIT package, the
guest SDK, and the dispatch machinery — all *next to* the existing extension
surface, which is left untouched. Nothing is ported yet; the deliverable is that
a trivial module can be loaded and called while the six current extensions carry
on working.

Three design questions that blocked this are now settled (§3.6.7, §5.3, §5.3.1):
`rad-abi` holds only the manifest schema, since opaque dispatch means the kernel
never parses payloads; `rad-sdk` uses a declarative `module!` macro rather than a
proc-macro attribute; and modules are configured in a separate `modules` array,
so migration never has to guess whether an entry is old or new.

### 💡 Current AWU Status
- [✅] AWU 949: Add `wit/kernel/kernel.wit` and register its bindings (Result: Success — §2's claim holds on the real build; all six extensions still load and complete a live task)
- [✅] AWU 950: `rad-abi` — manifest schema (Result: Success — manifest types only; builds native and wasm32-wasip2)
- [✅] AWU 951: `rad-sdk` — `module!` macro (Result: Success — split into `routes!`/`module!` so routing stays natively testable; a real module builds to a component exporting `manifest`/`handle`)
- [✅] AWU 952: Module loading, `manifest()` reading, routing table (Result: Success — the real `modules/echo` component loads, answers, and conflicts are caught at registration)
- [✅] AWU 953: `dispatch.call`/`post` with cycle detection (Result: Success — proven between two real wasm modules; the cycle that would deadlock is refused before the lock is taken)
- [✅] AWU 954: Epoch interruption (Result: Success — a runaway module is preempted; `async_support` and the scheduler deferred, see Result)
- [✅] AWU 955: `modules` config array and an echo module proving the path (Result: Success — both surfaces live in one process against the real binary)

### 📝 AWU Details

#### AWU 949: Add `wit/kernel.wit` and register its bindings
- **Objective**: Land the kernel contract without disturbing anything.
- **Scope**: `wit/kernel.wit` (new), `src/wasm/bindings.rs`.
- **Context**: A seventh world across a fourth package. §2's second experiment
  proves a *new* package cannot break existing extensions; this AWU is where that
  claim gets exercised for real rather than in a throwaway probe.
- **Result**: Confirmed. All six extensions load unchanged, `mcp-tool-provider`
  still verifies its 19 tools, and a live task against the local endpoint
  completes — with a fourth WIT package registered in the same binary. The
  migration's central assumption survives contact with the real build.
  Two things surfaced that the design had not anticipated. The file lives in
  `wit/kernel/`, not beside `rad.wit`, because `rad_context_tools` binds
  `path: "wit"` as a *directory* and a second unrelated package there collides
  with that scan — the existing code already documents this hazard for
  `wit/connector/`. And `stream` turned out to be a reserved word in WIT, since
  the component model claims it for the async `stream<T>` primitive underlying
  WASI 0.3; the resource is now `byte-stream`. That is the §3.1.1 argument
  showing up in the syntax itself.

#### AWU 950: `rad-abi` — manifest schema
- **Objective**: One shared type, agreed by guest and kernel.
- **Scope**: `crates/rad-abi/` (new, ~2 files).
- **Context**: The only thing both sides must agree on. Payload types stay out
  until a second consumer actually exists (§5.3.1).
- **Result**: `Manifest` + `ManifestError`, six tests. Kept to manifest types
  only, as decided in §5.3.1. One test pins `ABI_VERSION` against the WIT
  package version — they live in separate files with nothing linking them, so
  silent divergence is the failure worth guarding.

#### AWU 951: `rad-sdk` — `module!` macro and syscall wrappers
- **Objective**: Make writing a module short enough to be worth doing (§5.3).
- **Scope**: `crates/rad-sdk/` (new, ~3 files).
- **Context**: `macro_rules!`, not a proc macro. Generates `manifest()` from the
  method map so `provides` cannot drift from the implementation, plus `handle()`
  dispatch with serde on both sides, plus the `export!` wiring.
- **Result**: `modules/echo` compiles to a component whose exports are exactly
  `manifest` and `handle` (verified by instantiating it under wasmtime and
  reading the component type). `provides` is derived from the method map, so it
  cannot drift from the implementation.
  Two things forced design changes. The SDK cannot own the WIT bindings and
  re-export `export!`: wit-bindgen emits `#[export_name]` shims that edition
  2024 requires to be `#[unsafe(export_name)]`, and across a crate boundary
  they land where nothing can accept them — a hard error, not a lint. Bindings
  are generated in the module's crate instead, taking a *path* to the single
  `wit/kernel/kernel.wit` rather than a copy, so no drift is possible. And the
  macro is split in two: `routes!` generates manifest/handle and is testable
  natively, `module!` adds the bindings and can only build on-target.
  Also observed: a module imports only what it uses — `echo` touches neither
  syscall nor dispatch, and its component declares no `rad:kernel` imports at
  all. The kernel's linker must therefore tolerate modules that import nothing
  from it (relevant to AWU 952).

#### AWU 952: Module loading, `manifest()` reading, routing table
- **Objective**: The kernel can find out what a module provides.
- **Scope**: `src/wasm/` (module loader), routing table.
- **Context**: `manifest()` must be callable before the module has any
  capabilities — it is required to be pure (§3.2). Duplicate `provides` entries
  are a startup error, never implicit first-wins (§3.6.8).
- **Result**: `src/kernel/` — deliberately a separate tree from `src/wasm/`, so
  the final step is a deletion rather than a disentangling. Registration rejects
  both a duplicate method (naming the method and both modules) and a duplicate
  module name, and validates all methods before inserting any: a conflict on a
  module's second method would otherwise leave its first already routed to a
  module that was then refused.
  Also enforced: a module whose configured name disagrees with its self-declared
  one is rejected, since routing and diagnostics would otherwise disagree about
  what to call it.
  Syscall and dispatch hosts are stubs returning explicit errors — the linker
  must supply every import for a component to instantiate at all, and
  instantiation is what this AWU verifies. Confirmed against the real component:
  `modules/echo` loads, `manifest()` is readable before anything is granted,
  `handle()` round-trips, a handler error crosses as a message rather than a
  trap, and an unknown method leaves the module still working.

#### AWU 953: `dispatch.call`/`post` with cycle detection
- **Objective**: The two dispatch primitives, including the anti-deadlock rule.
- **Scope**: dispatch host implementation.
- **Context**: `call` tracks the call stack and rejects re-entry with an explicit
  error rather than hanging (§3.6.3); `post` queues for delivery when the target
  is idle (§3.6.2). The kernel itself registers as a dispatch target so modules
  reach `kernel.config` the same way they reach each other (§3.6.7).
- **Result**: Both demonstrated with real components (`modules/relay` forwards,
  `modules/echo` answers). The locking shape is what makes cycle detection
  necessary rather than merely tidy: `dispatch.call` runs while the caller's
  `Store` is borrowed, so runtimes cannot share one lock — each has its own, and
  re-entering a module already on the call stack would block on a lock the
  in-flight call still holds. The check therefore happens *before* the target's
  lock is taken; afterwards it is a hang nobody could report. The stack unwinds
  on rejection, verified by making a second call succeed after a refused cycle.
  Also landed: routing by method name as well as module name, so a caller can
  ask for a capability without knowing which module currently provides it; and a
  bounded drain, so a module posting to itself on every message cannot spin the
  queue forever.
  One SDK ergonomics issue surfaced: every handler must return `Result` even when
  it cannot fail, and clippy's `unnecessary_wraps` then fires in the module
  author's crate. Worked around legitimately here; a `methods` form accepting
  infallible handlers is worth adding before third parties write modules.

#### AWU 954: Async wasmtime, epoch interruption, scheduler loop
- **Objective**: The execution model of §3.6.
- **Scope**: `src/wasm/loader.rs` (Config), scheduler.
- **Context**: `async_support` so a blocking syscall suspends only its caller;
  `epoch_interruption` so a runaway module can be preempted, which
  `src/esc_abort.rs`'s cooperative flag cannot do. Both confirmed present in
  wasmtime 29.0.1. **The existing extension path must keep working under async.**
- **Result**: Epoch interruption landed and demonstrated: `relay.spin` loops
  forever and is trapped, the error names the method and says it exceeded its
  budget rather than merely "trapped", and the kernel keeps serving other
  modules afterwards. The engine moved to `KernelShared` so one ticker thread
  drives every module's deadline, and the budget is per-module so different
  workloads can differ — a compaction pass and a UI redraw do not deserve the
  same ceiling.
  **`async_support` and the scheduler loop were deliberately not done.** Async
  exists to stop a blocking syscall from holding the process, and every syscall
  is still a stub — there is nothing yet to block on. Enabling it would make
  every call on the engine async, forcing AWU 953's dispatch to be rewritten
  around re-entrant async delivery for no present benefit, which is the
  speculative complexity `CODING.md` §3 rules out. It belongs with `net-open`,
  the first syscall that actually blocks. The scheduler is the same story: it
  has nothing to schedule until modules own the main loop (stage 9). Neither is
  abandoned — both move to where they earn their cost.

#### AWU 955: `modules` config array and an echo module proving the path
- **Objective**: End-to-end proof on the smallest possible module.
- **Scope**: `src/config.rs`, `modules/echo/` (new).
- **Context**: `#[serde(default)]` so existing configs are untouched (§3.6.7).
- **Result**: Verified against the real binary with a real `~/.rad/config.json`
  entry (restored afterwards): `[OK] Verified 19 tools from extension
  'mcp-tool-provider'` and `[OK] Loaded 1 kernel module(s): echo` in the same
  startup, followed by a completed LLM task. Both surfaces coexist, which is the
  premise the whole in-place migration rests on.
  `kernel.config` works: the kernel registers as an ordinary dispatch target, so
  a module fetches its config with the same call it would use for a peer and
  never special-cases the host. A broken or disabled module is skipped with a
  warning rather than aborting startup — one bad third-party module must not
  stop rad from running.

---

## 🛠️ Short-Term Plan: Phase 69 (Microkernel Migration — Preparation & Stage 0)

**Direction**: `ARCHITECTURE-NEXT.md` defines the target: the Core becomes a Wasm
runtime and dispatcher, and every capability becomes a module above it. The
contract splits in two — typed syscalls that may only grow by new functions, and
an opaque `dispatch(target, method, payload)` whose type never changes.

**The migration happens in place, one extension at a time.** `wit/kernel.wit` is
added as a *new* WIT package, which §2's second experiment proves cannot break
the existing extensions — and the Core already hosts six worlds across three
packages, so a seventh is routine. The old RPC surface and the new dispatch
surface coexist while extensions move across one by one; the kernel is what
remains after the last old surface is deleted, not something built separately
and swapped in. **rad works at every step**, and the `--workspace` CI covers the
whole thing throughout.

**Do not treat `ARCHITECTURE-NEXT.md` as current.** Nothing in it is implemented.

### 💡 Current AWU Status
- [✅] AWU 946: Verify CI is green after the workspace fix (Result: Success — all five jobs green on `ea7740c`, 195 tests actually executed on both runners)
- [✅] AWU 947: Settle `net-open` vs `wasi:http` (Result: Success — keep `net-open`; WASI 0.3 deleted `wasi:io`, so importing `wasi:http` would break every module at once)
- [✅] AWU 948: Stage 0 — dialect table in `ext/llm-connector` (Result: Success — Gemini and Azure now expressible; existing profiles verified bit-identical, end-to-end against the live llama.cpp endpoint)

### 📝 AWU Details

#### AWU 946: Verify CI is green after the workspace fix
- **Objective**: Establish a trustworthy baseline before a long parallel rebuild.
- **Context**: CI has never actually run `clippy`/`test` — the `check-secrets` job
  was failing on a transient `actions/checkout` resolution error, which skips
  `build-and-test`. Phase 68 fixed what CI runs; that it passes is unconfirmed.
- **Result**: Green, and getting there surfaced six real problems that the
  outage had been masking. The Actions outage was the surface; underneath, CI
  had never once run clippy or the test suite.
  betterleaks was never installed — the workflow piped an `install.sh` that does
  not exist, so `check-secrets` died on `command not found` and took the other
  jobs with it as skipped dependencies. `actions/checkout` clones at depth 1,
  and `betterleaks git` on a one-commit clone prints "no leaks found" —
  indistinguishable from a real pass while 222 commits sat unexamined. The
  licence audit sat in a three-OS matrix while `cargo-deny-action` is a Linux
  container, and used `|| true`, so a failed install skipped it silently. No job
  built the `.wasm` components the integration suite loads, so 82 of 195 tests
  ran and the rest were never attempted.
  Two were bugs rather than CI configuration. **`kill_group` sent SIGKILL and
  never reaped**, so every spawned process stayed a zombie until rad exited —
  one PID slot per spawn, in a program whose job is spawning things. It passed
  locally because macOS drops zombies from `getpgid` promptly and Linux does
  not; the test was right and the implementation was wrong. And **Windows had
  never compiled at all** — listed in the matrix since the workflow's first
  commit, promising support that does not exist (now Phase 73).
  Three of my own measurements were also wrong: an awk summing test results
  could not count failures (it reported "0 failed" for a run with three), the
  tests I added this session returned early on missing components and so passed
  vacuously, and a bulk edit placed timeout assertions inside the loops they
  were meant to follow.
  Nine tests waited on a budget and then continued regardless, so a slow runner
  surfaced as whichever later assertion first noticed an incomplete state. They
  now assert that the task finished, which turns a misleading logic failure into
  "it timed out".
  The single lesson, repeated in every one of these: **a check that cannot run
  is far worse than one that fails, because it looks like a pass.**

#### AWU 947: Settle `net-open` vs `wasi:http`
- **Objective**: Decide whether the kernel needs a custom HTTP syscall at all.
- **Context**: `ARCHITECTURE-NEXT.md` §3.1 lists three syscalls and flags this one
  as unresolved. `wasi:http`'s outgoing-handler may cover it, which would leave
  two syscalls and drop `resource stream`. **This determines the content of
  `wit/kernel.wit`, so it must be settled before stage 1 begins.**
- **Result**: Resolved on stronger grounds than the latency measurement it asked
  for. `wasi:http` *can* express SSE (`incoming-body.stream() -> input-stream`),
  but WASI 0.3.0 (2026-06-11) removed the `wasi:io` package outright — pollables
  and streams moved into the Canonical ABI, and `wasi:http` was refactored onto
  `stream<T>`/`future<T>`, requiring Wasmtime 43+ (rad is on 29). A module
  importing `wasi:http@0.2` therefore breaks on the 0.3 move — exactly the
  failure `ARCHITECTURE-NEXT.md` §2 exists to prevent, but originating outside
  the project where it cannot be controlled. Keeping `net-open` lets the kernel
  absorb that break instead. Recorded as §3.1.1, with §3.1.2 generalising the
  rule: use WASI where Rust's std insulates the module (fs, clock, stdio); define
  a rad syscall where it does not (http, process spawn). Syscalls are fixed at
  three, and `wit/kernel.wit` is unblocked.

#### AWU 948: Stage 0 — dialect table in `ext/llm-connector`
- **Objective**: `const Dialect` table plus `LlmEndpointProfile.dialect`, bringing
  Gemini and Azure into range.
- **Scope**: `ext/llm-connector/src/`, `wit/connector/llm-connector.wit`,
  `src/config.rs`, `src/command/llm/`.
- **Context**: Self-contained in the connector's own WIT package, so it does not
  touch the shared `wit/rad.wit`. Independent of every other decision here and
  ports to the new design unchanged — the one item that cannot become rework.
- **Result**: `const Dialect` table with struct-update inheritance — Gemini is
  one field, Azure two. `{model}` substitution in the path covers Azure putting
  the deployment name in the URL. SSE parsing now reads the dialect's JSON
  Pointers instead of three hardcoded strings. `dialect: Option<String>` on the
  profile, `--dialect` on `/llm add` (preserved across re-runs like
  `context_length`), threaded host-side through `resolve_active_llm_profile`.
  Unknown names warn and fall back to `openai` rather than failing.
  Regression-pinned: a dedicated test asserts the `None` dialect produces
  byte-identical URLs, headers and pointers to the previous hardcoded code, and
  a live run against `127.0.0.1:8080` confirmed the endpoint is unchanged.
  157 tests pass (was 149); clippy clean on native and wasm32-wasip2.

---

## 🛠️ Short-Term Plan: Phase 68 (Repository Hygiene & Convention Audit)

### 💡 Current AWU Status
- [✅] AWU 943: Rewrite commit authorship across all 195 commits (Result: Success — `git filter-repo`, repo deleted and recreated so old SHAs are unreachable)
- [✅] AWU 944: Fix CI to run the whole workspace (Result: Success — `--all-targets` selected only the root package, skipping 52 of 149 tests; cleared 11 latent `needless_update` errors and split two files over the 300-line limit)
- [✅] AWU 945: Audit and correct the convention documents (Result: Success — TESTING.md's 85% coverage target had no measurement tooling behind it; CODING.md mandated the same `--all-targets` trap and banned an `#[allow]` that generated code requires; AGENTS.md's governance map omitted over half the documents)

---

## 🛠️ Short-Term Plan: Phase 67

### 💡 Current AWU Status
- [✅] AWU 942: Spec-First Architecture Reconciliation (Result: Success)

### 📝 AWU Details

#### AWU 942: Spec-First Architecture Reconciliation (ARCHITECTURE.md & README.md)
- **Trigger**: User asked for a refactoring pass beginning with a fresh comparison of `ARCHITECTURE.md` against the actual code. Two prior audits had covered adjacent ground (AWU 934 documentation alignment, AWU 935 architecture re-verification), but both compared *listings* — enum variants, RPC command tables, file paths. This pass instead checked the document's **behavioral claims**, which is where the surviving defects were: every finding below describes a mechanism the document asserted but the code never performed, and none of them were detectable by diffing symbol lists.
- **Process**: user directed a spec-first flow — write `ARCHITECTURE.md` as the intended design first, then move code to match — and explicitly asked for the *correct* design even where implementation cost was higher. Three findings were genuine forks (implement the doc vs. correct the doc); the rest were unambiguous documentation defects. Each fork was decided on architectural merit rather than cost, which produced a split verdict rather than a blanket "implement everything".
- **Finding 1 — §5.3 Slash Commands was entirely fictional (doc corrected)**: the section claimed the Core "simply passes the text event, and the Extension handles parsing and execution control", with a sequence diagram showing `HumanInputReceived{"/rollback node_a1b2"}` being delivered to an extension that parses the leading `/`. No such flow exists: `process_input` (`src/main.rs`) resolves commands entirely host-side through the static `CommandSpec` registry (`src/command.rs`) → `cmd_rollback` (`src/command/handlers.rs`) → `orchestrator.rollback()`. **The host-side design is the correct one and was kept**: a command like `/rollback` or `/reload` acts *on* the extension runtimes (rewinding the DAG, discarding cached Wasm instances), so it cannot be implemented by the extensions it manipulates. The section was rewritten to document the real four-tier resolution order (`!`-shell → built-in registry → markdown templates → agent-task fallthrough) and the reason it must be host-side.
- **Finding 2 — L3 recovery was specified but unimplemented, and classified in a layer where it can never be observed (implemented)**: the doc promised that on context exhaustion the Orchestrator would "re-invoke `context-tools`'s compaction … then reset state". The actual L3 arm (`done.rs`'s `handle_l2_l3_error`) only prints a message. Worse, that classifier runs on the **tool-execution** path, whereas context exhaustion is reported by the backend rejecting a request — it arrives on the **LLM response** path (`LlmConnectorEvent` with an error payload) and could never reach the L3 arm at all. This was judged a real correctness gap rather than aspirational prose, because it is the designed complement to an earlier decision: Phase 49-2 deliberately declined exact tokenization, leaving budgeting on a `chars/4` approximation, on the reasoning that the approximation errs toward under-use. A reactive backstop for the cases where it errs the other way (dense code, CJK) is exactly what makes that trade-off sound. **Implemented** as a new `context_recovery` module owning the policy: narrow case-insensitive matching of context-exhaustion phrasings (deliberately excluding generic words like "exceeds"/"limit" that also appear in rate-limit and output-token errors, since a false positive burns retries on a permanent failure), a compounding 60% budget backoff saturating at 1% rather than 0 (a zero budget would make compaction discard everything), and a hard cap of 2 retries. Bounding is the essential property — an unbounded shrink-and-retry would spin forever whenever a failure merely *looks* budget-related. The reduced budget is threaded through `load_messages_from_dag`'s `max_content_chars`, and reset to full on any turn that completes without rejection so one transient over-estimate cannot starve the rest of the task.
- **Finding 3 — FS watcher was unreachable in production (deleted)**: §2.1 listed filesystem change detection as a tracked subsystem state. `FsWatcher` (`src/fs/watcher.rs`, `notify`-based) existed and worked, but was instantiated **only by its own tests** — and even had it been wired up, `handle_event` matches `HumanInputReceived`/`LlmConnectorEvent`/`Rehydrate` and drops everything else through `_ => Ok(())`, so `FileChanged` had no consumer either. Inventing a consumer now would be speculative infrastructure with no demand (the Phase 47-4 caution), and the concrete motivation one might reach for — an agent editing against externally-stale content — is already handled correctly by content-addressed edits failing loudly on mismatch (AWU 940). **Deleted end-to-end** following the AWU 932 precedent: `src/fs/watcher.rs` + tests, `RasCoreEvent::FileChanged`, the `file-changed` WIT variant and `file-change-info` record, both conversion arms, and the now-unused `notify` dependency.
- **Finding 4 — `execute-tool`'s gating was overstated (doc corrected)**: §1.1 listed `open-file`/`open-process`/`execute-tool` together as passing through a gateway enforcing `rad.json` whitelists. `imports_tool.rs` calls only `verify_rpc_exclude` (the security-guard hook), never `check_permissions`. **Not "fixed" in code deliberately**: `PermissionConfig` has no dimension describing which tools may be invoked, so a mask check on `ExecuteTool` would fall through `_ => Ok(())` — structurally vacuous. Making it meaningful would require inventing a per-extension tool allowlist, which is a new feature, not a reconciliation. The section now documents the real two-layer model: the security-guard hook gates the call, and the tool's physical side-effects are masked one layer down when the tool-provider extension performs the actual `file-write`/`spawn-bash-process` RPC against *its own* permissions.
- **Findings 5–8 — documentation-only defects**: §3.1's `RasCoreEvent` listing omitted `LlmConnectorEvent` (the Orchestrator's *primary* input during a turn); §1.2 claimed "count-based windowing only", contradicting §1.3 and the actual `optimize` pipeline (stale tool-result clearing → count- *and* size-bounded windowing → relevance-based retention); §5.5 named a nonexistent RPC `request_human_approval` (real: `ask-human-approval`), while §3.2.3 listed it correctly — an internal contradiction; §5.4's intro still asserted every tool comes from MCP servers, contradicting §5.4.1 immediately below it after Skills landed. Also corrected two stale *code* comments found in passing, both claiming compaction is count-based-only (`llm.rs`, `context-tools/src/lib.rs`) — the same drift as §1.2, in the code rather than the doc. The tool-failure circuit breaker (AWU 939) was undocumented and is now specified as §5.1.3 alongside L3 as §5.1.2, since the two are the same architectural pattern: converting a potentially infinite retry into a bounded one that terminates with a clear diagnosis.
- **Scope**: `ARCHITECTURE.md`, `ext/rad-orchestrator/src/context_recovery.rs` (new), `ext/rad-orchestrator/src/context_recovery/tests.rs` (new), `ext/rad-orchestrator/src/{lib,types,llm,orchestrator}.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `ext/context-tools/src/lib.rs`, `src/fs.rs`, `src/fs/watcher.rs` (deleted), `models/src/lib.rs`, `wit/rad.wit`, `src/wasm/bindings_event.rs`, `ext/rad-orchestrator/src/conv/event.rs`, `Cargo.toml`.
- **Finding 9 — post-implementation review caught a bug in this AWU's own L3 code (fixed)**: a second verification pass over the just-written implementation found that the retry path never cleared the per-turn streaming buffers. A rejected attempt can leave partial text in `state.assistant`, buffered reasoning, half-assembled `tool_calls` deltas, and `is_reasoning = true`; because a retry re-runs the *whole* turn, the retry's tokens would append to that residue and `handle_done` would write one corrupted assistant message to the DAG — with tool-call deltas that never received closing arguments. Fixed by clearing all four in the `Retry` branch of `plan_context_recovery`, where the decision to retry is committed under the same `STATE` lock. Notably the *original* (unimplemented) spec text said "…then reset state", a clause dismissed as vague during Finding 2's design; it was in fact naming this exact requirement. §5.1.2 now states the buffer reset as a numbered step rather than leaving it implicit.
- **Finding 10 — the WIT contract is duplicated with no sync mechanism (guard added)**: `wit/rad.wit` is copied verbatim into `templates/rust/wit/rad.wit` and `templates/go/wit/rad.wit` so scaffolded extensions compile standalone. Nothing keeps them in sync, and Finding 3's `file-changed` removal silently desynced both — the *same* class of breakage AWU 934 had already repaired once (for `spawn-mcp-server`). Two occurrences on consecutive WIT edits makes the duplication itself the defect, not the individual misses. Copies re-synced, and `scripts/build_all.sh` gained a pre-flight `diff` gate that fails the build with the exact `cp` fix-up command. The guard was verified in both directions: passing on synced copies, and correctly failing on a deliberately introduced drift. (The many `rad.wit` copies under `.rad/snapshots/` are frozen rollback snapshots and are intentionally excluded — they *should* preserve their historical contents.)
- **Finding 11 — `README.md` audited on the same basis; its contributor instructions were actively harmful (fixed)**: the same behavioral-claim check was applied to `README.md`, and the worst defect was in the instructions contributors are told to follow before submitting code. §2.3/§4.4 both prescribed `cargo test -- --test-threads=1`; measured, a bare `cargo test` runs **97 tests versus 149 for `cargo test --workspace`** — it silently skips every test in `ext/*` and `models/`, about a third of the suite, so anyone following the README could believe a green run covered code it never touched. The `--test-threads=1` requirement was itself stale: env-var-sharing tests now serialize themselves via an in-file `TEST_MUTEX` (6 files), and `scripts/build_all.sh` — the actual project standard — has long run plain parallel `cargo test --workspace`, directly contradicting the README. §4.4's `cargo clippy --all-targets` was likewise weaker than the enforced `cargo clippy --workspace -- -D warnings` (no `-D warnings` means warnings don't fail). Both sections now match `build_all.sh`, with a note on why `--workspace` is mandatory and an added `wasm32-wasip2` lint step, since a native-only check misses target-specific breakage in the extensions.
- **Finding 12 — remaining `README.md` inaccuracies (fixed)**: prerequisites claimed Rust 1.75 while the whole workspace is on edition 2024 (needs 1.85+); the clone URL was still the `yourusername` placeholder rather than the real remote; §4.1 described "isolated micro-extensions (LLM Orchestrator, Security Guard, Tool Provider)" — three — contradicting §1's correct list of six in the same document; §4.2's L3 strategy still read "run `context-tools` pruning/summarization and reset", describing summarization that was never implemented and predating Finding 2's actual bounded-backoff design; §4.3 claimed `cargo build` "copies" the artifact to the configured location, which it does not (only `build_all.sh` installs), and omitted the `/reload` step needed for a running session to pick up a replaced `.wasm`. §4.3 now also points at the WIT-mirroring requirement from Finding 10. Verified as correct and left unchanged: the §3.1 slash-command list (all 10 match the `CommandSpec` registry — an initial mismatch was traced to a flawed extraction regex on my side, not a doc defect) and §3.2's "6 extensions" count.
- **Finding 13 — the onboarding path dead-ended (new `README.md` §3.3 added)**: §3.2 correctly warned that "without at least one MCP server configured, the agent has zero tools and can't act on anything" — and then never told the reader where to obtain one, leaving a fresh install with no route to a working agent. Added §3.3 "Companion MCP Servers" documenting the two servers `rad` is dogfooded against, [`core-utilities-mcp`](https://github.com/akahmys/core-utilities-mcp) (15 tools: filesystem, shell, structured data) and [`web-access-mcp`](https://github.com/akahmys/web-access-mcp) (4 tools: fetch/search with headless-Chromium fallback), with install commands and the `config.mcp_servers` snippet. Framed explicitly as "any stdio MCP server works — `rad` is not coupled to a particular one" so the section documents an example rather than implying a hard dependency, consistent with §5.4's provider-agnostic invariant. Every claim was verified against the sources rather than written from memory: tool counts (15 and 4) counted from each server's own tool table and cross-checked against a live run reporting `Verified 19 tools`; repository URLs checked against the actual `git remote`s rather than assumed; installed binary paths confirmed present at `~/.cargo/bin/`; the `edit_file` description confirmed content-addressed (`old_string`, no `start_line`) per AWU 940; and the Chrome/Chromium prerequisite taken from `web-access-mcp`'s own documented requirements.
- **Finding 14 — `README.md` prose pass surfaced a false quantitative claim (fixed)**: user asked for an editorial pass, specifically calling the opening redundant and asking for factual rather than promotional phrasing. The intro carried two competing definitions in consecutive paragraphs ("minimal, ultra-fast agent-oriented shell runtime" then "coding agent runtime inspired by pi-coding-agent"); collapsed to one sentence naming what `rad` is, plus one describing the mechanism/policy split in plain terms. Removing marketing adjectives then required checking whether the measurable ones were even true — and **"a tiny memory footprint of just a few megabytes" was false: the installed binary is 24 MB** (extensions add 2.8 MB). "Starts instantly" was likewise unmeasured; actual cold start including extension load is ~0.4 s. Both were long-standing claims inherited from the original README, and an earlier draft of this pass had preserved the size figure verbatim while rewriting around it — a reminder that copy-editing propagates unverified assertions unless each one is independently checked. The bullet was replaced with a factual statement about the Core/extension split. Other unsubstantiated qualifiers removed throughout ("instantly", "simply", "robust", "Zero Dependencies" — the last also contradicted the design, since the binary does nothing without its `.wasm` extensions).
- **Finding 15 — `/reload`'s documented behavior omitted its most useful effect (fixed)**: §3.1 described `/reload` as "dynamically reloads the configuration file". Reading `Orchestrator::reload()` shows it also reapplies sandbox permissions and — the operationally important part — calls `wasm_runtime.lock().clear()`, dropping cached runtimes so the next task re-loads extensions from disk. That is precisely what makes a rebuilt `.wasm` take effect without restarting, a workflow §4.3 now depends on. Description corrected, and the §3.1 command list made grammatically consistent (two of ten entries were third-person amid eight imperatives). Every command description was re-checked against its handler: `/session`'s reported fields, `/new`'s session-ID rotation plus DAG clear, and `/compact`'s persist-versus-ephemeral distinction all verified against the code rather than trusted from the existing text.
- **Definition of Done (DoD)**: `ARCHITECTURE.md` and `README.md` contain no claim the code does not perform, and every command they instruct a contributor to run is the one the project actually enforces; all tests + Clippy (`-D warnings`) pass on native and `wasm32-wasip2`; CODING.md's 300-line file limit respected; no regression in a real `rad` run.
- **Result**: Success. 7 new unit tests in `context_recovery` covering backend phrasing recognition, false-positive rejection (rate limits, output-token limits, connection errors), compounding backoff, and the non-zero saturation floor. Full workspace suite green on both targets (the one removed test is the deleted watcher's own). `llm.rs` had crossed to 301 lines and was brought back to 299 by rewriting — not merely shortening — the stale comment flagged above. Verified against the real installed binary and a live local LLM: multi-turn tool loop, compaction, and thinking display all unaffected. A final mechanical re-verification confirms every remaining `ARCHITECTURE.md` claim against the code: the §3.1 event listing and §3.2.3 RPC listing both diff clean against `models/src/lib.rs` and `wit/rad.wit`, §5.5's RPC name resolves, §5.1.2's specified behaviors are present, and §2.1's "no FS events" claim holds with zero `FileChanged` references left in `src`/`models`/`wit`.
- **Known pre-existing flakiness (investigated, not introduced here, left as-is)**: `src/http/tests.rs`'s three streaming tests failed once during a full-workspace run immediately after a large rebuild, then passed 3/3 in isolation, 2/2 parallel, 2/2 single-threaded, and 2/2 in subsequent full-workspace runs. Root cause is their tight 2-second `recv_timeout` budget for connect-plus-stream, which can be exceeded when many test binaries and tokio runtimes contend for a loaded machine. Ruled out as a regression from this AWU: a `tokio` feature-unification hypothesis (that dropping `notify` had disabled `tokio/net`) was tested directly with `cargo tree -e features` and disproved — `net` is enabled identically with and without `notify`. Raising the timeout is a reasonable follow-up but was left out of scope rather than changed on a hunch.

---

## 🛠️ Short-Term Plan: Phase 66

### 💡 Current AWU Status
- [✅] AWU 941: Rebuild WASM Component Extensions and Reinstall `rad` Binary Locally (Result: Success)

### 📝 AWU Details

#### AWU 941: Rebuild WASM Component Extensions and Reinstall `rad` Binary Locally
- **Trigger**: User requested re-installing `rad` locally (`ローカルにインストールし直して`).
- **Scope**: `~/.rad/wasm/`, `~/.cargo/bin/rad`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Run `./scripts/build_all.sh` with network bypass (`BypassSandbox: true`) so all verification steps (WASM compilation, formatting, license checks, secret scanning, unit/integration tests, Clippy audit, and `cargo install --path .`) succeed cleanly.
  - Verify that all 6 WASM extension components are copied to `~/.rad/wasm/`.
  - Verify that `rad` binary is installed cleanly at `~/.cargo/bin/rad`.
- **Result**: Success. Executed `./scripts/build_all.sh` with network bypass. All 6 WASM extension components (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`) built and deployed to `~/.rad/wasm/`, all audit checks and workspace tests passed, and `rad` binary installed successfully at `~/.cargo/bin/rad`.

---

## 🛠️ Short-Term Plan: Phase 65

### 💡 Current AWU Status
- [✅] AWU 940: Comprehensive Audit of Unit and Integration Test Coverage across Core & WASM Extensions (Result: Success)

### 📝 AWU Details

#### AWU 940: Comprehensive Audit of Unit and Integration Test Coverage across Core & WASM Extensions
- **Trigger**: User requested an audit to verify if test code in the workspace has any gaps, excess, or missing scenarios.
- **Scope**: `tests/`, `src/`, `ext/`, `models/`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Audit all test files in `tests/` and companion test modules in `src/` and `ext/`.
  - Check coverage for Core subsystems (FS, Process, DAG, Network, Wasm loader, RPC meta, Permissions, Commands, Config), WASM extensions (rad-orchestrator, llm-connector, security-guard, mcp-tool-provider, skill-tool-provider, context-tools), and IPC contract (`models`).
  - Identify any redundant/dead tests (excess) or missing critical test scenarios (deficiency/gaps).
  - Run test suite and present audit findings clearly to the user.
- **Result**: Success. Audited total of 17 integration test files in `tests/` and 12 companion test modules across `src/`, `ext/`, and `models/`. All test suites are active, well-structured, compliant with the CODING.md requirement (companion test files `< 300` lines, zero `#[cfg(test)]` in production files), and 100% passing across the workspace. Zero dead/redundant legacy tests remain (cleaned up during AWU 932/933).

---

## 🛠️ Short-Term Plan: Phase 64

### 💡 Current AWU Status
- [✅] AWU 939: Fix & Update `scripts/build_all.sh` to Align with Current Project Workspace & Audit Pipeline (Result: Success)

### 📝 AWU Details

#### AWU 939: Fix & Update `scripts/build_all.sh` to Align with Current Project Workspace & Audit Pipeline
- **Trigger**: User requested updating `scripts/build_all.sh` to match current project state, components, and verification pipeline.
- **Scope**: `scripts/build_all.sh`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - `scripts/build_all.sh` builds all 6 current WASM component extensions (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`).
  - Explicitly copy only the built WASM components to `~/.rad/wasm/` and `target/wasm32-wasip2/debug/` instead of wildcards.
  - Include code audit/verification steps (`cargo fmt --check`, `python3 scripts/check_licenses.py`, `scripts/check_secrets.sh --all`).
  - Perform clean build, test (`cargo test --workspace`), clippy (`cargo clippy --workspace -- -D warnings`), and local binary installation (`cargo install --path .`).
  - Verification: `./scripts/build_all.sh` executes cleanly.
- **Result**: Success. Updated `scripts/build_all.sh` to explicitly build all 6 WASM extension crates (`rad-orchestrator`, `llm-connector`, `security-guard`, `mcp-tool-provider`, `skill-tool-provider`, `context-tools`), copy specific WASM files instead of wildcards, add code formatting (`cargo fmt --check`), license audit (`python3 scripts/check_licenses.py`), and secrets/path scanning (`scripts/check_secrets.sh --all`). Verified clean end-to-end execution of `./scripts/build_all.sh`.

---

## 🛠️ Short-Term Plan: Phase 64

### 💡 Current AWU Status
- [✅] AWU 939: Edit-Failure & Infinite-Loop Countermeasures (Result: Success)

### 📝 AWU Details

#### AWU 939: Edit-Failure & Infinite-Loop Countermeasures
- **Trigger**: User reported that the LLM frequently fails at file edits and sometimes falls into infinite retry loops, and asked for countermeasures spanning both `rad` and the `core-utilities-mcp` server that provides the edit tool. Investigation found two independent layers at fault, plus (during implementation) a third, more serious pre-existing bug.
- **Rejected scope**: a per-task *total* tool-call cap. It can't distinguish a legitimately long task (a 20-file refactor makes 50+ successful calls) from a stuck one, so any threshold either kills real work or detects loops too late. Narrowed by agreement to tracking **consecutive failures of the same tool** only, which stays silent as long as progress is being made.
- **Fix 1 — shell-quoting corruption (pre-existing bug, found while testing the circuit breaker)**: `spawn_std_pipe_process` (`src/process.rs`) has a Phase-37 "fast direct binary execution" path that skips `bash -c` when the command contains no shell metacharacters — but it builds its argv with a naive `split_whitespace()`, which destroys quoting. Since every tool provider returns its result via `open_process("echo -n '<result>'")` (the only way to satisfy the WIT `execution-handle` return type), `echo` ran directly with argv `["-n", "'{\"directories\":...", ...]`, so **every MCP tool result reached the LLM wrapped in literal single quotes** (`'{"directories":...}'` instead of `{"directories":...}`). Confirmed against real transcripts from earlier in this session. Fixed by treating `'` and `"` as shell features, so quoted commands go through `bash` like redirection already did; the direct-exec optimization still applies to its actual target (MCP server binaries like `~/.cargo/bin/core-utilities-mcp`, which have no quotes). This bug would also have silently defeated the new circuit breaker, since `'Error: ...` doesn't start with `Error:`.
- **Fix 2 — consecutive-failure circuit breaker (`rad`)**: `mcp-tool-provider`'s `execute_tool` flattens `CallToolResult` to plain text and previously discarded `isError` entirely, leaving no structural failure signal. It now normalizes `isError: true` into a guaranteed `Error:` prefix (independent of any server's own wording), and `handle_l2_l3_error`'s L2 branch was aligned to the same prefix. `OrchestratorState` gained `last_tool_name`/`consecutive_tool_failures`/`max_consecutive_tool_failures`; `execute_pending_calls` (`orchestrator/runner/done.rs`) updates the streak after each result, resets it on any success or on a *different* tool failing, and at 4 consecutive same-tool failures stops the task cleanly (skips remaining calls from that turn, prints an explanation, calls `CompleteTask`) instead of triggering another LLM turn. Threshold deliberately generous — 2-3 retries while a model converges on a correct edit is normal, not stuck.
- **Fix 3 — content-addressed `edit_file` (`core-utilities-mcp`)**: the root cause of the edit failures themselves. `EditChunk { start_line, end_line, target_content, replacement_content }` required both a correct line range *and* a near-exact content match, so it broke whenever an earlier edit shifted line numbers or the model's memory of the file went stale after context compaction. Replaced with `EditChunk { old_string, new_string }`: no line numbers at all, matched by locating `old_string` in the file's current content. Uniqueness is required (0 matches → "not found, re-read the file"; 2+ → "not unique, add surrounding context"), so an ambiguous edit is refused rather than guessed at. A whitespace-tolerant line-based search is tried only if there's no exact match, covering invisible trailing-whitespace differences. All chunks are resolved and overlap-checked before anything is written, preserving the previous all-or-nothing atomicity. `replace_all` was deliberately not added — refusing ambiguity is safer than silently editing every occurrence.
- **Scope**: rad — `src/process.rs`, `ext/mcp-tool-provider/src/lib.rs`, `ext/rad-orchestrator/src/types.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `tests/circuit_breaker_tests.rs` (new). core-utilities-mcp (separate repo) — `core-utilities-mcp-lib/src/file_ops/mutate/edit.rs`, its `tests.rs`, `core-utilities-mcp-lib/src/text_ops/read.rs`, `core-utilities-mcp/src/tools.rs`, `core-utilities-mcp/tests/integration_test.rs`, `README.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass in both repos, native and `wasm32-wasip2`; both fixes verified against the real installed binaries, not just mocked tests.
- **Result**: Success. 3 new `rad` integration tests (breaker fires at exactly 4 consecutive same-tool failures leaving later turns unserved; 3 failures stay under threshold; a mid-streak success resets the counter so 6 non-consecutive failures still complete normally) driving a full `Orchestrator` + mocked LLM. 13 rewritten/added `core-utilities-mcp` unit tests covering not-found, ambiguity, disambiguation-by-context, whitespace-tolerant fallback, out-of-order and overlapping chunks, deletion via empty `new_string`, and all-or-nothing rollback. Full test suites and Clippy clean in both repos. Verified against real installed binaries: raw JSON-RPC against `core-utilities-mcp` confirmed all four `edit_file` behaviors including `isError: true` on failure (the exact signal the breaker consumes), and a real `rad` run against a real local LLM confirmed both that tool results now arrive unquoted (`{"directories":...}`) and that a full read→edit→verify cycle succeeds.

---

## 🛠️ Short-Term Plan: Phase 63

### 💡 Current AWU Status
- [✅] AWU 938: Release Build, Local Installation & Push to Main (Result: Success)

### 📝 AWU Details

#### AWU 938: Release Build, Local Installation & Push to Main
- **Trigger**: User requested local installation and pushing current changes to git remote repository.
- **Scope**: `Cargo.toml`, `PLANS.md`, Git repository.
- **Definition of Done (DoD)**:
  - Run `./scripts/build_all.sh` to compile WASM components and install `rad` locally.
  - Commit all modified files with a clear commit message.
  - Push changes to git remote (`main` branch).
- **Result**: Success. Executed `./scripts/build_all.sh` successfully, compiling all WASM extensions and installing `rad` binary to `~/.cargo/bin/rad`. Committed modified files and pushed successfully to `origin/main` (`0419abc`).

---

## 🛠️ Short-Term Plan: Phase 62

### 💡 Current AWU Status
- [✅] AWU 937: Make LLM Thinking Process Display Independent of `RAD_DEBUG` (Result: Success)

### 📝 AWU Details

#### AWU 937: Make LLM Thinking Process Display Independent of `RAD_DEBUG`
- **Trigger**: User requested displaying LLM thinking/reasoning process text without requiring `RAD_DEBUG=1`.
- **Scope**: `ext/rad-orchestrator/src/orchestrator/reasoning.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/runner/done.rs`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Thinking process display is enabled by default independently of `RAD_DEBUG`.
  - Display can be suppressed if explicitly set via `RAD_SHOW_THINKING=0` or `RAD_HIDE_THINKING=1`.
  - All workspace unit tests, clippy checks (`-D warnings`), and `./scripts/build_all.sh` pass cleanly.
- **Result**: Success. Updated `thinking_enabled()` in `ext/rad-orchestrator/src/orchestrator/reasoning.rs` so that thinking traces and markers default to being shown. Added support for explicit opt-out via `RAD_SHOW_THINKING=0` or `RAD_HIDE_THINKING=1`. Added unit tests in `reasoning.rs`. Verified all workspace tests, zero-warning clippy audit, and clean `./scripts/build_all.sh` installation.

---

## 🛠️ Short-Term Plan: Phase 61

### 💡 Current AWU Status
- [✅] AWU 936: Fix False-Positive `[FAILED]` Tool Initialization Status for Non-MCP Extensions (`skill-tool-provider`) (Result: Success)

### 📝 AWU Details

#### AWU 936: Fix False-Positive `[FAILED]` Tool Initialization Status for Non-MCP Extensions (`skill-tool-provider`)
- **Trigger**: Launching `rad` without any custom skills defined in `.agents/skills` or `~/.rad/skills` outputs misleading `[FAILED] Extension 'skill-tool-provider' initialized with 0 tools. See [MCP Diagnostic] lines above for the actual cause.`, mistaking empty skill discovery for a failed MCP initialization.
- **Scope**: `src/orchestrator/runner/runtimes.rs`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - `runtimes.rs` distinguishes between `mcp-tool-provider` (where 0 tools indicates MCP server failure/misconfiguration) and other tool-provider extensions like `skill-tool-provider` (where 0 tools is a normal state when no skills are defined).
  - Non-MCP tool provider extensions initializing with 0 tools print `[OK] Extension '<name>' initialized with 0 tools` in green instead of `[FAILED]` in red with MCP diagnostic hints.
  - All workspace tests and `cargo clippy --workspace -- -D warnings` pass cleanly.
- **Result**: Success. Updated `src/orchestrator/runner/runtimes.rs` to check `ext.name == "mcp-tool-provider"` before reporting a 0-tool `[FAILED]` error with MCP diagnostics. Non-MCP tool providers like `skill-tool-provider` now output `[OK] Extension 'skill-tool-provider' initialized with 0 tools` when no local skills are present. Verified clean `cargo check --workspace`, zero-warning `cargo clippy --workspace -- -D warnings`, and full workspace test suite pass.


### 💡 Current AWU Status
- [✅] AWU 935: Comprehensive Re-verification of Basic Architecture Design & Implementation (Result: Success)

### 📝 AWU Details

#### AWU 935: Comprehensive Re-verification of Basic Architecture Design & Implementation
- **Trigger**: User requested a fundamental re-verification starting from the basic architecture design (`ARCHITECTURE.md` and related design policies).
- **Scope**: `ARCHITECTURE.md`, `wit/rad.wit`, `src/`, `ext/`, `models/`, `scripts/`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - Perform thorough verification comparing basic architecture specification (`ARCHITECTURE.md`) against actual codebase implementation.
  - Run mechanical audit checks (`cargo check`, `cargo clippy --workspace -- -D warnings`, `cargo test`, `./scripts/build_all.sh`).
  - Document verification findings, structural compliance, and any discrepancies or confirming evidence in a structured report.
- **Result**: Success. Verified strict alignment between `ARCHITECTURE.md` specification and physical implementation across all 6 Wasm extension crates, Trait-based Core subsystems, IPC contract in `wit/rad.wit` (27 RPC variants, zero legacy artifacts), process group (PGID) management, single capability configuration (`~/.rad/config.json`), and error handling workflows. Mechanical audit passed: `cargo check --workspace` clean, `cargo clippy --workspace -- -D warnings` clean, full workspace test suite passed cleanly.

---

## 🔭 Phase 46–51 Scope Notes (Planned, Not Yet Started)

Captured from a 2026-07-27 design review of local-LLM context-overflow prevention, which
expanded into a broader architecture/technical-debt pass (`/llm` command design, RPC
contract duplication, `context-tools` WIT scope, slash-command extensibility). Recorded here
at roadmap granularity; each phase gets its own AWU-numbered "Short-Term Plan" section with
Trigger/Fix/Result detail once work actually starts on it.

### Phase 46: RPC Contract Consolidation & `context-tools` WIT Unification — ✅ complete
*(46-1/46-3/46-4 shipped as AWU 915; 46-2 shipped as AWU 916 — see "Short-Term Plan: Phase 46" below.)*

### Phase 47: Slash Command Registry Redesign — ✅ complete
- **Context**: A `/llm` UX review (positional-arg-only `add`, reserved subcommand names shadowing profile names, no `delete`, inconsistent help text) traced back to a structural issue in `src/command.rs` itself: `Command` enum, `CommandParser::parse`, `CommandManager::execute`, the hand-written `/help` text, and `CommandHelper`'s tab-completion list were **5** independently-maintained sources of truth for "what commands exist" (the 5th, tab-completion, didn't even know `/llm` existed — a live bug). Compared against pi-coding-agent's `registerCommand(name, {description, handler})` unified-registry design.
- **47-1 — done (AWU 917)**: Refactored `src/command.rs`'s built-ins onto a single `CommandSpec` registry generating the parser, dispatcher, `/help` text, and tab-completion from one table.
- **47-1b — done (AWU 918)**: Applied the same registry pattern recursively to `/llm`'s own subcommands, fixing the residual issues 47-1 didn't touch. Also uncovered and fixed a real bug where testing it clobbered a developer's real `~/.rad/config.json`.
- **47-2 — done (AWU 919)**: Markdown-template-based lightweight command tier — see "Short-Term Plan: Phase 47" below.
- **47-3 — done (AWU 919)**: Extension loading switched from lazy to eager-at-startup.
- **47-4 — deliberately not implemented**: Extension-provided slash commands via a WIT export. Re-evaluated after 47-2 shipped, per this item's own stated condition: no current extension (rad-orchestrator/security-guard/mcp-tool-provider/llm-connector/context-tools) has a concrete need for a UI-facing command, and markdown templates already cover the actual demand this was meant to serve (user-defined prompt shortcuts). Building the WIT plumbing now would be speculative infrastructure with no consumer — revisit only if/when a real extension needs to do more than expand into a task prompt (e.g. programmatic output bypassing the LLM).
- **47-5 — done (AWU 924, Phase 52)**: Broader review of the slash-command *composition* itself, revisited at user request once Phase 51 wrapped. See Phase 52 below.

### Phase 48: Security Policy & Extension Config Hygiene — ✅ complete
*(48-1 and 48-2 shipped together as AWU 920, since the fix for one is the mechanism for the other — see "Short-Term Plan: Phase 48" below.)*
- **48-1 — done (AWU 920)**: `ext/security-guard/src/lib.rs::verify_rpc` hardcoded demo-fixture policy (`"blocked.txt"`/`"blocked_command"` string literals) with no config-driven policy mechanism at all. Replaced with a policy fetched from the extension's own config.
- **48-2 — done (AWU 920)**: `ExtensionConfig.config: HashMap<String, serde_json::Value>` (`src/config.rs`) is now actually read at runtime — via a new `GetExtensionConfig` RPC that `security-guard` calls to resolve its own blocklist patterns.

### Phase 49: Context-Overflow Feature Completion — ✅ complete
*(49-1 shipped as AWU 921; 49-2 deliberately not implemented — see "Short-Term Plan: Phase 49" below.)*
- **49-1 — done (AWU 921)**: Manual `context_length` override for backends `detect_context_length` can't probe — shipped as `/llm context <n>` atop Phase 47's registry.
- **49-2 — deliberately not implemented**: Exact token counting via llama.cpp's `/tokenize` endpoint. Would only cover llama.cpp (Ollama has no equivalent endpoint), making context-window budgeting backend-inconsistent — some profiles precise, others still approximated — for a benefit the existing chars/4 approximation already covers safely (it reserves a 10% margin, so the failure direction is under- not over-using the budget). Also adds an HTTP round-trip to every single `optimize` call on the hot path. Revisit only if the chars/4 approximation is observed to actually cause overflow in practice.

### Phase 50: Session Storage Operational Hygiene — ✅ complete
*(50-1 shipped as AWU 922 — see "Short-Term Plan: Phase 50" below.)*
- **50-1 — done (AWU 922)**: `.rad/sessions/<timestamp>.json` had no rotation/cleanup mechanism; files accumulated unbounded (150+ observed in this repo's own `.rad/` during investigation).

### Phase 51: Advanced Context Compression Techniques — ✅ complete
*(51-1/51-2/51-3 shipped as AWU 923; 51-4 deliberately not implemented — see "Short-Term Plan: Phase 51" below.)*
- **51-1 — done (AWU 923)**: Size-aware tool-result clearing, extending `context-tools.optimize`'s existing request/response (no parallel mechanism, no new WIT field).
- **51-2 — done (AWU 923)**: Lexical relevance-based retention (keyword overlap with the goal) reinstating earlier turns windowing would otherwise drop, when size budget leaves slack.
- **51-3 — done (AWU 923)**: Deterministic structured-fact digest (files touched, commands run) extracted from tool-call metadata in `rad-orchestrator`, attached to the system prompt so it survives windowing/clearing unconditionally.
- **51-4 — deliberately not implemented**: LLM-based recursive summarization (MemGPT-style). The scope note's own framing — "last resort only" — is the reason not to build it yet: 51-1/51-2/51-3 just shipped and haven't been exercised in real usage, so their effectiveness at the stated goal (preventing local-LLM context overflow) is unproven. Adding a mechanism that consumes the same constrained local-LLM compute the user is trying to protect (a nested summarization call competing with the primary task, on the very hardware this whole effort was scoped around) is a meaningfully larger commitment than 51-1–51-3 combined, and premature before knowing whether the cheaper techniques already solve the problem. Revisit only if real usage shows the cheaper techniques still overflow.

### Phase 52: Slash Command Composition Review, Role-Based Extension Lookup & Manual Compaction — ✅ complete
*(All shipped together as AWU 924, since `/compact`'s persistence step is what motivated the role-based lookup fix — see "Short-Term Plan: Phase 52" below.)*
- **Context**: User asked to revisit the deferred 47-5 composition review now that Phase 51 shipped, this time compared directly against pi-coding-agent's built-in slash-command set (fetched from its official docs) rather than in the abstract. That comparison surfaced concrete, externally-validated findings: pi merges what rad split across `/status`+`/session` into one `/session` command; pi has no `/clear` at all; pi's session-reset command is named `/new`, not `/reset`; and pi has a manual `/compact` with no rad equivalent — directly relevant given this whole session's context-overflow focus.
- **Command changes**: `/session` and `/status` merged (removed `/status`, `/session` now shows the full former-`/status` output — matches pi's own `/session` semantics: "session file, ID, messages, tokens, and cost"). `/clear` removed (no pi precedent, and terminals already provide Ctrl+L). `/reset` renamed to `/new`.
- **Extension-provided slash commands, reconsidered and re-confirmed not needed**: designing `/compact` surfaced that its logic is reachable entirely through the *existing* `CallExtension` RPC — proof that "host command delegating to an extension via RPC" already covers the use case a dedicated extension-owned-command mechanism would exist for. 47-4's original call stands.
- **Role-based `CallExtension` lookup**: `call-extension-payload.extension-id` now resolves by the target's declared `role` (`ExtensionConfig.role`) instead of its literal name, via a new `Orchestrator::find_extension_arc_by_role` (reads the role from static config rather than locking every candidate `WasmRuntime`, avoiding the nested-lock pattern AWU 900 fixed away from). A user can now swap in their own compatible `context-tools` replacement under any name without breaking the automatic per-turn optimize call or `/compact`. `wit/rad.wit`'s payload doc comment updated; no wire-schema change.
- **`/compact`**: new host-side command — no new WIT surface, since `rad-orchestrator`'s world only exports `on-event` (no generic "invoke me now" entry point the host could call on demand). Implemented entirely in `src/command/compact.rs`: walks the DAG natively (host already holds it directly, no RPC), calls `context-tools.optimize` the same way `rad-orchestrator` does (`WasmRuntime::call_extension_method`, found via the new role-based lookup), diffs surviving vs. dropped node IDs from the response, and persists the result via `Dag::merge_nodes` — split into one `merge_nodes` call per *contiguous* dropped run (Phase 51-2's relevance retention can make the dropped set non-contiguous, and `merge_nodes` isn't designed to collapse a non-contiguous list correctly). Failures surface as an explicit message (unlike the automatic path's silent fallback, appropriate for a deliberately user-triggered action).
- **`merge_nodes` bug fix (found while building `/compact`)**: previously reassigned `current_node_id` to the new merge node *unconditionally*, even when merging a historical (non-tip) range — would have silently rewound the active conversation pointer. Fixed to mirror `delete_node`'s existing conditional-clear: only follow the pointer if it was actually pointing at one of the merged nodes. No prior consumer of `merge_nodes` existed (`/compact` is the first), so this was a safe, uncontested correction rather than a behavior change with a dependent to worry about.
- **Scope**: `src/command.rs`, `src/command/handlers.rs`, `src/command/compact.rs` (new), `src/command/compact/tests.rs` (new), `src/orchestrator/runner/runtimes.rs`, `src/orchestrator/runner/runtimes/tests.rs` (new), `src/wasm/rpc_meta.rs`, `ext/rad-orchestrator/src/llm.rs`, `models/src/dag.rs`, `src/dag/tests.rs`, `wit/rad.wit`, `tests/command_tests.rs`, `README.md`, `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. New tests: 2 `merge_nodes` `current_node_id` tests, 1 role-based-lookup test (registers a `context-tools`-role extension under a different name and confirms resolution), 3 `/compact` tests (too-few-messages no-op, missing-extension error, and a full end-to-end run through the real `context_tools.wasm` that actually reduces DAG node count). `./scripts/build_all.sh` clean end-to-end.

---

## 🛠️ Short-Term Plan: Phase 59

### 💡 Current AWU Status
- [✅] AWU 934: Documentation & Template Alignment with Current Codebase Architecture (Result: Success)

### 📝 AWU Details

#### AWU 934: Documentation & Template Alignment with Current Codebase Architecture
- **Trigger**: User requested an audit to ensure `README.md`, `ARCHITECTURE.md`, `EXTENSIONS.md`, and all project documentation are fully consistent with the codebase.
- **Audit Findings**:
  1. `README.md` §4.3 references `cd ext/openai-orchestrator` (renamed to `ext/rad-orchestrator`).
  2. `ARCHITECTURE.md` §1 diagram omits `skill-tool-provider` and `context-tools` micro-extensions; §3.2.3 `ras-rpc-command` variant listing omits `list-dir`, `get-active-llm-profile`, `get-extension-config`, `generate-llm-stream`, `call-extension`, `log-traced-event`; §4.2 example config references obsolete extension names (`openai-orchestrator`), obsolete `wasm32-wasip1` target, and obsolete `"rpc_allow"` permission.
  3. `EXTENSIONS.md` §2 claims single `rad-extension` world (now multiple worlds: `rad-extension`, `rad-orchestrator`, `rad-security-guard`, `rad-tool-provider`); §3 lists deleted variants `spawn-mcp-server / send-mcp-request` and omits newer variants; §4 lists obsolete `"rpc_allow"`; §5 step 4 recommends `wasm32-wasip1` instead of `wasm32-wasip2`.
  4. Template WIT files (`templates/rust/wit/rad.wit`, `templates/go/wit/rad.wit`) contain deleted `spawn-mcp-server` variant and lack `list-dir`, etc.
  5. Template `README.md` files (`templates/rust/README.md`, `templates/go/README.md`) reference `wasm32-wasip1` target and `"rpc_allow"`.
- **Scope**: `README.md`, `ARCHITECTURE.md`, `EXTENSIONS.md`, `templates/rust/README.md`, `templates/rust/wit/rad.wit`, `templates/go/README.md`, `templates/go/wit/rad.wit`, `PLANS.md`.
- **Definition of Done (DoD)**:
  - All identified documentation discrepancies resolved.
  - Template `rad.wit` files synchronized with `wit/rad.wit`.
  - All workspace tests and `./scripts/build_all.sh` pass cleanly without errors or Clippy warnings.
- **Result**: Success. Updated `README.md`, `ARCHITECTURE.md`, and `EXTENSIONS.md`. Synchronized WIT contracts in `templates/rust/wit/rad.wit` and `templates/go/wit/rad.wit` with `wit/rad.wit`. Updated Rust and Go template READMEs to reference `wasm32-wasip2` and removed obsolete `rpc_allow`. Verified compilation of `templates/rust` (`cargo check --target wasm32-wasip2`), clean workspace tests (`cargo test --workspace -- --test-threads=1`), and Clippy audit (`cargo clippy --workspace -- -D warnings`).

---

## 🛠️ Short-Term Plan: Phase 58

### 💡 Current AWU Status
- [✅] AWU 933: Skills — Autonomously-Discoverable Markdown Tool Definitions (Result: Success)

### 📝 AWU Details

#### AWU 933: Skills — Autonomously-Discoverable Markdown Tool Definitions
- **Trigger**: User asked how `rad` handles Skills/Workflows (a concept from Claude Code and other agent tools) — confirmed via grep that no such feature exists, and that a *fictional* version of it was previously purged from docs in AWU 926. Follow-up discussion converged on a concrete design: unlike `.agents/commands/` (explicit `/name` invocation), Skills should be discoverable the same way any other tool is — the model sees a description in the tool list and chooses to invoke it autonomously.
- **Design pivot during planning**: the first design considered was fully host-native (`.agents/skills/` scanned directly in `src/wasm/rpc_meta.rs`'s `GetTools`/`ExecuteTool` handlers, no new WIT/RPC surface at all). The user pushed back: this would have repeated the exact anti-pattern just removed in AWU 932 (Core hosting a "tool" directly, contradicting README.md's "no built-in tools of its own" invariant from Phase 19/20). Re-designed as a genuine WASM extension (`skill-tool-provider`) implementing the existing `rad-tool-provider` WIT world — architecturally identical to `mcp-tool-provider`, just backed by local Markdown files instead of MCP servers.
- **New host primitive — `ListDir`**: extensions could already read one known file (`FileRead`/`open-file`) but had no way to enumerate a directory's entries (`.agents/commands/` only worked host-side because that feature runs as trusted Core code with direct `std::fs::read_dir` access — extensions have no equivalent). Since "drop a folder in, get auto-discovered" is core to the UX, added `RasRpcCommand::ListDir { path }` — a minimal, `FileRead`-symmetric primitive (`models/src/lib.rs`, `wit/rad.wit`'s `list-dir(string)` variant, both `rpc_conversion` macros, `FsSubsystem::dir_list`/`FsSandbox::dir_list` reusing `file_read`'s exact `fs_read_allow` permission-check pattern, `rpc_fs.rs` handler, `permissions.rs` check) rather than anything Skills-specific — any future extension needing directory enumeration can reuse it.
- **`skill-tool-provider` (`ext/skill-tool-provider/`, new crate)**: discovers `.agents/skills/<name>/SKILL.md` (project-local, checked first) then `~/.rad/skills/<name>/SKILL.md` (user-global) — same precedence direction and first-seen-wins dedup as `src/command/templates.rs`'s custom slash commands, implemented via `ListDir` + `FileRead` instead of direct `std::fs` calls. Each `SKILL.md` is a `---`-delimited frontmatter block (hand-rolled `key: value` parser, no new YAML dependency — the schema is a handful of scalar fields) plus a body: `description` (required — a skill missing it is logged and skipped, not fatal to the rest), `mode` (reserved for a future nested-task/subagent execution mode; defaults to `inline`, and specifying `subagent` now returns a clear "not implemented" error rather than silently running inline or being ignored), `allowed_tools` (reserved, parsed but unenforced). Invocation substitutes an optional `args` string via the same `$ARGUMENTS`-or-append rule as custom commands. `execute_tool`'s inline-mode result has to come back through the WIT `execution-handle` return type, so — matching `mcp-tool-provider`'s own pattern — it's produced via `open_process(echo ...)`, which means the extension needs bash execution permission even though it never runs arbitrary commands itself.
- **Bug found and fixed along the way**: verifying this via a direct (non-LLM) integration test surfaced that `WasmRuntime::execute_tool` (`src/wasm.rs`) had become **dead code returning nonsense** — it called into the extension's `execute-tool` export but then returned `res.rep().to_string()` (the WASM resource table's raw internal index, e.g. `"0"`) instead of resolving the handle to actual process output. Its only caller had been `rpc_meta.rs`'s `ExecuteTool` RPC handler, which AWU 932 correctly identified and removed as unreachable — but that inadvertently orphaned this method too, since nothing else ever called it (the real production path is `imports_tool.rs`'s WIT-import-level `execute_tool`, which properly transfers the resource handle to the caller for `rad-orchestrator`'s `execute_tool_sync` to resolve). Removed the dead method; rewrote the test to exercise the real path instead (full `Orchestrator` + mocked LLM tool call, matching `tool_loop_tests.rs`'s pattern) rather than calling a method that never actually worked correctly for this.
- **Scope**: `ext/skill-tool-provider/` (new crate: `Cargo.toml`, `src/lib.rs`, `src/skill.rs`, `src/skill/tests.rs`), `Cargo.toml` (workspace members), `scripts/build_all.sh`, `models/src/lib.rs`, `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `wit/rad.wit`, `src/subsystems.rs`, `src/fs.rs`, `src/wasm/{rpc_fs,rpc,permissions,wasm}.rs`, `tests/skill_tool_provider_tests.rs` (new), `CONFIG.md`, `README.md`, `ARCHITECTURE.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2` (including the new crate's own `#![deny(clippy::pedantic)]`); verified against the real installed `rad` binary with a real local LLM, not just mocked tests.
- **Result**: Success. 10 new unit tests in `ext/skill-tool-provider` (frontmatter parsing, `$ARGUMENTS` substitution, precedence ordering, missing-description skip) — genuinely runnable via plain `cargo test` despite the crate's WASM-only WIT imports, since the test binary never calls them. 4 new integration tests in `tests/skill_tool_provider_tests.rs` (discovery via a real compiled `.wasm`, full-Orchestrator execution round-trip, unknown-skill error, missing-description skip) exercising the real production paths for both halves of the feature. Full workspace test suite and Clippy clean on both targets. Real end-to-end verification against the actual installed `rad` binary and a real local LLM (`gemma-4-26B-A4B-it-qat`): the model autonomously selected the registered skill based purely on its tool description (no explicit naming in the prompt) and the exact `SKILL.md` body content came back as the tool result — confirmed after two earlier attempts where the same local model emitted malformed pseudo-tool-call text instead of a proper `tool_calls` JSON payload for this specific tool (a known local-model formatting quirk, not a `rad`-side defect — the integration tests independently prove the mechanism itself is correct regardless of any one model's tool-calling reliability).

---

## 🛠️ Short-Term Plan: Phase 57

### 💡 Current AWU Status
- [✅] AWU 932: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem (Result: Success)

### 📝 AWU Details

#### AWU 932: Legacy-Artifact Cleanup — Dead Built-in-Tool Fallback & Superseded MCP-Spawn RPC Subsystem
- **Trigger**: While designing a Skills feature (deferred — see below), a candidate design leaned on `src/wasm/rpc_meta_fallback.rs`'s "read"/"write"/"edit"/"bash" built-in-tool fallback as precedent for "Core can host tools directly." User asked to verify that precedent first. It turned out to directly contradict README.md's own documented invariant — "`rad` ships 5 extensions with no built-in tools of its own" (a Phase 19/20 decision) — and further investigation kept surfacing more dead code in the same shape, so the user asked to finish a full cleanup pass before returning to the Skills work.
- **Finding 1 — the built-in-tool fallback was genuinely unreachable in two different ways**:
  - `src/wasm/rpc_meta_fallback.rs` (reached via `RasRpcCommand::ExecuteTool` sent through generic `host_rpc`) had **zero callers anywhere** — no extension, no test, ever constructs and sends this command for execution (the one place it's constructed, `imports_tool.rs`, only builds a throwaway copy to describe the call to `verify_rpc_exclude` for security checks). Deleted entirely; `rpc_meta.rs`'s `ExecuteTool` match arm now just explains why the variant still exists (kept only because `imports_tool.rs`'s security-check construction and `security-guard`'s policy matching still reference the enum variant) and errors if ever actually dispatched.
  - `src/wasm/imports_tool.rs`'s *separate* fallback (reached via the real `execute-tool` WIT import that `rad-orchestrator` actually calls) **was** reachable, but only because three tests (`self_healing_tests.rs`, `self_healing_core_auto_tests.rs`, `hitl_tests.rs`, `tool_loop_tests.rs`) never registered a real tool-provider extension, so every "bash"/"write" tool call in their mocked LLM responses fell through to it by construction. Not a real product code path — no tool named `read`/`write`/`edit`/`bash` is ever advertised by `GetTools` in normal operation.
- **Finding 2 — a second, larger legacy subsystem for MCP servers, fully superseded**: `RasRpcCommand::SpawnMcpServer`/`SendMcpRequest`, their WIT payloads, the `RasCoreEvent::McpResponse` event, `PermissionConfig::allowed_mcp_servers`, and a dedicated `src/mcp.rs` (`McpProcess`, using plain `std::process::Command` with **no** PGID isolation, unlike the real process manager) — an entirely separate MCP-process-management path from the one `mcp-tool-provider` actually uses today (`RasRpcCommand::OpenProcess` + raw stdio piping, confirmed in Phase 55/AWU 930's work on that extension). Only `tests/mcp_tests.rs` ever exercised it. Independent corroboration found in this file's own AWU 926 entry, which had already flagged `allowed_mcp_servers` as a "wrong field name" removed from the *docs* back then — the code was simply never cleaned up to match. All of it removed: `src/mcp.rs` deleted; `RasRpcCommand::SpawnMcpServer`/`SendMcpRequest` and `RasCoreEvent::McpResponse` removed from `models/src/lib.rs` and their WIT variants/payloads from `wit/rad.wit`; `PermissionConfig::allowed_mcp_servers` removed (and its `/tools` display line in `src/command/tools.rs`); `active_mcp_servers` field removed from `WasmState`/`RpcContext`/`execute_rpc_command`'s threading; conversion arms removed from `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `src/wasm/bindings_event.rs`, `ext/rad-orchestrator/src/conv/event.rs`; stale copies of the removed WIT variants/event fixed in `ARCHITECTURE.md` (§3.2.3's reproduced `ras-rpc-command` listing, §4.1's PGID text, and the `RasCoreEvent` listing) plus a real, pre-existing PGID-isolation inaccuracy corrected in the same pass (the deleted `McpProcess` never actually called `setpgid`, contradicting what §4.1 claimed).
- **Bug found and fixed along the way**: `mcp-tool-provider`'s `get_tools()` (`ext/mcp-tool-provider/src/lib.rs`) pushed its `RAD_TEST_PORT` synthetic tools ("read"/"write"/"execute") but then **unconditionally** still called `init_mcp_servers()?` afterward, discarding those synthetic tools via `?` the instant no real `mcp_servers` config existed — silently defeating the whole point of test mode (self-contained, no real MCP servers needed) any time `get_tools()` (as opposed to `execute_tool()`, which was already correct) was the thing under test. Fixed by returning the synthetic tools directly, before ever touching real MCP discovery — this is what surfaced Finding 1's tests' true breakage once the dead fallback was gone (they'd been silently relying on it to paper over this bug too).
- **Test restructuring** (following from Finding 1): `tests/self_healing_tests.rs` deleted outright — its manual single-`WasmRuntime` driving style only worked because of two simultaneous dead-code shortcuts (no tool-provider needed thanks to the built-in fallback; no real `Orchestrator` needed because `ctx.orchestrator == None` routed `GenerateLlmStream` to a *third*, still-legitimate test-only fallback, `rpc_meta_llm_fallback.rs`, confirmed still load-bearing for `tool_loop_tests.rs` and kept as-is). Fixing the tool-provider lookup requires a real `Orchestrator`, which forecloses that LLM-mocking shortcut too — properly fixing it would have converged on exactly what `self_healing_core_auto_tests.rs` (kept) already tests, so it was redundant once fixed rather than worth reconstructing. `self_healing_core_auto_tests.rs`, `hitl_tests.rs`, and `tool_loop_tests.rs` were restructured to register a real `mcp-tool-provider` (in its self-contained `RAD_TEST_PORT` mode) alongside `rad-orchestrator`/`llm-connector`, using the `execute` test-tool (with real shell commands, e.g. `echo -n '...' > file`) instead of the no-longer-served `bash`/`write` names. `tests/mcp_tests.rs` (Finding 2) deleted outright — it exclusively tested the removed `SpawnMcpServer`/`SendMcpRequest` mechanism.
- **Flakiness found and fixed while restructuring** (not pre-existing regressions from this AWU, but surfaced by touching these files): `hitl_tests.rs` and `tool_loop_tests.rs` each run 2 `#[test]` functions that set process-global `RAD_YOLO`/`RAD_TEST_APPROVE`/`RAD_TEST_PORT` env vars — under `cargo test`'s default parallel execution these raced between threads in the same binary. Fixed with the same `static TEST_MUTEX: std::sync::Mutex<()>` pattern already established in `llm_command_tests.rs`, verified stable across repeated parallel runs. Separately, `self_healing_core_auto_tests.rs`'s 5s completion timeout became occasionally too tight after gaining a third registered extension (one more Wasm component to compile/instantiate on the self-healing retry); bumped to 8s and reverified stable across 5 consecutive runs.
- **Scope**: `src/mcp.rs` (deleted), `src/wasm/rpc_meta_fallback.rs` (deleted), `tests/mcp_tests.rs` (deleted), `tests/self_healing_tests.rs` (deleted), `src/lib.rs`, `src/wasm.rs`, `src/wasm/{rpc,rpc_meta,rpc_process,imports_rpc,imports_tool,permissions,bindings_event,loader}.rs`, `src/config.rs`, `src/command/tools.rs`, `models/src/lib.rs`, `models/src/rpc_conversion/{core_to_wit,wit_to_core}.rs`, `ext/rad-orchestrator/src/conv/event.rs`, `ext/mcp-tool-provider/src/lib.rs`, `wit/rad.wit`, `ARCHITECTURE.md`, `tests/{hitl_tests,tool_loop_tests,self_healing_core_auto_tests}.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; a real `rad` run against real MCP servers confirms no functional regression.
- **Result**: Success. Full workspace build/clippy clean on both targets. Full test suite green, including repeated-run stability checks for the two previously-racy files (5+ consecutive clean parallel runs each) and the retimed self-healing test. Verified end-to-end via the real installed `rad` binary against the real `core-utilities-mcp`/`web-access-mcp` servers (`~/projects/test`): a multi-tool-call task (directory listing, two file reads, a real shell `wc`-style tool call) completed correctly with 19 tools recognized at startup, confirming the real WIT-import-based tool-execution path is completely unaffected by removing the dead RPC-command-based one.

---

## 🛠️ Short-Term Plan: Phase 56

### 💡 Current AWU Status
- [✅] AWU 931: wasmtime Compile Cache & MCP Tool-List Caching (Result: Success)

### 📝 AWU Details

#### AWU 931: wasmtime Compile Cache & MCP Tool-List Caching
- **Trigger**: User asked where `rad`'s own architecture (as opposed to raw LLM
  inference latency, acknowledged as out of scope) could be sped up. A
  code-reading pass surfaced two candidates; both were verified empirically
  (not just reasoned about) before being scoped for a fix, and a couple of
  other candidates raised along the way were explicitly retracted or ruled
  out of scope once measured — see below.
- **Fix 1 — wasmtime compile cache**: any single extension crash triggers
  `clear_runtimes()` (`src/orchestrator/runner/runtimes.rs`), which discards
  **all 5** extension runtimes; the retry then recompiles all 5 from scratch
  via `WasmRuntime::new` (`src/wasm/loader.rs`), which builds a fresh
  `Engine` + `Component::from_file` with no caching. Measured with a
  throwaway benchmark (same wasmtime 29.0.1, same debug `.wasm` files
  `~/.rad/wasm` actually points at): **210ms cold, uncached**. Enabled
  wasmtime's built-in disk-based compilation cache
  (`Config::cache_config_load_default()`, gated behind the crate's `cache`
  feature — added to `wasmtime`'s features in `Cargo.toml`) in
  `WasmRuntime::new`; a load failure only logs via `log_host!` and falls
  back to always-recompile rather than blocking extension load, since
  caching is an optimization, not a correctness requirement. Deliberately
  **did not** pursue sharing a single `Engine` across extensions/self-heals
  (would have required changing `WasmRuntime::new`'s signature and touching
  11 call sites — 1 production + 10 tests): the benchmark showed
  `Engine::new()` itself is already near-zero cost after the first call
  in-process, and wasmtime's compile cache is a content-hash-keyed disk
  cache independent of which `Engine` instance requests the compile, so
  caching alone captures effectively all the available win without that
  blast radius.
- **Fix 2 — MCP tool-list caching**: `mcp-tool-provider`'s `get_tools()`
  (`ext/mcp-tool-provider/src/lib.rs`) issued a live, uncached `tools/list`
  to every configured MCP server on **every single call** — and it's called
  from both `trigger_llm_stream()` (every LLM turn, via
  `get_available_tools()` in `ext/rad-orchestrator/src/tool.rs`) and the
  host's `ExecuteTool` routing lookup (`src/wasm/rpc_meta.rs`, once per tool
  call, to find which extension owns a tool name). A raw JSON-RPC timing
  test against the real `core-utilities-mcp`/`web-access-mcp` binaries
  showed each server's own `tools/list` response is negligible
  (~0.06–0.08ms) — an earlier suspicion that `mcp_transport.rs::read_line`'s
  10ms poll-on-empty-read loop was adding latency here was **investigated
  and retracted**: the host-side stream read
  (`src/wasm/imports_resources.rs`) blocks on `mpsc::Receiver::recv()`
  rather than polling, so that path barely triggers in normal operation.
  Even so, the *design* was pure waste: `init_mcp_servers()`
  (`ext/mcp-tool-provider/src/client.rs`) now returns `Result<bool, String>`
  (`did_reinit` — true only when it actually had to (re)spawn a server, false
  when the existing connections' liveness check passed) instead of
  `Result<(), String>`; `get_tools()` skips the entire live-fetch loop and
  returns a cached `Vec<Tool>` (new `MCP_TOOLS_CACHE` static, alongside the
  existing `MCP_TOOL_MAPPING`) whenever `did_reinit` is false and the cache
  is populated, and `init_mcp_servers()` itself clears the cache whenever it
  does have to reinitialize. The live-fetch loop was extracted into a
  `fetch_and_cache_mcp_tools()` helper to keep `get_tools()` under the
  100-line Clippy pedantic limit. `mcp_transport.rs`'s two
  `init_mcp_servers()?` call sites needed no changes at all — both already
  discard the result as a bare statement, and `bool` isn't `#[must_use]`.
  Host-side `ExecuteTool`'s separate `get_tools()` call for routing was left
  untouched: with the extension-side cache in place it's now a cheap
  in-memory return, so restructuring host routing logic for a now-marginal
  gain wasn't worth the added risk.
- **Retracted/out-of-scope after investigation**: `read_line`'s 10ms poll
  loop (see above — real code path, but not the normal-operation bottleneck
  it first appeared to be); `main.rs`'s 50ms Esc/completion poll loop
  (estimated impact too small to matter); `security-guard`'s
  `verify_rpc_exclude` WASM-boundary crossing on every RPC (inherent
  sandboxing cost — removing it would weaken the security model for a small
  gain).
- **Scope**: `Cargo.toml`, `src/wasm/loader.rs`,
  `ext/mcp-tool-provider/src/client.rs`, `ext/mcp-tool-provider/src/lib.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass,
  native and `wasm32-wasip2`; both fixes verified with real measurements
  against the actual artifacts involved, not just reasoning from the code.
- **Result**: Success. Fix 1: re-measured with the same benchmark — cold
  compile of all 5 extensions went from 210ms to (with the cache warm)
  **~110ms** (a fresh from-scratch timing run measured 308ms cold → 107ms
  warm, ~65% reduction, stable across repeated runs; confirmed the on-disk
  cache directory `~/Library/Caches/BytecodeAlliance.wasmtime` was actually
  created and used). Fix 2: verified with temporary unconditional `eprintln!`
  markers (added, checked, then reverted — not part of the shipped diff)
  during a real multi-tool-call task via the installed `rad` binary against
  the real `core-utilities-mcp`/`web-access-mcp` servers: **1 live fetch
  followed by 7 cache hits** for a task with 3 tool calls (previously would
  have been 8 separate live `tools/list` round-trips). Full workspace test
  suite (native, including both self-healing tests) and Clippy clean on
  both targets; end-to-end task output unchanged/correct with the cache
  path active.

---

## 🛠️ Short-Term Plan: Phase 55

### 💡 Current AWU Status
- [✅] AWU 930: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider` (Result: Success)

### 📝 AWU Details

#### AWU 930: Adopt `rust-mcp-schema` for Typed MCP Protocol Messages in `mcp-tool-provider`
- **Trigger**: Continued dogfooding review of `ext/mcp-tool-provider/src/client.rs` found it hardcoded `protocolVersion: "2024-11-05"` — two spec generations behind — and still performed the `initialize`/`notifications/initialized` handshake that the 2026-07-28 MCP spec redesign removed entirely (stateless `server/discover` replaces it). All JSON-RPC request/response construction across the crate (`client.rs`'s handshake, `lib.rs`'s `tools/list` and `tools/call`) was hand-built via `serde_json::json!()` and parsed via manual `.get("...")` chains, with no compile-time guarantee of matching the real MCP schema. User asked for a fresh survey of the MCP-Rust-crate ecosystem (`rmcp`, gateway/aggregator tools like AgentGateway/MetaMCP, `rust-mcp-schema`) before deciding how to fix it.
- **Rejected alternatives**: `rmcp` (the official SDK) is Tokio-based and assumes direct OS access to spawn processes/open sockets — architecturally incompatible with `mcp-tool-provider`'s WASM-sandboxed, host-mediated-`host_rpc` transport model. MCP gateway/aggregator tools (AgentGateway, MetaMCP) solve a different-scale problem (multi-tenant aggregation behind one endpoint) than rad's single-user local use case, add operational complexity conflicting with rad's "single static binary, zero dependencies" philosophy, and don't even eliminate the need for an MCP client implementation since rad would still have to speak MCP to the gateway itself.
- **Fix**: adopted `rust-mcp-schema` (crates.io, MIT, transport-agnostic — just `serde` structs, no async runtime) for protocol *types only*, keeping the existing host-mediated process-spawn/stdin-stdout-pipe transport unchanged. Verified it builds cleanly for `wasm32-wasip2` with zero shim code needed. `client.rs`'s `init_mcp_servers` handshake now builds `ClientJsonrpcRequest::new(RequestId::String("init_1"), RequestFromClient::InitializeRequest(InitializeRequestParams { protocol_version: ProtocolVersion::latest().to_string(), .. }))` (currently resolves to spec `2025-11-25`, the newest the crate ships — still 2 generations ahead of the previous hardcoded `2024-11-05`) and `InitializedNotification::new(None)`, extracted into a new `perform_handshake` helper to keep `init_mcp_servers` under the 100-line Clippy pedantic limit. `lib.rs`'s `get_tools` builds `RequestFromClient::ListToolsRequest(None)` and parses the response via `serde_json::from_value::<ListToolsResult>`; `execute_tool` builds `RequestFromClient::CallToolRequest(CallToolRequestParams { name, arguments, .. })` and parses via `serde_json::from_value::<CallToolResult>`, extracting `ContentBlock::TextContent` variants. `RequestId::String(...)` (not `Integer`) was used throughout deliberately, to match `mcp_transport.rs::send_mcp_bytes`'s existing `req_val.get("id").and_then(|v| v.as_str())` response-correlation logic without needing to touch that file at all — the JSON-RPC wire format and transport layer stay exactly as they were; only the construction/parsing at the two call sites is now typed.
- **Scope**: `ext/mcp-tool-provider/Cargo.toml`, `ext/mcp-tool-provider/src/client.rs`, `ext/mcp-tool-provider/src/lib.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; verified end-to-end against the real installed `core-utilities-mcp` and `web-access-mcp` servers via the actual `rad` binary, not just automated tests.
- **Result**: Success. Full workspace test suite and Clippy (`#![deny(clippy::pedantic)]` in this crate) clean on both targets. Verified via two real `rad` runs from `~/projects/test` against the real running MCP servers: a `core-utilities-mcp` directory-listing call and a `web-access-mcp` HTTP-fetch call both round-tripped correctly end-to-end through the new typed handshake/list/call path (`[OK] Verified 18 tools from extension 'mcp-tool-provider'`).

---

## 🛠️ Short-Term Plan: Phase 54

### 💡 Current AWU Status
- [✅] AWU 926: MCP/LLM Onboarding Fixes & Documentation Consolidation (Result: Success)
- [✅] AWU 927: Fix `llm-connector` Eager-Load Stale-Environment-Snapshot Bug (Result: Success)
- [✅] AWU 928: Dedupe Repeated Inline Tool Calls from Local-Model Output (Result: Success)
- [✅] AWU 929: Restore Esc-to-Abort for Running Tasks (Result: Success)

### 📝 AWU Details

#### AWU 926: MCP/LLM Onboarding Fixes & Documentation Consolidation
- **Trigger**: User began real-world dogfooding (first actual `rad` run outside this repo, in a fresh `~/projects/test` directory) per the plan agreed at the end of Phase 53. Two real failures surfaced immediately.
- **Issue 1 — zero tools available**: `[FAILED] Extension 'mcp-tool-provider' get_tools error: load_mcp_config returned None (no mcp_servers config found)`. Root cause: post-Phase-19/20, `rad` has no built-in file/shell tools at all — MCP servers registered under `mcp-tool-provider`'s own `config.mcp_servers` are the *only* source of tools, and the user's `~/.rad/config.json` (assembled in Phase 48) never had any registered. **Fix**: registered the user's already-installed `core-utilities-mcp` and `web-access-mcp` binaries (found via `which`/`~/.cargo/bin`) in `~/.rad/config.json`'s `mcp-tool-provider.config.mcp_servers`. Verified via a real `rad` run: `[OK] Verified 17 tools`.
- **Issue 2 — stale/inaccurate documentation**: Investigating Issue 1 revealed `README.md`'s and `CONFIG.md`'s example configs were both stale relative to the real schema (fictional `builtin://` scheme, fictional `rad --update` flag, fictional `.rad/skills/`/`.rad/workflows/`/`extension.json` directory-packaged-extension mechanism — none implemented anywhere in the codebase, confirmed by exhaustive grep — plus wrong field names like `allowed_mcp_servers` and a wrong alternate-path claim `.rad/rad.json` instead of the real `.rad/config.json`). **Fix**: rewrote `CONFIG.md` as the single authoritative schema/discovery/directory-layout reference (real 5-extension example including `mcp_servers`, corrected precedence-cascade file paths, added previously-undocumented real features: `core.max_sessions`, `.rad/sessions/`, `~/.rad/wasm/`, `.agents/commands/`/`~/.rad/commands/` custom slash commands, `AGENTS.md` project rules). Trimmed `README.md`'s duplicate full JSON example down to a 2-line summary + pointer to `CONFIG.md`, per user's explicit "don't duplicate facts across files" direction — the duplication was the root cause of both files independently rotting out of sync in the first place. Removed the same fictional Skills/Workflows content from `README.md`'s feature list and `ARCHITECTURE.md` §5.4 (whose "Basic OS Primitives (Core)" item was independently stale for the same reason: no host-side tool primitives exist post-Phase-19/20).
- **Issue 3 — no LLM endpoint configured**: Task execution failed with `[LLM Connector Error] No LLM endpoint configured`. Root cause: `~/.rad/config.json`'s `llm.active`/`llm.endpoints` were still empty from Phase 48. Found a real `llama-server` already running locally (`qwen2.5-coder-32b-instruct-q4_k_m`, port 8080, `/props` reporting `n_ctx: 16384`) and registered it as the `local` profile. This surfaced Issue 4 below.
- **Scope**: `~/.rad/config.json` (not part of the git repo), `README.md`, `CONFIG.md`, `ARCHITECTURE.md`.
- **Result**: Success. Real `rad` run against the local `llama-server` confirmed tools + config load correctly.

#### AWU 927: Fix `llm-connector` Eager-Load Stale-Environment-Snapshot Bug
- **Trigger**: Directly following AWU 926's LLM endpoint fix, the *first* real task after starting `rad` failed with `[LLM Connector Error] No LLM endpoint configured`, self-healed (Wasm instance respawned), and the *second* attempt succeeded — a real, reproducible regression, not a one-off.
- **Root cause**: `WasmRuntime::new` (`src/wasm/loader.rs`) builds its WASI context via `WasiCtxBuilder::new().inherit_env()`, which snapshots the *host process's* environment variables once, at Wasm instance creation. `ext/llm-connector/src/connector.rs`'s `generate_stream` read its `base_url`/`api_key` from `LLM_BASE_URL`/`RAD_BASE_URL`/`OPENAI_BASE_URL`/`*_API_KEY` env vars — but those were only ever `std::env::set_var`'d inside `run_task_internal` (`src/orchestrator/runner.rs`), i.e. *after* a task starts. Phase 47-3 (AWU 919) made extension loading eager — at startup, before any task and before those `set_var` calls ever ran — so the very first `llm-connector` instance's WASI snapshot was permanently missing them; only a self-healing respawn (which recreates the instance *after* `run_task_internal` had already run once) picked up the correct values. The same staleness meant `/llm switch`/`/llm add`/`/llm model` mid-session would also silently not take effect until the next self-heal.
- **Fix (root cause, not a timing patch)**: extended `producer::generate-stream` in `wit/connector/llm-connector.wit` to take `base-url: option<string>` and `api-key: option<string>` as explicit call arguments instead of reading them from environment variables inside the guest at all. The host (`src/wasm/rpc_meta_llm_connector.rs`) now resolves the active profile fresh on *every* call via a new `resolve_active_llm_profile` helper in `rpc_meta.rs` (reading `orch.config.lock()` directly — no RPC round-trip needed, the host already holds the config) and passes `base_url`/`api_key` straight into the WIT call. This structurally eliminates the staleness vector — there's no snapshot to go stale, the values are fresh function arguments on every invocation — rather than just reordering `set_var` calls to happen earlier (which would only have fixed the startup case, not the mid-session `/llm switch` case). `resolve_active_llm_profile` deliberately isn't merged into the existing `GetActiveLlmProfile` RPC response (`active_llm_profile_json`): that RPC is generically reachable by any extension, and a credential doesn't belong in a broadly-readable fact query — `base_url`/`api_key` stay host-internal, passed only to the one extension that needs them, arguably *tighter* exposure than the old process-wide environment variables. The now-dead `unsafe { std::env::set_var(...) }` block in `run_task_internal` (`OPENAI_MODEL`/`LLM_MODEL` were already unused by anything, confirmed by grep) was removed entirely. `RAD_TEST_PORT` (test infrastructure, unrelated to this bug) was deliberately kept as an env-var override in `connector.rs`.
- **Scope**: `wit/connector/llm-connector.wit`, `ext/llm-connector/src/connector.rs`, `src/wasm/rpc_meta.rs`, `src/wasm/rpc_meta_llm_connector.rs`, `src/orchestrator/runner.rs`, `tests/llm_connector_eager_load_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`; a new regression test reproduces the exact eager-load ordering with a real (non-`RAD_TEST_PORT`) `llm.endpoints` config.
- **Result**: Success. New `tests/llm_connector_eager_load_tests.rs` proves the first task after eager loading succeeds outright (one connection attempt, not two). Verified against the real local `llama-server` too: previously 2 connection attempts (1 failure + 1 self-heal retry) before success, now exactly 1. Full workspace test suite (21 binaries) and Clippy clean on both targets; `./scripts/build_all.sh` clean end-to-end.

#### AWU 928: Dedupe Repeated Inline Tool Calls from Local-Model Output
- **Trigger**: Continued dogfooding surfaced a real task where the local `qwen2.5-coder-32b` model responded with the identical inline tool-call JSON (`{"name": "list_directory_contents", "arguments": {"path": "./"}}`) twice back-to-back in one turn's plain-text content — the preceding apology sentence appeared only once, so this was the model itself repeating just the JSON snippet (a known small/quantized-model quirk), not a streaming/duplication bug in `rad`'s own SSE handling. `parse_bare_json_tool_calls` (`ext/rad-orchestrator/src/orchestrator/runner/inline_tool_calls.rs`) scans the full text for every JSON-object-shaped occurrence and fires a separate tool call for each one found, so it faithfully — but unhelpfully — executed the identical call twice, producing two identical `[Tool Output]` blocks and likely contributing to the `LLM stream stalled` timeout that followed (the model's next turn had to make sense of a confusing duplicated tool-call/tool-response pair in its context).
- **Fix**: `push_tool_call` (the shared helper both the `call:`-prefixed and bare-JSON inline parsers funnel through) now skips pushing a call whose `(name, arguments)` exactly matches the *immediately preceding* pushed call in the same parse pass. Deliberately adjacent-only, not a full-history dedup: `assistant_tool_calls` is always empty when inline parsing starts (it only runs when the proper `delta.tool_calls` streaming mechanism produced nothing), so this can't cross-contaminate with unrelated real tool calls, and calling the same tool again later in the same turn with something else in between remains a legitimate, unaffected case.
- **Scope**: `ext/rad-orchestrator/src/orchestrator/runner/inline_tool_calls.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. 3 new tests: dedupes the exact reported case (adjacent identical calls collapse to 1), confirms non-adjacent repeats of the same call still both fire, confirms adjacent calls with *different* arguments still both fire. Full workspace test suite (21 binaries) and Clippy clean on both targets; `./scripts/build_all.sh` clean end-to-end.

#### AWU 929: Restore Esc-to-Abort for Running Tasks
- **Trigger**: User asked to be able to press Esc to interrupt a running task's thinking/work — mentioned in passing during AWU 927/928's dogfooding that they'd previously used Esc to stop a stuck task. The roadmap's own Phase 14 ("Esc Key Task Abort Polish & Robustness", v0.19.0) implies this once existed, but grepping the current codebase for `Esc`/`KeyCode`/`crossterm` turned up nothing at all in `src/` — `crossterm` is declared in `Cargo.toml` but was completely unused. `main.rs`'s `run_agent_task` blocks the main thread in a plain `while orchestrator.is_running() { sleep(50ms) }` loop while a task runs, and `rustyline`'s `Editor::readline()` — the only thing that would otherwise read stdin — isn't being called during that window. Conclusion: nothing currently reads stdin at all while a task is running, so no keypress (Esc included) could have reached any code; the earlier "I stopped it with Esc" was most likely the natural 15s stall timeout coinciding with the keypress, not a real abort path. This was a confirmed regression/dead-feature, not a verification task.
- **Design constraint discovered**: `crossterm::terminal::enable_raw_mode()` (the obvious approach) clears the *output* side of termios (`OPOST`/`ONLCR`) as well as input — but the background task thread concurrently `println!`s streamed LLM tokens via `src/terminal.rs`'s `TerminalController`, which relies on the terminal's automatic `\n` → `\r\n` translation. Enabling full raw mode while that's happening would turn concurrent output into a staircase mess.
- **Fix**: new `src/esc_abort.rs` module manipulates *only* the input side of termios directly via `nix::sys::termios` (already a dependency with the `"term"` feature) — clears `ICANON`/`ECHO` from `local_flags`, sets `VMIN`=0/`VTIME`=0 for a non-blocking-style `read()`, and leaves output flags untouched. `RawInputGuard` is an RAII guard restoring the original settings on drop; `esc_pressed(&RawInputGuard)` takes the guard by reference specifically so it can't be called without raw mode active (which would otherwise block waiting for a full line). `main.rs`'s `run_agent_task` poll loop now checks `esc_pressed` each 50ms tick and calls the pre-existing (already fully implemented) `orchestrator.abort()` on the first hit, routing the confirmation message through `TerminalController::write_log` (deferred like every other log during `Thinking`/`Streaming` state, so it surfaces cleanly once things settle back to `Idle` rather than interleaving with in-flight streamed tokens).
- **Scope**: `src/esc_abort.rs` (new), `src/lib.rs`, `src/main.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 2 new tests (`contains_esc` byte-detection logic, and confirming `RawInputGuard::enable()` gracefully returns `None` rather than hanging on the non-tty stdin `cargo test` itself runs under — exercises the real fallback path for free). Full workspace test suite and Clippy clean; `./scripts/build_all.sh` clean end-to-end. The actual interactive Esc keypress behavior needs a real TTY to verify (not reproducible via piped Bash-tool input), left for the user to confirm directly.

---

## 🛠️ Short-Term Plan: Phase 53

### 💡 Current AWU Status
- [✅] AWU 925: File-Size Limit Compliance Cleanup (Result: Success)

### 📝 AWU Details

#### AWU 925: File-Size Limit Compliance Cleanup
- **Trigger**: User asked, before starting real-world dogfooding, whether a comprehensive refactor was warranted. Assessed no (wait for real usage to surface concrete pain points first) but flagged 3 files that had drifted over CODING.md's 300-line limit as cheap, mechanical, zero-design-risk cleanup worth doing regardless: `models/src/rpc_conversion.rs` (370), `tests/multi_extension_tests.rs` (350, pre-existing since AWU 920), `tests/self_healing_tests.rs` (411). User agreed to clear them.
- **`models/src/rpc_conversion.rs`**: the two large per-direction command macros (`impl_rpc_command_wit_to_core!`, `common_rpc_command_core_to_wit!`) moved to new companion files `rpc_conversion/wit_to_core.rs` / `rpc_conversion/core_to_wit.rs`. Pure mechanical move — `#[macro_export]` always exports at the crate root regardless of which module a macro is textually defined in, so every one of the 5 consumer crates was unaffected (confirmed by full rebuild across all `wasm32-wasip2` extension crates with zero changes needed at any call site).
- **`tests/multi_extension_tests.rs`**: `test_multi_extension_isolated_roles` split into a new `tests/multi_extension_isolated_roles_tests.rs`. Each file's shared `run_mock_http_server`/`security_guard_config` helpers duplicated per-file (matching the precedent already set by `security_guard_policy_tests.rs`/`security_hook_tests.rs`) rather than factored into a shared `tests/common/` module — out of scope for a same-day mechanical split. `TEST_MUTEX` dropped from both resulting files: it only ever mattered for serializing tests sharing one process, and each file is now its own single-test binary.
- **`tests/self_healing_tests.rs`**: `test_core_auto_self_healing_integration` split into a new `tests/self_healing_core_auto_tests.rs`. Unlike the multi_extension split, the two original tests used entirely disjoint helpers (`MockNetwork`+direct `WasmRuntime` driving vs. a real mock HTTP server + full `Orchestrator`), so no duplication was needed at all — a completely clean cut.
- **Scope**: `models/src/rpc_conversion.rs`, `models/src/rpc_conversion/wit_to_core.rs` (new), `models/src/rpc_conversion/core_to_wit.rs` (new), `tests/multi_extension_tests.rs`, `tests/multi_extension_isolated_roles_tests.rs` (new), `tests/self_healing_tests.rs`, `tests/self_healing_core_auto_tests.rs` (new), `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; zero files over 300 lines anywhere in `src/`, `ext/`, `models/`, `tests/`.
- **Result**: Success. Verified with a full repo-wide line-count sweep (zero violations remaining) and `./scripts/build_all.sh` end-to-end.

---

## 🛠️ Short-Term Plan: Phase 52

### 💡 Current AWU Status
- [✅] AWU 924: Slash Command Composition Review, Role-Based Extension Lookup & `/compact` (Result: Success)

### 📝 AWU Details

#### AWU 924: Slash Command Composition Review, Role-Based Extension Lookup & `/compact`
See the Phase 52 roadmap scope note above for the full trigger/design/result narrative — recorded there rather than duplicated here since this AWU's work *is* Phase 52 in its entirety (no sub-items split across multiple AWUs).

---

## 🛠️ Short-Term Plan: Phase 51

### 💡 Current AWU Status
- [✅] AWU 923: Size-Aware Tool-Result Clearing, Lexical Relevance Retention & Structured-Fact Digest (51-1, 51-2, 51-3) (Result: Success)

### 📝 AWU Details

#### AWU 923: Size-Aware Tool-Result Clearing, Lexical Relevance Retention & Structured-Fact Digest (51-1, 51-2, 51-3)
- **Trigger**: User asked to execute Phases 49–51 in one pass following Phase 48's completion.
- **51-1 (tool-result clearing)**: Added `windowing::clear_stale_tool_results` in `ext/context-tools/src/windowing.rs`, run before windowing inside `optimize`. When the full message list exceeds `max_content_chars`, it replaces the content of older `tool`-role messages (oldest first) with a short placeholder — never removing the message itself, so the `tool_calls`/`tool` pairing invariant (AWU 909) stays intact — always preserving at least the single most recent tool result untouched. Reuses the existing `max_content_chars` field entirely; no new WIT surface.
- **51-2 (lexical relevance retention)**: Added `windowing::trim_with_relevance_retention`, layered on top of the existing positional windowing. After computing the standard "first + recent tail" cutoff, if `max_content_chars` leaves slack, earlier ("middle") turns are scored by keyword overlap with the goal message (simple lowercase-word-set intersection, no ML/embedding cost) and greedily reinstated highest-score-first until the budget is used. Turns are grouped so a `user` message and everything following it up to the next `user` message move as one unit — a reinstated turn can never split an `assistant`/`tool_calls` message from its `tool` reply. Purely additive: can only add back messages positional windowing already decided to drop.
- **51-3 (structured-fact digest)**: New `ext/rad-orchestrator/src/digest.rs` walks assistant `tool_calls` JSON (heuristically, via common argument key names like `path`/`file_path`/`command`/`cmd`, since this project has no built-in tools post-Phase-19/20 and MCP tool schemas vary) to build a deterministic "files touched / commands run" digest, capped to the most recent 30 items each. Appended to the system prompt in `load_messages_from_dag`, computed from the full pre-optimization message list. Since the system message is split out before `context-tools.optimize` and re-attached unconditionally after (pre-existing behavior), the digest survives regardless of how aggressively the rest of history gets windowed or cleared — no new plumbing needed for the "survives compaction" property.
- **File-size compliance**: `ext/context-tools/src/lib.rs`'s windowing logic moved to a new `ext/context-tools/src/windowing.rs` (+ companion `windowing/tests.rs`) to stay under 300 lines after 51-1/51-2 grew it past the limit; `lib.rs` now just wires `Guest::optimize` to `windowing::` calls. Similarly split `ext/rad-orchestrator/src/llm.rs`'s `CtMessage`/`CtOptimizationRequest`/`CtOptimizationResponse` wire structs into `ext/rad-orchestrator/src/llm/context_tools_wire.rs`.
- **Scope**: `ext/context-tools/src/lib.rs`, `ext/context-tools/src/windowing.rs` (new), `ext/context-tools/src/windowing/tests.rs` (new), `ext/context-tools/src/tests.rs`, `ext/rad-orchestrator/src/digest.rs` (new), `ext/rad-orchestrator/src/digest/tests.rs` (new), `ext/rad-orchestrator/src/llm.rs`, `ext/rad-orchestrator/src/llm/context_tools_wire.rs` (new), `ext/rad-orchestrator/src/lib.rs`, `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, native and `wasm32-wasip2`.
- **Result**: Success. `context-tools` unit tests grew from 10 to 13 (windowing-specific ones now live in `windowing/tests.rs`); `rad-orchestrator` gained 7 digest tests; full workspace `cargo test` (18 test binaries) and Clippy clean on both targets. Verified through the real WASM component boundary via `tests/context_tools_tests.rs` and the full `e2e_tests`/`tool_loop_tests` suites.

---

## 🛠️ Short-Term Plan: Phase 50

### 💡 Current AWU Status
- [✅] AWU 922: Session File Rotation (50-1) (Result: Success)

### 📝 AWU Details

#### AWU 922: Session File Rotation (50-1)
- **Trigger**: Phase 50 item 50-1 — `.rad/sessions/<id>.json` files had no cleanup mechanism at all; nothing ever deleted them.
- **Fix**: Added `CoreConfig.max_sessions: usize` (default 50, `~/.rad/config.json`-configurable) and `session::prune_sessions(workspace, keep, keep_id)` in `src/session.rs`, called once at startup from `src/startup.rs::load_config_and_session` right after the session ID and DAG are resolved. Deletes session files beyond the `keep` most-recently-modified ones, always excluding the current session's own file regardless of its age (it may be an explicitly `--session`-resumed old session about to be written to again). Best-effort: I/O errors (missing directory, permissions) are silently ignored rather than surfaced, since pruning is hygiene, not a startup precondition.
- **Scope**: `src/config.rs`, `src/session.rs`, `src/startup.rs`, `tests/command_tests.rs`, `tests/llm_command_tests.rs`, `tests/git_autopilot_tests.rs`, `tests/stabilization_tests.rs`, `tests/self_healing_tests.rs` (the last 4 needed a `..Default::default()` addition to their manually-constructed `CoreConfig` literals after the new field landed).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 4 new unit tests in `src/session.rs` (keeps-newest-N, never-deletes-current, no-op-under-limit, no-op-on-missing-directory); full workspace test suite (18 binaries) and Clippy clean.

---

## 🛠️ Short-Term Plan: Phase 49

### 💡 Current AWU Status
- [✅] AWU 921: `/llm context <n>` Manual Context-Length Override (49-1) (Result: Success)

### 📝 AWU Details

#### AWU 921: `/llm context <n>` Manual Context-Length Override (49-1)
- **Trigger**: Phase 49 item 49-1 — some backends (plain OpenAI-compatible proxies with neither `/props` nor `/api/show`) never get a detected `context_length`, leaving size-based windowing permanently disabled for them with no way for the user to supply the real number.
- **Fix**: Added a `context` entry to `llm_subcommand_specs()` (`src/command/llm.rs`) and `set_manual_context_length` (`src/command/llm/profile_admin.rs`), following the exact registry pattern AWU 918 established — no new dispatch mechanism. Sets `context_length` on the active profile and persists via the existing `save_global_config`. Like every other detected value, it's overwritten again the next time real detection succeeds (`/llm add`/`/llm test`/`/llm model`), since a fresh real reading always wins over a manual guess.
- **Scope**: `src/command/llm.rs`, `src/command/llm/profile_admin.rs`, `tests/llm_command_tests.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. 3 new tests (sets override, rejects non-numeric input, errors with no active profile); full test suite and Clippy clean.

---

## 🛠️ Short-Term Plan: Phase 48

### 💡 Current AWU Status
- [✅] AWU 920: Config-Driven `security-guard` Policy via New `GetExtensionConfig` RPC (48-1 & 48-2) (Result: Success)

### 📝 AWU Details

#### AWU 920: Config-Driven `security-guard` Policy via New `GetExtensionConfig` RPC (48-1 & 48-2)
- **Trigger**: User asked to proceed with Phase 48 after Phase 47 wrapped. `ext/security-guard/src/lib.rs::verify_rpc` hardcoded its blocklist (`"blocked.txt"`/`"blocked_command"` literals) directly in the match arms, with no way for `~/.rad/config.json` to influence it — misleading for a component named "security-guard", and the reason `ExtensionConfig.config` (already parsed, tested, never read) existed as dead weight.
- **Design**: Rather than inventing a bespoke config-passing side channel, added a new generic RPC (`RasRpcCommand::GetExtensionConfig`) that lets any extension ask the host "what's in my own `ExtensionConfig.config`", the same "generic fact query, no special-casing" pattern AWU 915/916 established for `GetActiveLlmProfile`. This resolved 48-1 and 48-2 as one fix: the RPC *is* the wiring 48-2 asked for, and `security-guard` consuming it *is* 48-1's real policy mechanism.
- **Host plumbing**: Added `GetExtensionConfig` to `models/src/lib.rs`'s `RasRpcCommand` and `wit/rad.wit`; the Phase-46 shared macros (`rpc_conversion.rs`) picked up both directions with a 2-line addition each, confirming the consolidation's payoff. Threaded a new `caller_name: &'a str` field through `RpcContext` (`src/wasm/rpc.rs`) and its one construction site (`src/wasm/imports_rpc.rs`, passing `&self.name`), so `rpc_meta::handle_meta`'s new arm can look up `orch.config.lock().extensions.iter().find(|e| e.name == caller_name)` and return that extension's `config` map as JSON (empty object if unregistered or no orchestrator, e.g. the lower-level `WasmRuntime` unit-test harness).
- **`security-guard` fix**: Added `ext/security-guard/src/policy.rs` — a `Policy { block_path_patterns, block_command_patterns }` fetched once via `GetExtensionConfig` and cached in a `thread_local!` `RefCell` for the lifetime of the component instance (avoids a host round-trip on every `verify_rpc` call). Empty/missing config means "block nothing" — the policy is opt-in, not a hardcoded fallback. `lib.rs` gained the core-to-wit conversion direction it previously didn't need (`impl_rpc_target_core_to_wit!`, `impl_rpc_timeout_policy_core_to_wit!`, and a hand-written `From<CoreRpcCommand> for wit::RasRpcCommand` residual for `OpenFile`/`OpenProcess`, mirroring `rad-orchestrator`'s `conv/rpc.rs` pattern) since it now constructs a command to send, not just receives one.
- **Test fallout (3 files needed the same fix)**: Making the policy opt-in broke every existing test that relied on the old hardcoded literals with an empty `config: HashMap::new()` — `src/wasm/tests.rs` (`test_verify_rpc_blocked_file`, `test_verify_rpc_blocked_command`), `tests/multi_extension_tests.rs` (both tests), `tests/security_hook_tests.rs`. Fixed by explicitly configuring `block_path_patterns`/`block_command_patterns` on each security-guard-role `ExtensionConfig` — `src/wasm/tests.rs`'s harness needed a real (if minimal) `Orchestrator` wired into `WasmRuntime::new`'s `orchestrator: Option<Weak<Orchestrator>>` parameter for the first time, since `verify_rpc` now round-trips through host config lookup. Added one new test proving the opt-in direction too — `test_security_guard_blocklist_is_opt_in_and_blocks_nothing_when_unconfigured` — split into its own `tests/security_guard_policy_tests.rs` file rather than growing `multi_extension_tests.rs` further past CODING.md's 300-line limit (that file was already over at 336 lines pre-existing this AWU; left as a known, not-worsened violation rather than a full re-split, since restructuring its two already-passing integration tests was out of scope here).
- **Scope**: `models/src/lib.rs`, `wit/rad.wit`, `models/src/rpc_conversion.rs`, `src/wasm/rpc.rs`, `src/wasm/imports_rpc.rs`, `src/wasm/rpc_meta.rs`, `ext/security-guard/src/lib.rs`, `ext/security-guard/src/policy.rs` (new), `src/wasm/tests.rs`, `tests/multi_extension_tests.rs`, `tests/security_hook_tests.rs`, `tests/security_guard_policy_tests.rs` (new), `PLANS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass, for both the native host target and `wasm32-wasip2` extension crates.
- **Result**: Success. 46 `rad` lib unit tests + all integration test binaries (including the 3 security-related ones) pass; Clippy clean (`-D warnings`) on both native and `wasm32-wasip2` targets across `rad`, `rad-models`, `security-guard`, `rad-orchestrator`, `context-tools`, `llm-connector`, `mcp-tool-provider`.

---

## 🛠️ Short-Term Plan: Phase 47

### 💡 Current AWU Status
- [✅] AWU 917: Top-Level Slash Command Registry (47-1) (Result: Success)
- [✅] AWU 918: `/llm` Subcommand Registry (47-1b) & Fix Test Pollution of Real `~/.rad/config.json` (Result: Success)
- [✅] AWU 919: Markdown-Template Commands (47-2) & Eager Extension Loading (47-3) — Phase 47 Complete (Result: Success)

### 📝 AWU Details

#### AWU 917: Top-Level Slash Command Registry (47-1)
- **Trigger**: Phase 47 item 47-1. `src/command.rs`'s `Command` enum, `CommandParser::parse`, `CommandManager::execute`, and hand-written `/help` text were 4 independently-maintained sources of truth for the command list; a 5th (`CommandHelper`'s tab-completion array in `src/command/completion.rs`) was discovered while implementing this and had drifted furthest — it never listed `/llm` at all.
- **Fix**: Replaced the `Command` enum with `ParsedCommand { name: &'static str, args: String }` (canonical name + raw trailing text) and a `CommandSpec { name, aliases, description, handler: fn(&str, &Arc<Orchestrator>) -> CommandResult }` registry (`command_specs()`). `CommandParser::parse` now looks up the registry instead of a hand-written match; `CommandManager::execute` dispatches to `spec.handler`; `/help`'s text is generated by iterating `command_specs()`; `CommandHelper`'s completion list now iterates the same registry (including aliases), fixing the missing-`/llm` gap.
- **Incidental fixes surfaced while porting each handler** (preserved-behavior parity was the goal, but two handlers were clearly broken and got fixed since they were being rewritten anyway): `/session` previously required an argument and echoed it back verbatim (`/session foo` → "Current session: foo" regardless of `foo`'s value) instead of showing the actual session ID, and `/session` with no argument fell through to being sent to the LLM as a task; now shows the real `orchestrator.session_id` unconditionally. `/rollback` with no node ID previously also fell through to being sent to the LLM as a task instead of erroring; now shows a `Usage: /rollback <node_id>` message.
- **Explicitly not fixed by this AWU** (see the corrected Phase 47 scope note above): `/llm`'s own internal issues — positional-only `add`, `test`/`add`/`model` shadowing profiles of the same name, no `delete`, incomplete internal help in `render_llm_profiles` — are untouched; `cmd_llm` just delegates to the pre-existing `llm::parse_llm_command`/`LlmSubcommand` unchanged. Applying the same registry pattern to `/llm`'s own subcommands is separate, unstarted follow-up work.
- **Scope**: `src/command.rs` (rewritten), `src/command/completion.rs`, `src/main.rs` (one call-site signature update), `tests/command_tests.rs` (rewritten for the new `ParsedCommand`/registry API).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines.
- **Result**: Success. `src/command.rs` is 281 lines (was 236 — net growth from per-command handler functions replacing match arms, still under the 300-line limit). Host `cargo build`/`clippy --lib` clean. `command_tests.rs` rewritten and passing (3 tests, including a new completion assertion that `/llm`/`/models` are now suggested). Full suite verified: host `cargo test --lib` (40 tests) and integration suite (`command`/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 17 tests total) all pass with no regressions.

#### AWU 918: `/llm` Subcommand Registry (47-1b) & Fix Test Pollution of Real `~/.rad/config.json`
- **Trigger**: Phase 47 item 47-1b, per the corrected scope note left by AWU 917 (user explicitly asked for this residual work before the deferred broader command-composition review).
- **47-1b fix**: Replaced `LlmSubcommand`/`parse_llm_command`/`execute_llm_command` with an `LlmSubcommandSpec { name, usage, handler: fn(&str, &Orchestrator) -> String }` registry (`llm_subcommand_specs()`) and a single `execute_llm(args, orchestrator)` entry point, mirroring AWU 917's top-level pattern one level down. Fixed all four residual issues:
  1. **Reserved-keyword/profile-name collision**: `execute_llm` now checks whether the first word matches a *registered profile name* (or a numeric index) before checking the subcommand table, so a profile literally named `test`/`add`/`model`/etc. is switched to directly; `/llm switch <name>` remains an explicit, unambiguous escape hatch either way.
  2. **Added `/llm delete <name>`** (`delete_llm_profile` in `profile_admin.rs`) — removes a profile, falling `active` back to another remaining profile (or `None`) if the deleted one was active.
  3. **`add` switched from strict positional args to flags**: `/llm add <name> <url> [--model <m>] [--api-key <k>]`, fixing the inability to set `api_key` without `model`. Accepted as a breaking change to the old 4-positional form (pre-1.0 internal tool).
  4. **`render_llm_profiles`'s help footer now always lists every subcommand's usage** (generated from `llm_subcommand_specs()`), not just conditionally showing `add`'s usage when the endpoint list was empty.
- **Critical bug found while writing `tests/llm_command_tests.rs`**: `save_global_config` (in `profile_admin.rs`) unconditionally wrote to the *real* `dirs::home_dir()/.rad/config.json` with no test seam. The new tests — the first ones in this repo to exercise `/llm add`/`test`/`model`/`delete` — clobbered the developer's actual global config with tempdir workspace paths and dummy `test`/`primary` profiles pointing at `http://127.0.0.1:1`. Disclosed to the user immediately on discovery; **the original content could not be recovered (no backup existed)**.
- **Root-cause fix**: Added `crate::config::global_config_path()` in `src/config.rs` — the single shared resolver for `~/.rad/config.json`, honoring a `RAD_TEST_CONFIG_HOME` override — and routed both the read side (`load_config`'s three independent `dirs::home_dir()` call sites in `config/load.rs`) and the write side (`save_global_config`) through it. Previously these resolved the same conceptual path independently via separate `dirs::home_dir()` calls with no shared test seam at all. This also fixed a second, pre-existing latent bug it surfaced: `src/config/tests.rs`'s `test_load_config_default_when_no_file`/`test_load_config_with_local_override` had always implicitly depended on the real machine's `~/.rad/config.json` being absent-or-default (since `load_config` always reads it as a base layer, even with an explicit path given) — invisible until this session's corruption made `test_load_config_default_when_no_file` fail. Both tests now set `RAD_TEST_CONFIG_HOME` to a per-test tempdir, serialized via a new `CONFIG_TEST_MUTEX` (env vars are process-global; `cargo test` runs same-binary tests in parallel threads by default) — same pattern `tests/multi_extension_tests.rs` already used for its own global state.
- **New test coverage**: `tests/llm_command_tests.rs` (8 tests) — flag-based `add` with/without `model`, unrecognized-flag rejection, `delete` (success/not-found/active-profile-fallback), profile-name-vs-keyword priority (including the `switch` escape hatch), and the always-populated help footer. Every test holds a `TEST_MUTEX` (matching `multi_extension_tests.rs`) for its full duration and sets `RAD_TEST_CONFIG_HOME` to its own tempdir, so this suite cannot repeat the pollution incident.
- **Scope**: `src/command/llm.rs` (rewritten), `src/command/llm/profile_admin.rs` (`delete_llm_profile` added, `save_global_config` fixed), `src/config.rs` (`global_config_path` added), `src/config/load.rs` (3 call sites unified), `src/config/tests.rs` (2 tests made hermetic), `src/command.rs` (`cmd_llm` simplified), `tests/llm_command_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; the real `~/.rad/config.json` is verifiably untouched by a full test run (checksummed before/after).
- **Result**: Success. Host `cargo build`/`clippy --lib` clean. `src/config.rs` 203, `config/load.rs` 118, `config/tests.rs` 223, `command/llm.rs` 174, `command/llm/profile_admin.rs` 216, `tests/llm_command_tests.rs` 185 lines — all under the 300-line limit. Full suite: host `cargo test --lib` (40 tests, including the now-fixed `test_load_config_default_when_no_file`), and integration suite (`command`/`llm_command` (new, 8 tests)/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 25 tests total) all pass. `~/.rad/config.json`'s checksum confirmed identical before and after the full run (no further pollution) — but its content from before this incident remains lost.

#### AWU 919: Markdown-Template Commands (47-2) & Eager Extension Loading (47-3) — Phase 47 Complete
- **Trigger**: User asked to complete Phase 47 in full. Remaining items: 47-2 (markdown-template command tier), 47-3 (eager extension loading), and a decision on 47-4.
- **47-2 fix**: Added `src/command/templates.rs` — `expand(workspace, name, args)` looks up `<name>.md` in `.agents/commands/` (project-local, checked first) then `~/.rad/commands/` (user-global), substitutes a `$ARGUMENTS` placeholder if present, else appends `args` on a new line (or returns the template unchanged if `args` is empty and there's no placeholder). `list_names(workspace)` enumerates available template names for `/help`. Since Rust's static `command_specs()` function-pointer table can't hold entries discovered at runtime from the filesystem, this is a genuinely separate lookup tier rather than one literal array — `process_input` (`main.rs`) tries the built-in registry first, then `templates::expand` (via a new shared `command::split_slash` helper factored out of `CommandParser::parse`), before falling through to the pre-existing "send raw input as a task" behavior for a truly unrecognized `/whatever`. Added `CommandResult::RunTask(String)` as a first-class variant for "this command expanded into a task prompt" rather than piping it through a template-specific side channel. `/help` now also lists discovered template names.
- **47-3 fix**: `main()` now calls `orchestrator.get_or_init_runtimes(&throwaway_tx)` immediately after constructing the `Orchestrator`, instead of waiting for the first task or `/tools` to trigger it lazily. `get_or_init_runtimes` is idempotent (skips already-loaded extensions by name), so this doesn't change the real per-task load path that follows later (`runner.rs` still re-applies `set_event_tx` with the real per-task channel) — it just moves the one-time ~150ms-class cost earlier and surfaces a broken extension's diagnostics (e.g. an MCP tool-provider failing to find its server config) at startup instead of silently deferring them to the first time a task actually needs that extension.
- **47-4 decision**: Not implemented. Re-evaluated per this item's own stated condition (after 47-2 shipped) and found no concrete consumer: none of the 5 current extensions need a UI-facing command, and markdown templates already serve the demand this was meant to address (user-defined prompt shortcuts). Documented as deliberately deferred rather than built speculatively — revisit only when a real extension needs more than "expand into a task prompt."
- **Manual verification side-incident**: Testing `/hello world` (a throwaway template) triggered the orchestrator's own git-autopilot feature, which created and checked out a new branch (`rad-autopilot-1785133346`) carrying this session's entire uncommitted working tree with it. Confirmed via `git diff`/`git log` that the new branch had zero unique commits (identical tip to the working branch `rad-autopilot-1785051822`) before switching back and deleting the stray branch — no work was at risk, but this is a reminder that manually running `rad` for verification during a `rad`-development session can itself mutate git state.
- **Scope**: `src/command/templates.rs` (new), `src/command/templates/tests.rs` (new, 6 tests), `src/command.rs` (`CommandResult::RunTask`, `split_slash` extracted, handlers moved out), `src/command/handlers.rs` (new — built-in handler fns relocated here to stay under the 300-line limit after `templates` integration), `src/main.rs` (`process_input`'s template tier, `run_task_and_save` extracted, eager-load call in `main()`).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; manually verified end-to-end (`/help` lists a test template, `/name args` expands and runs as a task, all 5 extensions load at startup with diagnostics visible before the first task).
- **Result**: Success. `src/command.rs` 203, `src/command/handlers.rs` 112, `src/command/templates.rs` 71, `src/main.rs` 228 lines — all under the 300-line limit. Host `cargo build`/`clippy --lib` clean. Host `cargo test --lib` (46 tests, +6 new template tests) and integration suite (`command`/`llm_command`/`context_restoration`/`context_tools`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 25 tests) all pass. Manual verification: `rad` startup now shows `[FAILED] Extension 'mcp-tool-provider' get_tools error: ...` immediately (previously only visible after the first task); `/help` correctly listed a test `.agents/commands/hello.md` template under "Custom commands"; `/hello world` expanded to the template body with `$ARGUMENTS` substituted and ran as a real task through the full orchestrator pipeline. Test artifacts (`.agents/commands/hello.md`, the stray git branch) cleaned up after verification.

---

## 🛠️ Short-Term Plan: Phase 46

### 💡 Current AWU Status
- [✅] AWU 915: RPC Conversion Macro Consolidation, `check_endpoint()` Unification & State Constant Dedup (Result: Success)
- [✅] AWU 916: `context-tools` WIT Unification (46-2) — Discovered & Fixed a Silent Windowing Bug (Result: Success)

### 📝 AWU Details

#### AWU 915: RPC Conversion Macro Consolidation, `check_endpoint()` Unification & State Constant Dedup
- **Trigger**: Phase 46 of the 2026-07-27 long-term plan (items 46-1, 46-3, 46-4; 46-2 deferred as the next unit of work since it depends on 46-1).
- **46-1 fix**: Added `models/src/rpc_conversion.rs` with `#[macro_export]`'d `macro_rules!` definitions (`impl_rpc_target_wit_to_core!`/`_core_to_wit!`, `impl_rpc_timeout_policy_wit_to_core!`/`_core_to_wit!`, `impl_rpc_command_wit_to_core!`, `common_rpc_command_core_to_wit!`) generating the `RasRpcCommand`/`Target`/`TimeoutPolicy` wit↔core conversion boilerplate that was hand-copied across `src/wasm/bindings/rpc_command.rs`, `ext/security-guard/src/lib.rs`, `ext/mcp-tool-provider/src/conv.rs`, and `ext/rad-orchestrator/src/conv.rs`/`conv/rpc.rs`. `common_rpc_command_core_to_wit!` returns `Option` and omits `OpenFile`/`OpenProcess` (no WIT equivalent — host-only normalization variants), leaving that residual to each site (host converts for real; the 3 guest crates `panic!()`/`unreachable!()` since they never construct those). Each site now invokes only the macros for the direction(s) it actually needs (e.g. `security-guard` only wit-to-core, since `verify_rpc` inspects incoming commands but never constructs one). Hit and fixed one macro-hygiene issue mid-flight: `$wit:path` fragments produced "expected type, found module" errors when used as a path prefix (`$wit::Target`) in item position; switched to `$wit:ident` (every call site passes a single identifier — a local `use ... as wit` alias — so `path` fragment's extra generality wasn't needed and was the source of the parse ambiguity).
- **46-3 fix**: Added `EndpointStatus`/`check_endpoint()` in `src/command/llm/context_length.rs`, replacing the two previously-separate `probe_endpoint`(liveness)/`detect_context_length`(context window) call patterns. `check_endpoint` skips context-length detection when the liveness probe fails (avoids a second connect-timeout wait for a server already known down) and is now used uniformly by `main.rs`'s startup health check, `/llm add`, `/llm model`, and `/llm test` — closing the gap where startup never detected context length at all, and giving `/llm add`/`/llm model` a liveness signal they didn't previously have. `probe_endpoint`/`detect_context_length` are now private to `context_length.rs`.
- **46-4 fix**: Added `DEFAULT_MAX_HISTORY_MESSAGES`/`DEFAULT_MAX_TOOL_OUTPUT_CHARS` constants in `ext/rad-orchestrator/src/orchestrator.rs`, replacing the `Some(50)`/`Some(2000)` literals duplicated in `handle_human_input` and `handle_rehydrate`.

#### AWU 916: `context-tools` WIT Unification (46-2) — Discovered & Fixed a Silent Windowing Bug
- **Trigger**: Phase 46 item 46-2, now unblocked by AWU 915. Unify `context-tools`' bespoke single-variant `wit/context-tools.wit` (`package radcomp:context-tools`, `host-rpc` importing a fake `command(string)` "RPC") onto the shared `rad.wit` `radcomp:extension` package's real `ras-rpc-command`, so its declared `PermissionConfig.fs_read_allow`/`fs_write_allow` finally mean something instead of being silently ignored by a raw `sh -c` bridge.
- **WIT restructuring**: Changed `context-tools.wit`'s package to `radcomp:extension` and deleted its own `types`/`host-rpc` interfaces (reused rad.wit's). Since `wasm_bindgen`/`wasmtime::component::bindgen!`'s directory-scan mode requires every `.wit` file in a scanned directory to share one package (or use the `deps/` convention), and `wit/` also held `llm-connector.wit` (a third, unrelated `radcomp:connector` package), moved that file to `wit/connector/llm-connector.wit` — the two remaining call sites (`src/wasm/bindings.rs`, `ext/llm-connector/src/lib.rs`) updated to the new path. `rad_context_tools`'s host bindgen and the guest's `wit_bindgen::generate!` now both point `path` at the `wit` directory (not a single file) with `world: "context-tools-extension"`, plus a `with: { "radcomp:extension/types": ... }` mapping on the host side so its `RasRpcCommand` is the same Rust type every other extension's bindings use.
- **Host bridge**: Deleted the special-cased shell-command `impl ...host_rpc::Host for WasmState` in `src/wasm/imports_delegate.rs` entirely; `context-tools` now goes through `delegate_extension_imports!(bindings::rad_context_tools::ContextToolsExtensionImports, rpc_only)`, the same generic (permission-checked, security-guard-verified) `host_rpc` path every other extension uses.
- **`get-repo-map` upgrade**: Since `RasRpcCommand::GetRepoMap` (the real semantic/tree-sitter repo map, already used by other extensions) is now reachable, `MyContextTools::get_repo_map` calls `host_rpc(&RasRpcCommand::GetRepoMap)` instead of shelling out to `tree -L 2`.
- **Host-side call-site fallout**: `src/wasm.rs`'s `call_extension_method("optimize"/"get-repo-map")` updated from `rad_context_tools::exports::radcomp::context_tools::context_tools::...`/`.radcomp_context_tools_context_tools()` to `...::radcomp::extension::context_tools::...`/`.radcomp_extension_context_tools()` (package-rename fallout; `loader.rs`'s `ContextToolsExtension` struct name is derived from the world name, not the package, so it needed no change).
- **New integration coverage, and a real bug it caught**: No integration test had ever loaded `context-tools` as an extension before (confirmed by grep — `ext/context-tools`'s own tests only exercise guest-side logic natively, never through the real WASM component boundary). Added `tests/context_tools_tests.rs`, driving `context-tools.wasm` through `WasmRuntime::call_extension_method` directly. Writing it surfaced a **pre-existing, silent bug dating to AWU 908**: `ext/rad-orchestrator/src/llm.rs`'s `CtMessage`/`CtOptimizationRequest`/`CtOptimizationResponse` used `#[serde(rename = "node-id")]`-style kebab-case renames, on the incorrect assumption that the host's `additional_derives: [serde::Serialize, serde::Deserialize]`-generated structs serialize kebab-case (matching the WIT source spelling). They don't — wasmtime's component bindgen uses the plain (already-snake_case) Rust field names with no rename. Since `serde_json` silently ignores unrecognized JSON keys and defaults absent `Option<T>` fields to `None`, every `optimize` request `rad-orchestrator` ever sent had `max_history`/`max_content_chars`/`node_id` silently deserialize to `None`/`null` on the host side — **count-based windowing (AWU 908) and this session's size-based windowing had never actually windowed anything in the real end-to-end path**, always hitting `optimize`'s "no history, nothing to do"/"no messages were compressed" branch regardless of history size. Removed the incorrect renames (verified by round-tripping real snake_case JSON through `call_extension_method` in the new test — windowing now measurably fires, e.g. "Windowed history from 2 to 1 messages").
- **Scope**: `wit/context-tools.wit`, `wit/connector/llm-connector.wit` (moved), `src/wasm/bindings.rs`, `src/wasm/imports_delegate.rs`, `src/wasm.rs`, `ext/context-tools/src/lib.rs`, `ext/context-tools/src/tests.rs`, `ext/llm-connector/src/lib.rs`, `ext/rad-orchestrator/src/llm.rs`, `ARCHITECTURE.md`, `tests/context_tools_tests.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines; the real windowing round-trip is covered by an integration test through the actual WASM component boundary (not just native unit tests of guest-side logic).
- **Result**: Success. All 5 extensions build and pass `clippy --all-targets` cleanly under `wasm32-wasip2`; host `cargo build`/`clippy --lib` clean; `rad-models` (5 tests), `context-tools` (8 native unit tests), host `cargo test --lib` (40 tests), and the full integration suite (`context_restoration`/`context_tools` (2 new)/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 15 tests total) all pass. `ARCHITECTURE.md` §1.3.5 updated to describe size-based windowing and the `get-repo-map` semantic-map delegation.
- **File-size fallout**: `src/main.rs` grew to 302 lines from the `check_endpoint` integration (over CODING.md's 300-line limit). Split `load_config_and_session`/`check_active_llm_endpoint` into a new `src/startup.rs` (a binary-crate-root submodule — `main.rs` is its own crate root distinct from `lib.rs`, so its submodules resolve to plain `src/<name>.rs`, not the `src/<file>/<name>.rs` pattern used for `lib.rs` submodules like `config.rs`/`config/load.rs`). `main.rs` is now 192 lines, `startup.rs` 115.
- **Scope**: `models/src/lib.rs`, `models/src/rpc_conversion.rs` (new), `src/wasm/bindings.rs`, `src/wasm/bindings/rpc_command.rs`, `ext/security-guard/src/lib.rs`, `ext/mcp-tool-provider/src/conv.rs`, `ext/rad-orchestrator/src/conv.rs`, `ext/rad-orchestrator/src/conv/rpc.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `src/command/llm.rs`, `src/command/llm/context_length.rs`, `src/command/llm/profile_admin.rs`, `src/main.rs`, `src/startup.rs` (new).
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass; every touched file ≤300 lines.
- **Result**: Success. Line-count reductions from the macro consolidation alone: host `rpc_command.rs` 253→37, `security-guard/lib.rs` 159→46, `mcp-tool-provider/conv.rs` 196→25, `rad-orchestrator` `conv.rs`+`conv/rpc.rs` 71+293→34+24. All 5 extensions build and pass `clippy --all-targets` cleanly under `wasm32-wasip2`; host `cargo build`/`clippy --lib` clean; `rad-models` (5 tests), `context-tools` (8 tests), host `cargo test --lib` (40 tests), and the full integration suite (`context_restoration`/`e2e`/`hitl`/`mcp`/`multi_extension`/`repo_map`/`self_healing`/`tool_loop`, 13 tests) all pass with no regressions — `multi_extension_tests.rs` specifically exercises `security-guard`'s `verify_rpc` end-to-end through the new macro-generated conversion.

---

## 🛠️ Short-Term Plan: Phase 45

### 💡 Current AWU Status
- [✅] AWU 914: Fix Hardcoded LLM Model Name in `GenerateLlmStream` (Result: Success)

### 📝 AWU Details

#### AWU 914: Fix Hardcoded LLM Model Name in `GenerateLlmStream`
- **Trigger**: A 2026-07-27 design review of local-LLM context-overflow prevention (which shipped `context-tools` size-aware windowing, the `GetActiveLlmProfile` host RPC, and `/llm` context-length detection earlier the same session) surfaced a separate, pre-existing bug while auditing the same file: `ext/rad-orchestrator/src/llm.rs::trigger_llm_stream` sent a hardcoded `model: "qwen".to_string()` to `GenerateLlmStream` regardless of the active LLM profile's configured model — silently defeating `/llm add <name> <url> <model>` and `/llm model <new_model>` for any backend that actually routes on the request's `model` field (Ollama, LM Studio, vLLM; llama.cpp with a single loaded model happened to ignore it, masking the bug).
- **Fix**: Added `active_llm_model()` in `ext/rad-orchestrator/src/llm.rs`, mirroring the existing `active_llm_context_length()` pattern — both query the same `GetActiveLlmProfile` host RPC (built earlier this session) for a different field. `trigger_llm_stream` now sends the real configured model, falling back to an empty string (not a guessed model name) when none is configured, so a misconfiguration surfaces as a clear server-side error instead of a silent wrong-model request.
- **Scope**: `ext/rad-orchestrator/src/llm.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass.
- **Result**: Success. `rad-orchestrator` builds and passes `clippy --all-targets` cleanly under `wasm32-wasip2`; full host `cargo test --lib` (40 tests) and integration suite (`context_restoration`/`e2e`/`hitl`/`mcp`/`repo_map`/`self_healing`/`tool_loop`, 11 tests) pass with no regressions. `ext/rad-orchestrator/src/llm.rs` remains at 288 lines (CODING.md's 300-line limit).

---

## 🛠️ Short-Term Plan: Phase 44

### 💡 Current AWU Status
- [✅] AWU 913: Commit and Push Phase 43 Extension Crates Cleanups to Main (Result: Success)

### 📝 AWU Details

#### AWU 913: Commit and Push Phase 43 Extension Crates Cleanups to Main
- **Trigger**: User requested pushing completed Phase 43 changes to `main`.
- **Scope**: All modified workspace files in `ext/` and `PLANS.md`.
- **Definition of Done (DoD)**: All changes committed cleanly with descriptive commit message and pushed to `origin/main`.
- **Result**: Success. Committed (`4963118`) and pushed to `origin/main` on 2026-07-26. All pre-commit secret/path scanning checks passed.

---

## 🛠️ Short-Term Plan: Phase 43

### 💡 Current AWU Status
- [✅] AWU 912: Remove Hand-Written Clippy Allows in Extension Crates & Fix Warnings (Result: Success)

### 📝 AWU Details

#### AWU 912: Remove Hand-Written Clippy Allows in Extension Crates & Fix Warnings
- **Trigger**: User requested completely eliminating crate-level `#![allow(...)]` attributes by scoping necessary macro-generated allowances strictly to generated modules.
- **Changes**:
  1. **Zero Crate-Level Allow Directives**: Removed all crate-wide `#![allow(...)]` attributes from `lib.rs` across all 5 extension crates (`ext/context-tools`, `ext/llm-connector`, `ext/mcp-tool-provider`, `ext/rad-orchestrator`, `ext/security-guard`), leaving only `#![deny(clippy::pedantic)]` at crate root.
  2. **Scoped Macro Encapsulation**: Encapsulated `wit_bindgen::generate!` and `export!(...)` inside a dedicated `mod bindings` module annotated with `#[allow(unsafe_op_in_unsafe_fn, clippy::same_length_and_capacity, clippy::pedantic)]`, re-exporting via `pub use bindings::*;`. This ensures `wit_bindgen` generated code warnings are isolated to the generated module alone, while all hand-written code is strictly checked under `#![deny(clippy::pedantic)]`.
  3. **Code Fixes**: Resolved all hand-written Clippy warnings across extension crates (collapsible_if, uninlined_format_args, manual_strip, cast_possible_truncation, too_many_lines, single_match_else, manual_assert, collapsible_match).
- **Scope**: `ext/llm-connector/src/{lib.rs, connector.rs, event_stream.rs}`, `ext/mcp-tool-provider/src/{lib.rs, client.rs, conv.rs, mcp_config.rs, mcp_transport.rs}`, `ext/rad-orchestrator/src/{lib.rs, conv/rpc.rs, llm.rs, orchestrator.rs, tool.rs, orchestrator/reasoning.rs, orchestrator/runner/done.rs}`, `ext/security-guard/src/lib.rs`, `ext/context-tools/src/lib.rs`.
- **Definition of Done (DoD)**: Zero `#![allow(...)]` attributes at crate root; generated code warnings scoped strictly to `mod bindings`; all extension crates build cleanly under `wasm32-wasip1` with zero Clippy warnings; all workspace unit/integration tests and `./scripts/build_all.sh` pass cleanly.
- **Result**: Success. Verified via `cargo clippy --target wasm32-wasip1 -p <ext>` for all 5 extension crates, `cargo clippy --workspace --all-targets` (0 warnings), and full `./scripts/build_all.sh` run.

---

## 🛠️ Short-Term Plan: Phase 42

### 💡 Current AWU Status
- [✅] AWU 911: Split All Remaining CODING.md 300-Line Violations (Result: Success)

### 📝 AWU Details

#### AWU 911: Split All Remaining CODING.md 300-Line Violations
- **Trigger**: User asked for a recap of remaining known issues after AWU 910 shipped; AWU 910 had only fixed one of several files already over CODING.md's 300-line limit (`ext/rad-orchestrator/src/orchestrator.rs`, folded into that AWU because it was directly touched by the squashing fix). Nine more files were still over the limit. User then explicitly asked to address the 300-line limit ("300行制限に対応").
- **Files split** (each into 2–5 sibling files by concern, with the original either becoming a thin dispatcher or keeping the "core"/most-referenced piece and re-exporting where call sites needed an unchanged path):
  1. `src/wasm/imports_rpc.rs` (695→181 lines) → `imports_process.rs`, `imports_tool.rs`, `imports_http.rs`, `imports_delegate.rs` (process/tool/http import handlers + the trait-delegation macro & context-tools host bridge).
  2. `src/wasm/rpc_meta.rs` (603→166 lines) → `rpc_meta_fallback.rs` (built-in tool fallback), `rpc_meta_llm_connector.rs` (Wasm-connector LLM stream path), `rpc_meta_llm_fallback.rs` (raw-HTTP-SSE LLM stream path).
  3. `ext/mcp-tool-provider/src/client.rs` (387→132 lines) → `mcp_config.rs` (config discovery/parsing), `mcp_transport.rs` (JSON-RPC line transport; re-exports `mcp_request` so `lib.rs`'s import path is unchanged).
  4. `src/wasm/bindings.rs` (381→142 lines) → `bindings/rpc_command.rs` (the two large `RasRpcCommand`↔WIT `From` impls).
  5. `src/orchestrator/runner.rs` (366→202 lines) → `runner/runtimes.rs` (Wasm runtime init/lookup/clear), `runner/events.rs` (RPC verification fan-out + event-dispatch loop); both as sibling inherent-`impl Orchestrator` blocks.
  6. `src/wasm/imports_resources.rs` (351→114 lines) → `imports_resources_file.rs` (`file-handle` WIT impl), `imports_resources_exec.rs` (`execution-handle` WIT impl + llm-connector stream glue); shared `push_closed_fallback` helper made `pub(crate)`.
  7. `src/config.rs` (346→181 lines) → `config/merge.rs` (JSONC parse + recursive value merge), `config/load.rs` (config discovery/layered loading, `Config::apply_env_overrides`); `load_config` re-exported so `crate::config::load_config` call sites are unchanged.
  8. `ext/llm-connector/src/lib.rs` (337→28 lines) → `serialize_types.rs` (wire structs), `event_stream.rs` (SSE parsing + `GuestEventStream` impl), `connector.rs` (`ConnectorImpl`/`Guest` impl).
  9. `src/process.rs` (315→158 lines) → `process_child.rs` (`StdioChild`/`ChildKiller` glue + reader-thread helper), `process_running.rs` (`RunningProcess`, re-exported so `crate::process::RunningProcess` is unchanged).
  10. `ext/rad-orchestrator/src/orchestrator/runner.rs` (315→59 lines) → `runner/done.rs` (`handle_done`, the Done-event finalizer), `runner/inline_tool_calls.rs` (plain-text tool-call fallback parser); `handle_done` re-exported so `orchestrator.rs`'s import is unchanged.
- **Bugs hit and fixed during the split** (all caught by the user's `build_all.sh` runs, none reached `main`):
  - `wit_bindgen::generate!`'s `export!` macro only accepts a plain identifier, not a path — `export!(connector::ConnectorImpl)` failed to compile; fixed by `use connector::ConnectorImpl;` then `export!(ConnectorImpl);`.
  - `src/process/tests.rs` used `thread::sleep` via `use super::*`, relying on `process.rs`'s top-level `use std::thread;` which moved to `process_running.rs` in the split; added an explicit `use std::thread;` to the test file.
  - `src/config.rs`'s re-export of `merge_json_value`/`parse_jsonc` (added only so `config/tests.rs`'s `use super::*` could reach them) was flagged as an unused import in non-test builds, since nothing outside `config/tests.rs` used the re-exported path; removed the re-export and had the test file import directly from `super::merge::{..}` instead.
  - Clippy's `empty_line_after_doc_comments` (denied via `-D warnings`) fired on `config/merge.rs`, whose file-header note used `///` (a doc comment) followed by a blank line before the next doc comment block. The same header-comment pattern (`/// ... split out of ... 300-line limit.`) existed verbatim at the top of all 20 new/touched files from this AWU; rather than fix one and wait for the next to surface on a future build, converted all 20 headers from `///` to plain `//` in one pass, since they document the file's split-provenance for humans, not a specific downstream item for rustdoc.
- **Scope**: 10 original files edited (trimmed to dispatchers or "core" pieces), 22 new files created, plus `src/wasm.rs`, `src/lib.rs`, `ext/mcp-tool-provider/src/lib.rs`, `ext/llm-connector/src/lib.rs` updated with the corresponding `mod`/re-export declarations, and `src/config/tests.rs`/`src/process/tests.rs` fixed for the import fallout above.
- **Definition of Done (DoD)**: Every `.rs` file in the workspace (including test files) at or under 300 lines; all tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success. Verified via `find src ext -name "*.rs" | xargs wc -l` that no file exceeds 300 lines. User confirmed a clean `./scripts/build_all.sh` run (build, all unit/integration tests, Clippy audit, binary reinstall) after the three follow-up fixes above.

---

## 🛠️ Short-Term Plan: Phase 41

### 💡 Current AWU Status
- [✅] AWU 910: Remove Unsafe Tool-Call Squashing, Split orchestrator.rs, Remove Production Unwrap (Result: Success)

### 📝 AWU Details

#### AWU 910: Remove Unsafe Tool-Call Squashing, Split orchestrator.rs, Remove Production Unwrap
- **Trigger**: User asked for further improvement points after AWU 909; a second read-through of `ext/context-tools`, `ext/rad-orchestrator`, and `src/wasm/rpc_meta.rs` surfaced one more real bug plus the housekeeping items AWU 909 had left as "known outstanding issues." User said to fix all of it.
- **Issue 1 (real bug, pre-existing, not introduced by AWU 908/909)**: `context-tools`' role-based squashing (`compress_messages`) collapsed consecutive non-`user`/`assistant` messages into the last one. Since `system` is stripped out before `optimize()` sees the list, and the DAG only ever produces `user`/`assistant`/`tool`/`system` nodes, this logic only ever fires on runs of `tool` messages. `ext/rad-orchestrator/src/orchestrator/runner.rs` confirms parallel `tool_calls` (multiple tool calls in one `assistant` turn) are fully supported, each producing its own consecutive `tool` DAG node — so a 2+ tool-call turn would get squashed down to one `tool` message while the preceding `assistant` message's `tool_calls` array still listed all of them, producing an API-invalid request (same failure class as AWU 78/909, but reachable in a single turn, not just long sessions). There is no role in this system for which the squash is ever safe.
- **Fix 1**: Removed `compress_messages` and the squashing pass entirely from `ext/context-tools/src/lib.rs` rather than special-casing `tool` (nothing left for the generic mechanism to safely apply to). `optimize()` is now count-based windowing only. Updated `ARCHITECTURE.md`'s Context Compactor description and the L3-reset strategy line to match. Rewrote/renamed the two tests that exercised squashing to instead assert parallel tool-call pairs survive both un-windowed and windowed optimization.
- **Fix 2 (file-size limit)**: `ext/rad-orchestrator/src/orchestrator.rs` had grown to 318 lines (over CODING.md's 300-line limit) via AWU 909's earlier `RAD_DEBUG`-unification commit. Split the reasoning/thought-stream formatting helpers (`debug_enabled`, `RawEvent`/`ToolCallChunkEvent`/`CompletionUsageEvent`, `handle_content_token`, `handle_thought_start_tag`, `handle_reasoning_text`) into a new companion module `ext/rad-orchestrator/src/orchestrator/reasoning.rs` (128 lines). `orchestrator.rs` is now 194 lines.
- **Fix 3 (production unwrap)**: `src/wasm/rpc_meta.rs`'s background LLM-stream polling thread used `connector_ref.llm_connector.as_ref().unwrap()` every loop iteration. Replaced with a `let...else` that sends an `error`-type `LlmConnectorEvent` and breaks the loop instead of panicking the thread if the connector is ever gone.
- **Fix 4 (docs)**: Added a superseded-by-`PLANS.md` note to `TASKS.md` and marked its one abandoned unfinished item (AWU 205) rather than guessing at its original scope and leaving it silently stale.
- **Not fixed (deliberate)**: Crate-level `#![allow(clippy::...)]` in every extension crate. `unsafe_op_in_unsafe_fn` and `clippy::same_length_and_capacity` appear in literally every `wit_bindgen::generate!`-using crate — a generated-code artifact, not something hand-written code can clean up without patching `wit_bindgen` itself. The remaining crate-specific allows (`needless_pass_by_value`, `collapsible_if`, `uninlined_format_args`, `cast_possible_truncation`, `manual_strip`, `too_many_lines`, `collapsible_match`) look fixable but touching 5 crates' worth of lints with no local compiler to verify against was judged too risky for this round; left as-is.
- **Scope**: `ext/context-tools/src/lib.rs`, `ARCHITECTURE.md`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/orchestrator/reasoning.rs` (new), `src/wasm/rpc_meta.rs`, `TASKS.md`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success.

---

## 🛠️ Short-Term Plan: Phase 40

### 💡 Current AWU Status
- [✅] AWU 909: Fix Orphan Tool-Message Filter Ordering & CallExtension Blocking Lock (Result: Success)

### 📝 AWU Details

#### AWU 909: Fix Orphan Tool-Message Filter Ordering & CallExtension Blocking Lock
- **Trigger**: User asked for a general audit of the current code after AWU 908 shipped. Two concrete issues surfaced from reading `ext/rad-orchestrator/src/llm.rs` and `src/wasm/rpc_meta.rs` against the codebase's own documented history (AUDITING.md, CODING.md, PLANS.md archive).
- **Issue 1 (regression from AWU 908)**: The orphan-`tool`-message filter in `load_messages_from_dag` used to run *after* count-based history windowing, so any orphan the windowing created (by slicing between an `assistant`/`tool_calls` message and its `tool` reply) got cleaned up before the request reached the LLM. AWU 908 moved windowing into `context-tools` and left the filter running *before* it, on the unwindowed list — meaning `context-tools`' purely positional windowing could reintroduce an orphaned `tool` message that never gets filtered out again, recreating the class of bug AWU 78 ("Filter Out Isolated Tool Messages", a 400 Bad Request fix) originally addressed.
- **Issue 2**: `src/wasm/rpc_meta.rs`'s `CallExtension` RPC handler (the exact path AWU 908's `context-tools` calls go through every turn) held the outer `orch.wasm_runtime` lock for the full duration of the nested extension call and used a blocking `.lock()` on the target runtime, instead of the clone-Arc-then-`try_lock()` pattern used everywhere else in the same file (`GetTools`, `ExecuteTool`) since AWU 900's deadlock fix. Not a confirmed live deadlock, but the same risk class on a hot path.
- **Fix**:
  1. `ext/rad-orchestrator/src/llm.rs`: extracted the filter into `filter_orphaned_tool_messages()` and apply it twice — once after DAG reconstruction, once again after parsing `context-tools`' optimized response, before it's trusted.
  2. `src/wasm/rpc_meta.rs`: `CallExtension` now clones the target `Arc` out of a short-lived `wasm_runtime` lock, then `try_lock()`s the runtime, returning an error instead of blocking if it's busy.
- **Scope**: `ext/rad-orchestrator/src/llm.rs`, `src/wasm/rpc_meta.rs`.
- **Definition of Done (DoD)**: All tests + Clippy (`-D warnings`) pass under `./scripts/build_all.sh`.
- **Result**: Success. One Clippy pedantic fixup needed mid-flight (`clippy::doc_markdown` — backticks around `` `assistant`/`tool_call` `` in a doc comment). Final run: all 5 `context-tools` unit tests + 37 `rad` lib unit tests + 23 integration tests pass, Clippy clean, binary reinstalled.

---

## 🛠️ Short-Term Plan: Phase 39

### 💡 Current AWU Status
- [✅] AWU 908: Consolidate Context Compaction Logic into `context-tools` Extension (Result: Success)

### 📝 AWU Details

#### AWU 908: Consolidate Context Compaction Logic into `context-tools` Extension
- **Trigger**: User-led review of the context management mechanism (2026-07-26) surfaced that compaction logic is split across two places: count-based `max_history_messages` windowing lives in `ext/rad-orchestrator/src/llm.rs::load_messages_from_dag`, while role-based squashing of consecutive non-user/assistant messages lives in `ext/context-tools/src/lib.rs::optimize`. Same responsibility ("shrink history to fit"), two owners, one stateful (Orchestrator's global `OrchestratorState`) and one stateless.
- **Decision & Rationale**: Draw the boundary between "mechanical, DAG-coupled reconstruction" and "judgment-based compaction policy" rather than the doc's previous "construction vs. compaction" wording alone. DAG traversal (walking `parent_ids`, deserializing node text into `Message`, filtering orphaned `tool_call_id`s, assembling the system prompt) stays in the Orchestrator — it's a pure, deterministic transform tightly bound to `rad_models::Dag`, already shared as a library crate rather than crossed via RPC, and putting it behind a `context-tools` RPC call would add a hop on the hot path and turn an optional optimization into a single point of failure for message delivery. Compaction (what to keep/discard/summarize under a size budget) is genuine policy that benefits from being isolated, pluggable, and testable in one place — so both existing strategies (count windowing + role squashing) consolidate into `context-tools`.
- **Objective**: Extend the `context-tools` WIT `optimization-request` (`wit/context-tools.wit`) with a history-length threshold parameter. Move the count-based windowing logic (first-message-preserving trim, currently in `load_messages_from_dag`) into `MyContextTools::optimize` in `ext/context-tools/src/lib.rs`, applied ahead of or alongside the existing role-based squashing pass. Remove the now-duplicate windowing logic from `ext/rad-orchestrator/src/llm.rs`, and stop reading `max_history_messages` out of the Orchestrator's global `STATE` for that purpose (pass it as a request parameter instead).
- **Scope**: `wit/context-tools.wit`, `ext/context-tools/src/lib.rs`, `ext/rad-orchestrator/src/llm.rs`, `ext/rad-orchestrator/src/orchestrator.rs`, `ext/rad-orchestrator/src/types.rs` (state plumbing), unit tests in `ext/context-tools/src/lib.rs` and `ext/rad-orchestrator/src/orchestrator/tests.rs`.
- **Non-Goals**: DAG traversal, system-prompt assembly, and orphaned-tool-call filtering in `load_messages_from_dag` are explicitly out of scope and remain in the Orchestrator.
- **Definition of Done (DoD)**: `context-tools::optimize` accepts a history-length threshold and applies unified count- and role-based trimming in a single pass; `load_messages_from_dag` retains only DAG traversal/parsing/system-prompt assembly with no ad-hoc `max_history` windowing; all workspace unit/integration tests and Clippy audit pass cleanly.
- **Result**: Success. Added `max-history: option<u32>` to `optimization-request` in `wit/context-tools.wit`. `ext/context-tools/src/lib.rs` now applies `apply_history_window` (ported first+recent trim) ahead of the existing `compress_messages` role-squash pass inside `optimize`, with combined summary text and 5 unit tests (incl. windowing-only, under-limit no-op, and windowing+compression interaction). `ext/rad-orchestrator/src/llm.rs::load_messages_from_dag` no longer performs count-based trimming itself — it only reconstructs messages from the DAG, then forwards `max_history_messages` as the new `max-history` request field to `context-tools`. Used `try_from`/`unwrap_or` instead of `as` casts at the `usize`/`u32` boundary to stay clean under `clippy::pedantic`. Verified via `./scripts/build_all.sh` on 2026-07-26: all 5 new `context-tools` unit tests pass, all 37 `rad` lib unit tests + 23 integration tests pass, Clippy audit (`-D warnings`) clean, `rad` binary rebuilt and reinstalled to `~/.cargo/bin/rad`.

---

## 📜 Completed Work Archive (Phases 19–38)

Condensed one-line-per-AWU log. Full narrative detail (trigger/root-cause/fix breakdowns) has been trimmed; scope and outcome are preserved. Older phases (1–18) predate this log's granularity and are tracked only in the Roadmap above.

**Phase 38 (v0.43.0) — Deep End-to-End Codebase Audit & Verification of MCP Subsystem**
- AWU 907: Fixed `ExecutionHandle` being dropped (and SIGKILLing) spawned MCP servers right after handshake, by retaining the handle in `ActiveMcpServer`. — `ext/mcp-tool-provider`
- AWU 906: Non-blocking EOF handling in the MCP stream reader instead of looping to a 10s timeout. — `ext/mcp-tool-provider/src/client.rs`
- AWU 905: Unified canonical path resolution across permission checks and RPC gateways. — `src/wasm/permissions.rs`
- AWU 904: Standardized global config path resolution; cleaned diagnostic pipelines behind `RAD_MCP_DEBUG`. — `ext/mcp-tool-provider`, `src/config.rs`, `src/process.rs`
- AWU 903: Standardized stdio pipe spawning & executable PATH lookup, delegating shell features to `bash -c`. — `src/process.rs`
- AWU 902: Host-side deep-merge of global (`~/.rad/config.json`) and project configs in `load_config()`. — `src/config.rs`
- AWU 901: WASM-side deep-merge of `mcp_servers` configs (global + local). — `ext/mcp-tool-provider/src/client.rs`
- AWU 900: Fixed `verify_rpc_exclude` deadlock on nested extension calls via `try_lock()`. — `src/wasm/imports_rpc.rs`
- AWU 899: Exhaustive MCP subsystem audit — fixed stdin pipe pollution from `ping` health checks, dead handle cleanup, strict JSON-RPC id matching.
- AWU 897: Dedicated stdio piping for `open_process`/MCP server spawning, replacing PTY to avoid line mangling. — `src/process.rs`

**Phase 35 (v0.40.0) — Unified Path Resolution Architecture & Gateway Normalization**
- AWU 896: Centralized path resolution/tilde expansion/canonicalization into a single `resolve_target_path`. — `src/fs.rs`, `src/wasm/permissions.rs`, `src/wasm/rpc.rs`

**Phase 34 (v0.39.0) — Fix Tilde Expansion in Host Fs Subsystem**
- AWU 895: Applied `expand_tilde` across `file_read`/`file_write`/`canonicalize_path`/`has_permission`. — `src/fs.rs`

**Phase 33 (v0.38.0) — Fix read_config_file Error String Swallowing**
- AWU 894: `read_config_file` returns `None` on host RPC errors so config discovery falls back to `~/.rad/config.json` correctly. — `ext/mcp-tool-provider/src/client.rs`

**Phase 32 (v0.37.0) — Fix Tilde Expansion in WASM Permissions & Align MCP Config Discovery**
- AWU 893: Tilde expansion in `permissions.rs::has_path_permission`; aligned `load_mcp_config` search order with `src/config.rs`.

**Phase 31 (v0.36.0) — Stdio Pipe Fallback for PTY Permission Errors**
- AWU 892: Stdio pipe fallback in `ProcessManager::spawn_bash_process` when `openpty()` fails with permission denied. — `src/process.rs`

**Phase 30 (v0.35.0) — Complete Removal of Built-in Shell Fallbacks & Accurate MCP Startup Verification**
- AWU 888: Startup display reflects real runtime tool verification instead of pre-verification logs. — `src/main.rs`, `src/orchestrator/runner.rs`
- AWU 889: Removed legacy `execute_command`/`spawn_bash_process` fallbacks from the orchestrator. — `ext/rad-orchestrator/src/orchestrator/runner.rs`
- AWU 890: Removed legacy command fallbacks from host RPC. — `src/wasm/rpc_meta.rs`, `src/wasm/imports_rpc.rs`
- AWU 891: Build/test/audit/install/push cycle.

**Phase 29 (v0.34.0) — MCP Host Tilde Expansion & Instant Tool Result Visibility**
- AWU 884: Host-side tilde expansion & permission bypass for MCP binary paths (e.g. `~/.cargo/bin/`). — `src/wasm/imports_rpc.rs`, `src/wasm/permissions.rs`
- AWU 885: Removed redundant WASM-side unexpanded-path checks. — `ext/mcp-tool-provider/src/client.rs`
- AWU 886: Instant tool result output to terminal via `WriteStdout`. — `ext/rad-orchestrator/src/orchestrator/runner.rs`
- AWU 887: Build/test/audit/install/push cycle.

**Phase 28 (v0.33.0) — Documentation Update, Config Deployment & Git Main Release**
- AWU 880: Synced `ARCHITECTURE.md`/`CONFIG.md`/`README.md` with the `rad-orchestrator`/`llm-connector` refactor.
- AWU 881: Updated global/workspace config files to reference `rad-orchestrator`/`llm-connector`.
- AWU 882: Build/test/verification cycle (37 unit + 23 integration tests, Clippy).
- AWU 883: Commit & push to main.

**Phase 23 (v0.28.0) — Configurable LLM Endpoints & /llm Slash Command Management**
- AWU 863: Added `LlmConfig`/`LlmEndpointProfile` with `env:VAR_NAME` credential resolution. — `src/config.rs`
- AWU 864: Implemented `/llm` slash command subsystem (list/switch/test/add/model). — `src/command/llm.rs`
- AWU 865: Propagated active LLM profile settings to the WASM connector runtime. — `ext/openai-connector/src/lib.rs`
- AWU 866: Audit/build/install/verify cycle.

**Phase 22 (v0.27.0) — Fix LLM Connection Hangs & Add Graceful Error Handling**
- AWU 860: Added HTTP connect timeouts (10s) and clearer error messages for unreachable hosts. — `src/http.rs`
- AWU 861: `openai-connector` reads `OPENAI_BASE_URL`/`OPENAI_API_KEY`, fails fast when unreachable/unconfigured.
- AWU 862: Audit/build/install cycle.

**Phase 21 (v0.26.0) — Release Build, Local Installation, ~/.rad/config.json Creation & Push to GitHub**
- AWU 857: Created default `~/.rad/config.json`.
- AWU 858: Built & installed release binaries to `~/.cargo/bin`.
- AWU 859: Commit & push to GitHub.

**Phase 19–20 (v0.24.0–v0.25.0) — Remove Built-in Extensions & Standardize Global Config Directory**
- AWU 852: Finalized removal plan for `core-tool-provider`/`web-access` extensions.
- AWU 853: Deleted legacy extension crates and WIT/host bindings. — `ext/core-tool-provider`, `ext/web-access`, `src/wasm/*`
- AWU 854: Routed tool calls via `mcp-tool-provider`; updated `rad.json`/`Cargo.toml`/tests.
- AWU 855: Synced `ARCHITECTURE.md`/`EXTENSIONS.md`/`CONFIG.md`; ran full verification audit.
- AWU 856: Prioritized `~/.rad/config.json` in global config discovery. — `src/config.rs`
