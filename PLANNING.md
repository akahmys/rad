# PLANNING.md (Agent Planning Policy)

## 1. Core Directives
- **AWU-Driven**: Decompose all tasks into Atomic Work Units (AWUs) completable in a single execution cycle.
- **Centralized State**: Maintain `PLANS.md` as the source of truth for what is *open* — the roadmap, the unit in progress, and everything carried forward.
- **Plan Integrity**: NEVER delete an unstarted AWU, a roadmap entry, or anything still open. Completed units are a different matter — see §6.

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

---

## 🛠️ Short-Term Plan: Phase 2 (...)

### 📍 Where this is
Two or three sentences. What is in progress, what is paused and why, and the
shape of the system right now.

### 🗂️ What each AWU did
| AWU | Commit | What |
|---|---|---|
| 123 | `abc1234` | one line |

### 🔜 Next: AWU N — [title]
- **Objective**, **DoD**, and anything measured that the next session should
  not re-derive (blast radius, call-site counts, known hazards).

### ⚠️ Still open
Carried-forward items, deferred decisions, unresolved flakes.
```

## 6. What Belongs in `PLANS.md`, and What Does Not

`PLANS.md` is read at the start of a session to answer "where are we and what is
open". Everything else has a better home, and putting it here makes that
question harder to answer.

- **A completed AWU's reasoning goes in its commit message**, not here. Write it
  there in full — the measurements, the negative controls, what was rejected and
  why — and leave a one-line row in the index pointing at the hash. `git show`
  is one command; a 2,000-line plan file is not one read.
- **Reusable rules go in the policy documents.** A verification technique that
  worked belongs in `TESTING.md`, a design decision or invariant in
  `ARCHITECTURE.md` / `ARCHITECTURE-NEXT.md`, a configuration change in
  `CONFIG.md`. Left in a per-AWU narrative, nobody reads it again.
- **A limit or a caveat goes next to the thing it limits** — a code comment, or
  the section of the design document making the claim.
- **What stays**: the roadmap, the unit in progress, what is carried forward,
  what is still undecided, and any measurement the next session would otherwise
  re-derive.

When this file passes a few hundred lines, that is the signal it has absorbed
something belonging elsewhere. Earlier phases move to `PLANS-ARCHIVE.md` whole —
they are how the current design was arrived at, and summarising them would lose
exactly the part that is not derivable from the code.
