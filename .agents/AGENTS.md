# Rust AI Rules: AWU, Strict Audit, PLANS/TASKS Logs, and Auto-Commit

*(Overrides Antigravity planning: Use PLANS.md / TASKS.md at root instead of standard artifacts).*

## 🌀 Rust Design Constraints
- Strictly adhere to all rules in [CODING_RULES.md](file:///Users/akahmys/projects/rad/CODING_RULES.md) (Clippy, file/function size, test separation, safety).

## 🌐 Language Policy
- **Files & Code**: All files, code, documentation, schemas, and inline comments in the repository must be written exclusively in **English**.
- **Chat Communication**: All conversations and responses to the user in the chat must be conducted in **Japanese**.


## 🔍 Context Economy in Search & Directory Listing
- **Minimize Scope**: Restrict search paths and use file filters (e.g., specific globs) in `grep_search` / `list_dir`. Do not search the whole workspace.
- **Limit Output**: Always limit terminal command outputs (e.g., `head -n 30` or `-n 5` flags) to avoid token bloat.

## 📑 3-Step AWU Workflow

### Step 1: Planning & Tracking (DO FIRST - Code Changes Forbidden)
1. Create/update root files:
   - `PLANS.md`: Overall architecture/goals.
   - `TASKS.md`: Markdown checklist of AWUs. Mark current task with `(Current)`.
2. Present the plan for the **first AWU** in chat.
3. Wait for the user to say **"GO"** before editing any production code.

### Step 2: Incremental Implementation
- Implement **only** the approved AWU (YAGNI).
- Keep files and functions within limits defined in [CODING_RULES.md](file:///Users/akahmys/projects/rad/CODING_RULES.md).

### Step 3: Strict Audit & Auto-Commit
Perform verification in this strict order:
1. **Self-Size Check**: Verify compliance with [CODING_RULES.md](file:///Users/akahmys/projects/rad/CODING_RULES.md) size limits before compiling. Refactor immediately if violated.
2. Run `cargo check` -> Must pass.
3. Run `cargo clippy --all-targets` -> Must have **ZERO warnings**.
4. Run `cargo test` -> All tests must pass.

**Upon Success**:
1. Mark completed task in `TASKS.md` with `[x]`.
2. Run `git add .` and propose `git commit -m "<type>(<scope>): <description>"` (Conventional Commits) for user approval.
3. Report audit results, proposed commit, and ask to proceed to the next AWU.

**Upon Failure**:
- Fix errors immediately. Do not commit or ask for help unless completely blocked.
