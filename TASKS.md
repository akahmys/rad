# TASKS

- [x] Create `CONFIG.md`
- [x] Simplify `README.md` and link to `CONFIG.md`
- [x] Remove 'Honke' keyword from README.md
- [x] Remove configuration section from README.md
- [x] Create `ARCHITECTURE.md` based on the specified architecture
- [x] Configure remote origin, tag `v0.0.0`, and push to GitHub
- [x] Unify configuration file to `rad.json` and update `CONFIG.md` and `ARCHITECTURE.md`
- [x] Add global path layout rules based on the recommended proposal to `CONFIG.md`
- [x] Refine `rad.json` schema to be minimal, update `CONFIG.md`, and create `rad.json` at root
- [x] Create full `rad.json` with explanatory comments, and update `CONFIG.md` to support JSONC
- [x] Delete `version` field from both `rad.json` and `CONFIG.md`
- [x] Integrate `rad.json` into `CONFIG.md` and delete `rad.json`
- [x] Audit all files for consistency and redundancy (specifically ARCHITECTURE.md vs CONFIG.md schemas)

## Version 0.1 Implementation Phase (Atomic Work Units)
- [x] AWU 0: Security Rules & Secret Leak Prevention Harness
- [x] AWU 1: Project Setup & Configuration Parser
  - [x] Initialize Cargo project structure (`Cargo.toml`, `src/main.rs`, `src/config.rs`)
  - [x] Add dependencies (`serde`, `serde_json`, `jsonc-parser`, `dirs`, `clap`)
  - [x] Define Rust configuration structs (`Config`, `CoreConfig`, `TimeoutConfig`, `ExtensionConfig`, `PermissionConfig`, etc.) based on `CONFIG.md`
  - [x] Implement XDG/Windows configuration lookup logic
  - [x] Implement JSONC loading and merging for `rad.local.json`
  - [x] Write integration and unit tests in companion files (e.g., `src/config/tests.rs`)
  - [x] Ensure all code adheres to `#![deny(clippy::pedantic)]` and passes Clippy, check_secrets.sh, and cargo test
- [ ] (Current) AWU 2: Process Subsystem with PGID Isolation & Group Cleanup
- [ ] AWU 3: Filesystem Sandbox & Snapshot Backup/Restoration
- [ ] AWU 4: DAG Tracking & Dual-Channel JSON IPC Bridge
- [ ] AWU 5: WebAssembly Runtime Integration (wasmtime)
- [ ] AWU 6: PTY Support & Filesystem Watcher Sensor
- [ ] AWU 7: HTTP Streaming Client & Dynamic Timeout Policies
- [ ] AWU 8: Security Audit & E2E Integration Tests
