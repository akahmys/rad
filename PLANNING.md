# PLANNING.md (Agent Planning Policy)

## 1. Core Directives
- **AWU-Driven**: Decompose all tasks into Atomic Work Units (AWUs) completable in a single execution cycle.
- **Centralized State**: Maintain `PLANS.md` as the sole source of truth for all plans, progress, and execution logs.
- **Plan Integrity**: NEVER delete or omit existing sections in `PLANS.md` (past logs, roadmap, unstarted AWUs) unless explicitly instructed by the user.

## 2. AWU Criteria
An AWU must satisfy ALL:
- **Clear DoD**: Exactly one measurable Definition of Done.
- **Tight Scope**: Strictly limited target files and objectives (ideally < 3 files per AWU).
- **Context Fit**: Optimized for token efficiency. Avoid AWUs that require loading more than 10 files or performing massive refactors in one go.

## 3. Planning Layers
- **Long-Term (Roadmap)**: High-level milestones only. Minimize text.
- **Short-Term (Next 3–5 AWUs)**: Detailed steps, scope, and DoD. Expand roadmap items into AWUs only on-demand (when the current AWU is nearing completion).

## 4. Plan Revision Protocol
**If implementation reveals a fundamental design flaw or scope creep:**
1. **STOP** execution.
2. **Propose Revision**: Describe the discrepancy and the necessary change to the plan.
3. **Wait for User Approval**: Do not proceed with a revised plan without explicit confirmation.
4. **Update `PLANS.md`**: Once approved, update the roadmap/short-term plan and record the reason for the revision in the previous AWU's `Result` field.

## 5. `PLANS.md` Structural Template

```markdown
# Project Work Plan (PLANS.md)
**Last Updated**: YYYY-MM-DD

## 🗺️ Long-Term Plan (Roadmap)
- [✅] Phase 1: ...
- [🔄] Phase 2: ...
- [ ] Phase 3: ...

---

## 🛠️ Short-Term Plan: Phase 2 (...)

### 💡 Current AWU Status
- [✅] AWU-1: [Completed] ...
- [🔄] AWU-2: [In Progress] ...
- [ ] AWU-3: [Todo] ...

### 📝 AWU Details

#### AWU-1: [Title]
- **Objective**: ...
- **Scope**: ...
- **DoD**: ...
- **Result**: ...

#### AWU-2: [Title]
- **Objective**: ...
- **Scope**: ...
- **DoD**: ...
```
