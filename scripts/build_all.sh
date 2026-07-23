#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo "🚀 RAD Ecosystem One-Command Build Pipeline"
echo "=========================================="

echo "📦 Step 1: Building WASM Component Extensions..."
cargo build --target wasm32-wasip2 --release \
    -p rad-orchestrator \
    -p llm-connector \
    -p security-guard \
    -p mcp-tool-provider \
    -p context-tools

mkdir -p ~/.rad/wasm
cp target/wasm32-wasip2/release/*.wasm ~/.rad/wasm/

echo "🧪 Step 2: Running Unit and Integration Tests..."
cargo test --workspace

echo "🔍 Step 3: Running Clippy Audit..."
cargo clippy --workspace -- -D warnings

echo "⚙️ Step 4: Installing rad binary locally..."
cargo install --path .

echo "=========================================="
echo "✅ RAD Build and Deployment Completed Successfully!"
echo "=========================================="
