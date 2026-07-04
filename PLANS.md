# PLANS

## Objective
Create and manage the architectural specifications and foundational documentation of the `rad` (Rust Agent Dispatcher) project, followed by implementing the core runtime.

## Plan
1. Create `README.md` (Completed)
2. Refine `README.md` based on user feedback (Completed)
3. Adjust the introduction and simplify the overall document (Completed)
4. Fix phrasing in the introduction (Completed)
5. Delete "Configuration and Extension Management" section (Completed)
6. Create `rad` Architecture Design Specification (`ARCHITECTURE.md`) (Completed)
7. Configure remote repository, tag `v0.0.0`, and push to GitHub (Deferred)
8. Unify configuration file name to `rad.json`, revise `CONFIG.md`, and update `ARCHITECTURE.md` (Completed)
9. Add global layout rules based on recommended proposals to `CONFIG.md` (Completed)
10. Define minimal `rad.json` schema, revise `CONFIG.md`, and create a minimal `rad.json` file (Completed)
11. Create a full `rad.json` with explanatory comments and update `CONFIG.md` to support JSONC (Completed)
12. Remove `version` field from both `rad.json` and `CONFIG.md` (Completed)
13. Integrate `rad.json` contents into `CONFIG.md` and delete `rad.json` (Completed)
14. Audit all files for consistency and redundancy (specifically schemas in ARCHITECTURE.md and CONFIG.md) (Completed)

## Version 0.1 Implementation Plan (Building a Minimum Working Runtime)
Objective: Implement a minimal running `rad` Core runtime (CLI) and standard builtin Extension mechanism based on `ARCHITECTURE.md` and `CONFIG.md`.

### Milestones & Phases
1. **Phase 1: Project Initialization & Configuration Parser (AWU 1)**
   - Create `Cargo.toml` and define core structures (configuration schema, common events).
2. **Phase 2: Process Subsystem Implementation (AWU 2)**
   - Implement isolated PGID creation, stdout/stderr forwarding, and SIGKILL group cleanup on drop.
3. **Phase 3: File System Subsystem & Capability Checks (AWU 3)**
   - Implement file read/write, simple patching, and capability mask validation based on path normalization.
4. **Phase 4: DAG & Snapshot Subsystems (AWU 4)**
   - Implement in-memory DAG state tracking and physical snapshots backup/restoration.
5. **Phase 5: Core-Extension Bridge & Mock/Builtin Extension (AWU 5)**
   - Implement dual-channel JSON IPC loop and a test scenario runner as a builtin extension.
6. **Phase 6: Integration Tests & Quality Audit (AWU 6)**
   - Create comprehensive tests under `tests/`, ensure zero clippy warnings, and verify test suite passes.
