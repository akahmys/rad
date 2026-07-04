# Rust AI Coding Rules (Clippy-Driven & Compact Files)

This document defines strict guidelines for Rust development to ensure maximum code simplicity, safety, and optimal context efficiency for AI agents.

## 1. Zero Clippy Warnings
- All code must pass `cargo clippy --all-targets` with **zero warnings** under `#![deny(clippy::pedantic)]` enforced at the crate level.
- Do not add crate-level `#![allow(...)]` for pedantic lints.

## 2. File Size Limit & Test Separation
- **Strict Size Limit**: Keep every source file under **300 lines**.
- **Test Separation**: To minimize AI context size and isolate testing concerns:
  - **Never** write test code (`#[cfg(test)]`) inside production source files.
  - Always declare tests as an external module: `#[cfg(test)] mod tests;`
  - Place all test implementations in a dedicated companion file (e.g., `src/module/tests.rs` for `src/module.rs`).
  - Test files are also subject to the 300-line limit.

## 3. Function Size Limit
- Keep functions under **40 lines** of pure logic (excluding blank lines and comments).
- Strictly adhere to the Single Responsibility Principle (SRP). If a function exceeds this limit or handles multiple tasks, extract helper functions immediately.

## 4. YAGNI (You Aren't Gonna Need It)
- Implement only what is strictly required for current tasks.
- Avoid speculative features, placeholder structures, premature traits, or unnecessary generics. Keep it minimal and concrete.

## 5. Safety & KISS Lifetimes
- **No Panics**:
  - Never use `unwrap()` or `expect()` in production code. Handle errors gracefully using `Result` or `Option` mapping.
  - Exception: `unwrap()` is permitted in test files to keep test assertions concise.
- **No Unsafe**: Direct use of `unsafe` code is strictly prohibited.
- **KISS Lifetimes**: Avoid complex explicit lifetime annotations (`'a`). Prefer transferring ownership, cheap cloning, or smart pointers (`Rc`/`Arc`) to maintain simplicity.

## 6. Security & Leak Prevention
- **No Absolute Paths**: Never hardcode local absolute paths (e.g., `/Users/` or `/home/`). Always use relative paths or retrieve paths dynamically from the environment or configuration.
- **No Secrets**: Never commit API keys, tokens, or private credentials (e.g., OpenAI `sk-...`, Anthropic `sk-ant-...`). Use environment variables or configuration files that are excluded from version control (e.g., via `.gitignore`).
