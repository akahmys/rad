# AUDITING.md (Audit Rules & Checklist)

**Trigger**: Completion of every AWU $\rightarrow$ Perform Audit $\rightarrow$ Update `PLANS.md`.

## 📋 Planning Audit
- **Integrity**: Maintain `PLANS.md` structure (No deletion of past logs/roadmap).
- **Atomicity**: Ensure AWU is a single, completed unit.
- **Consistency**: Ensure `PLANS.md` reflects the current state.
- **No Omission**: Do not omit unstarted AWUs or roadmap items.

## ⚡ Efficiency Audit
- **On-Demand**: Read ONLY necessary files.
- **Incremental**: Use `edit` calls; NEVER overwrite entire files.
- **Minimal Verbosity**: Avoid high-output commands (e.g., `ls -R`).
- **Hygiene**: Prune logs/errors from context.

## ⚖️ Compliance Audit
- **Size Constraints**: Verify file size (< 300 lines) and function size (< 40 lines).
- **Test Structure**: Ensure tests are in companion files and not in production files.
- **Code Style**: Verify adherence to all `CODING.md` rules.

## 🔍 Documentation Audit
- **Doc Comments**: Ensure all new/modified public APIs have appropriate doc comments (`///`).
- **Consistency**: Ensure documentation accurately reflects the current implementation.

## 🤖 Mechanical Audit (Automated/Rule-based)
- **Compilation**: `cargo check` passes with no errors.
- **Formatting**: `cargo fmt --check` passes.
- **Linting**: `cargo clippy` passes with no warnings (or only permitted ones).
- **Testing**: `cargo test` passes all relevant tests.
- **Security**: (If applicable) `cargo audit` passes.
- **Project Scripts**: Run all applicable scripts in `scripts/` and ensure they pass.

## ⚠️ Violation Protocol
**If any check fails:**
1. **STOP** execution immediately.
2. **Identify** the breached rule.
3. **Remediate** the code or plan.
4. **Re-Audit**.
5. **Report** discrepancy and fix to user.
