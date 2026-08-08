#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo "🚀 RAD Ecosystem One-Command Build Pipeline"
echo "=========================================="

echo "📦 Step 1: Building WASM Component Extensions..."
# Packages and artefact names both derived from the workspace. Written by hand
# they drifted repeatedly — `skills-module` was missing from CI's list, and both
# of CI's wasm steps missed every module added after `context-module`. A
# component that is never built looks exactly like one that builds cleanly.
PKGS=$(cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[] | select(.manifest_path | test("/(ext|modules)/")) | .name')
echo "  components: $(echo "$PKGS" | tr '\n' ' ')"

# shellcheck disable=SC2046  # word splitting is what turns the list into flags
cargo build --target wasm32-wasip2 --release $(echo "$PKGS" | sed 's/^/-p /')

mkdir -p ~/.rad/wasm
mkdir -p target/wasm32-wasip2/debug

# Cargo replaces `-` with `_` in artefact names. Test-only components opt out
# with `[package.metadata.rad] ship = false` — `modules/echo`, `relay`, `spawn`
# and `net` are built for the suite but not installed. Both extensions and
# kernel modules land here; which of the two a component is depends on the world it
# exports, and is declared in `~/.rad/config.json` under `extensions` or
# `modules` respectively — see CONFIG.md.
SHIP=$(cargo metadata --no-deps --format-version 1 \
    | jq -r '.packages[]
             | select(.manifest_path | test("/(ext|modules)/"))
             | select(.metadata.rad.ship != false)
             | .name')
WASM_FILES=()
while IFS= read -r pkg; do
    WASM_FILES+=("${pkg//-/_}.wasm")
done <<< "$SHIP"

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
cargo deny check licenses

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
