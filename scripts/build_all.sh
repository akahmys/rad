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
    -p skill-tool-provider \
    -p context-tools

mkdir -p ~/.rad/wasm
mkdir -p target/wasm32-wasip2/debug

WASM_FILES=(
    "rad_orchestrator.wasm"
    "llm_connector.wasm"
    "security_guard.wasm"
    "mcp_tool_provider.wasm"
    "skill_tool_provider.wasm"
    "context_tools.wasm"
)

for file in "${WASM_FILES[@]}"; do
    cp "target/wasm32-wasip2/release/${file}" ~/.rad/wasm/
    cp "target/wasm32-wasip2/release/${file}" target/wasm32-wasip2/debug/
done

echo "🔍 Step 2: Running Code Quality & Safety Audits..."
echo "  - WIT contract sync check..."
# `wit/rad.wit` is the single source of truth; the template copies exist only
# so a scaffolded extension compiles standalone. Nothing keeps them in sync
# automatically, and they have silently drifted after WIT edits more than once
# (spawn-mcp-server removal, then file-changed removal), so the drift is
# checked here rather than discovered later by a confused extension author.
for template_wit in templates/*/wit/rad.wit; do
    if ! diff -q wit/rad.wit "$template_wit" >/dev/null; then
        echo "  ❌ $template_wit has drifted from wit/rad.wit"
        diff wit/rad.wit "$template_wit" || true
        echo "  Fix with: cp wit/rad.wit $template_wit"
        exit 1
    fi
done

echo "  - Formatting check (cargo fmt)..."
cargo fmt --check

echo "  - License compliance audit..."
python3 scripts/check_licenses.py

echo "  - Secret & path scanner..."
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    betterleaks git --no-banner --redact .
else
    echo "  (Skipping git secret scanner: not in a git repository context)"
fi

echo "🧪 Step 3: Running Unit and Integration Tests..."
cargo test --workspace

echo "🔍 Step 4: Running Clippy Audit..."
cargo clippy --workspace -- -D warnings

echo "⚙️ Step 5: Installing rad binary locally..."
cargo install --path .

echo "=========================================="
echo "✅ RAD Build and Deployment Completed Successfully!"
echo "=========================================="
