# CODING.md (Rust Coding Policy)

## 1. Clippy Compliance
- **Zero Warnings**: Must pass `cargo clippy --workspace --all-targets -- -D warnings` with `#![deny(clippy::pedantic)]`. `--workspace` is required: without it cargo checks only the root package and skips `ext/*` and `models/` entirely.
- **Both Targets**: Extensions ship as `wasm32-wasip2`. Lint that target too — a native-only pass misses target-specific breakage.
- **No Bypassing**: NEVER use `#[allow(...)]` to suppress a pedantic lint in code you wrote. The sole exception is a module wrapping generated code (`wit_bindgen::generate!`), which cannot be edited to satisfy the lint; keep the attribute on that module and nowhere else.

## 2. Size & Complexity
- **File Limit**: < 300 lines.
- **Function Limit**: < 40 lines (Strict SRP). **Maintain logical cohesion; avoid fragmentation caused by excessive splitting.**
- **Test Separation**: NO `#[cfg(test)]` in production files. Use companion files (e.g., `src/module/tests.rs`). In the parent module, use `#[cfg(test)] mod tests;`. Test files also < 300 lines.

## 3. Scope
- **Strict Adherence**: Implement ONLY what is required for the current AWU.
- **No Speculation**: NO extra traits, unused generics, or future-proofing.
- **No Placeholders**: NO `TODO` comments or empty function bodies.

## 4. Safety & Simplicity
- **Error Handling**:
  - Use `thiserror` for domain-specific error types.
  - Use `anyhow` for high-level error propagation in application/CLI logic.
  - NEVER use `unwrap()` or `expect()` in production (use `Result`/`Option`). Permitted in tests only.
- **No Unsafe**: `unsafe` code is strictly prohibited.
- **Lifetime Management**: Avoid complex lifetime annotations. Prefer ownership transfer, cloning, or smart pointers (`Rc`/`Arc`).

## 5. Precision
- **Minimal Footprint**: Solve with the smallest possible change.
- **No Collateral Refactoring**: Do NOT modify unrelated code or styles.
- **Localized Impact**: Keep all modifications strictly localized.

## 6. Security
- **No Absolute Paths**: Use relative paths or environment discovery.
- **No Secrets**: NO hardcoded API keys, tokens, or credentials. Use environment variables.
