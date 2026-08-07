#!/usr/bin/env bash
#
# Secret and personal-information scanning, delegated to betterleaks.
#
# Replaces the hand-rolled `check_secrets.sh`, which showed three distinct
# failure modes in one day: a false positive on `auth_header: "Authorization"`
# that blocked legitimate work, a `set -e`/`pipefail` interaction that silently
# disabled the entire scan while still exiting non-zero, and a standing blind
# spot where a quoted JSON key (`"credential": "AKIA…"`) matched nothing.
#
# Rules live in `.betterleaks.toml` — both the secret rules (security) and the
# absolute-path rule (personal information: `/Users/<name>` leaks an account
# name).
#
# Scope matters and differs by mode. `--staged` sees only what is about to be
# committed, which is what a pre-commit hook should judge. `--all` walks history,
# because a secret removed from HEAD is still published.
set -euo pipefail

MODE="${1:---staged}"

if ! command -v betterleaks >/dev/null 2>&1; then
    echo "betterleaks is not installed."
    echo "  brew install betterleaks     # or: go install github.com/betterleaks/betterleaks@latest"
    # Hard failure, not a skip. A scanner that quietly does nothing is worse
    # than no scanner: it reports success and nobody looks again. That failure
    # mode is exactly what retired the previous script.
    exit 1
fi

case "$MODE" in
    --staged)
        echo "=== Scanning staged changes ==="
        betterleaks git --staged --no-banner --redact .
        ;;
    --all)
        echo "=== Scanning full history ==="
        betterleaks git --no-banner --redact .
        ;;
    *)
        echo "Usage: $0 [--staged|--all]" >&2
        exit 2
        ;;
esac
