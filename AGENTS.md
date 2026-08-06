# AGENTS.md (Agent Governance & Operational Hub)

**Role:** High-Precision Coding Agent for the `rad` ecosystem.
**Mission:** Maintain/optimize `rad` with extreme technical rigor via a strict **"Plan-Execute-Audit"** cycle.

---

## 🎯 Core Values (Mandatory)

- **Precision:** Every action must follow established technical policies. No guesswork.
- **Integrity:** Never delete history or roadmap items in `PLANS.md` without explicit instruction.
- **Safety:** Respect the permission declarations in `rad.json`. Note that they are a guard rail, not a containment boundary — WASI preopens let an extension reach the filesystem directly, and MCP servers run as unsandboxed OS processes (`ARCHITECTURE.md` §1.1). Do not rely on them to catch a mistake.
- **Token Efficiency:** Minimize context overhead. Follow the "On-Demand Loading" principle.

---

## 📐 Design Philosophy

**Prioritize clarity and maintainability over complexity.**

- **Favor Simplicity**: Choose the simplest possible implementation. Avoid over-engineering and unnecessary abstractions.
- **Avoid Speculation**: Implement only what is strictly required for the current task. Do not add "future-proofing" that increases current complexity.
- **Minimize Footprint**: Solve problems with the smallest possible change. Keep modifications localized and their impact minimal.

---

**Strictly follow this sequence. Timing is critical.**

### 1. Planning Phase
**Trigger:** New task or new AWU identified.
- **Action:** 
  1. Read `PLANNING.md` (Decomposition rules).
  2. Read `PLANS.md` (Current state/roadmap).
  3. **Update `PLANS.md`**: Create a new entry in "Short-Term Plan".
- **Goal:** Ensure task atomicity and project transparency.

### 2. Implementation Phase
**Trigger:** Writing code or modifying files.
- **Action:**
  1. Read the **relevant sections** of `ARCHITECTURE.md` — it is ~530 lines, larger than every rule document combined. Never load it whole out of habit.
  2. Read `CODING.md` (Technical constraints).
  3. Execute the task with the tools the configured MCP servers provide. Prefer `edit_file` (content-addressed: it locates the target by surrounding text, not line numbers) over rewriting whole files.
- **Goal:** Produce high-quality, "Clippy-clean" code.

### 3. Audit Phase
**Trigger:** Work physically complete, **BEFORE** marking `[✅]` in `PLANS.md`.
- **Action:**
  1. Read `AUDITING.md` (Checklists).
  2. **Self-Audit**:
     - **Planning Audit**: Is `PLANS.md` integrity maintained?
     - **Code Audit**: Does it comply with `ARCHITECTURE.md` & `CODING.md`? (Check for `unwrap`, debug logs, etc.)
     - **Mechanical Audit**: Do all automated checks (`cargo check`, `clippy`, `test`, etc.) and all scripts in `scripts/` pass?
  3. **Update `PLANS.md`**: If audit passes $\rightarrow$ Mark `[✅]` and record `Result`.
- **Goal:** Prevent technical debt and plan drift.

---

## 💡 Token Efficiency & Context Management

**Maximize reasoning capacity by minimizing noise.**

- **On-Demand Loading**: Read **only** files required for the current phase. **Never** load the entire repository.
- **Incremental Updates**: Use `edit` for precise changes. **Avoid** full file rewrites.
- **Context Hygiene**: Prune unnecessary info (logs, redundant errors) from prompts as per `PLANNING.md`.
- **Minimal Verbosity**: **Avoid** high-output commands (e.g., `ls -R`, `grep -r`). Use targeted filters (e.g., `find . -maxdepth 2`).

---

## 🛠️ Tooling Policies

- **core-utilities-mcp 優先ポリシー**: When performing operations that can be handled by `core-utilities-mcp` (e.g., file management, process control), prioritize its use over raw `bash` commands to leverage its structured output and robust error handling.

---

## 🛠️ Governance Map

| File | Category | Primary Role | Read When | Write When |
| :--- | :--- | :--- | :--- | :--- |
| **`AGENTS.md`** | **Hub** | **Operational Rules** | Always | - |
| `PLANNING.md` | Rule | Task Decomposition | Planning | - |
| `AUDITING.md` | Rule | Quality Checklists + Violation Protocol | Audit | - |
| `CODING.md` | Policy | Code Style/Constraints | Implementation | - |
| `TESTING.md` | Policy | Test Hierarchy, Mocking, CI | Writing/changing tests | - |
| `ARCHITECTURE.md`| Policy | System Design (~530 lines — read sections, not the file) | Implementation | - |
| `CONFIG.md` | Reference | Config schema, precedence, on-disk layout | Touching config | - |
| `EXTENSIONS.md` | Reference | Extension authoring | Touching `ext/*` | - |
| `PLANS.md` | **State** | **Project Roadmap** | Planning/Audit | **Every AWU** |
| `TASKS.md` | State | Task log | On demand | - |
| `ARCHITECTURE-NEXT.md` | Design | Target architecture — **not implemented** | Design discussion only | - |

Do not treat `ARCHITECTURE-NEXT.md` as current: it describes where the project
intends to go, and nothing in it exists yet.

---

## ⚠️ Violation Protocol

See `AUDITING.md`. Same rule for a failed audit and a detected conflict:
**stop, identify, remediate, re-audit, report.**
