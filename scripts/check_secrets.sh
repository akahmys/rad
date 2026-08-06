#!/bin/bash
set -euo pipefail

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

# Obfuscated patterns to prevent the script from matching itself
PATH_PATTERN_USERS=$(echo -e "/Us""ers/[a-zA-Z0-9_-]+")
PATH_PATTERN_HOME=$(echo -e "/ho""me/[a-zA-Z0-9_-]+")

API_KEY_PATTERN=$(echo -e "sk-[a-zA-Z0-9]{20,}")
ANT_KEY_PATTERN=$(echo -e "sk-ant-[a-zA-Z0-9_-]{20,}")
ASSIGN_SECRET_PATTERN=$(echo -e "[a-zA-Z0-9_-]*(api[-_]?key|secret|token|password|credential|auth)[a-zA-Z0-9_-]*\\s*[:=]\\s*[\"'][a-zA-Z0-9_\\-\\.\\~]{8,}[\"']")

# Values that name *where* a credential goes rather than being one. The
# assignment pattern keys on any identifier containing "auth", so a field like
# `auth_header: "Authorization"` trips it — the value is an HTTP header name,
# not a secret. Allowlisting exact literals keeps detection intact: a real
# `auth_token = "sk-..."` still fails, because its value is not on this list.
ASSIGN_SECRET_ALLOWLIST=$(echo -e "[\"'](Authorization|Bearer|Basic|Proxy-Authorization|WWW-Authenticate|X-Api-Key|x-api-key|api-key|anthropic-version)[\"']")

echo "=== Secret & Absolute Path Scanner ==="

MODE="staged"
if [ "${1:-}" = "--all" ]; then
    MODE="all"
fi

FAILED=0

# Arguments:
# $1: file path
# $2: content to scan
check_content() {
    local file_path="$1"
    local content="$2"
    local local_failed=0

    # We exclude .agents/ directory from absolute path checks because agent rules
    # require absolute file paths to function correctly.
    local skip_path_check=0
    if [[ "$file_path" == .agents/* ]]; then
        skip_path_check=1
    fi

    if [ "$skip_path_check" -eq 0 ]; then
        # 1. Absolute path /Users/
        if echo "$content" | grep -E "$PATH_PATTERN_USERS" > /dev/null; then
            echo -e "${RED}ERROR: Absolute path containing '${PATH_PATTERN_USERS}' detected in $file_path:${NC}"
            echo "$content" | grep -n -E "$PATH_PATTERN_USERS" || true
            local_failed=1
        fi

        # 2. Absolute path /home/
        if echo "$content" | grep -E "$PATH_PATTERN_HOME" > /dev/null; then
            echo -e "${RED}ERROR: Absolute path containing '${PATH_PATTERN_HOME}' detected in $file_path:${NC}"
            echo "$content" | grep -n -E "$PATH_PATTERN_HOME" || true
            local_failed=1
        fi
    fi

    # 3. API Key sk-...
    if echo "$content" | grep -E "$API_KEY_PATTERN" > /dev/null; then
        echo -e "${RED}ERROR: Potential API Key (sk-...) detected in $file_path:${NC}"
        echo "$content" | grep -n -E "$API_KEY_PATTERN" || true
        local_failed=1
    fi

    # 4. Anthropic API Key sk-ant-...
    if echo "$content" | grep -E "$ANT_KEY_PATTERN" > /dev/null; then
        echo -e "${RED}ERROR: Potential Anthropic API Key (sk-ant-...) detected in $file_path:${NC}"
        echo "$content" | grep -n -E "$ANT_KEY_PATTERN" || true
        local_failed=1
    fi

    # 5. Secret assignment variable
    local assign_hits
    # The trailing `|| true` is required, not defensive: under `set -e` with
    # `pipefail`, a file with zero matches makes the first grep exit 1, which
    # aborts the whole scan silently — every later file goes unscanned.
    assign_hits=$(echo "$content" | grep -n -E -i "$ASSIGN_SECRET_PATTERN" \
        | { grep -v -E -i "$ASSIGN_SECRET_ALLOWLIST" || true; } || true)
    if [ -n "$assign_hits" ]; then
        echo -e "${RED}ERROR: Hardcoded secret assignment detected in $file_path:${NC}"
        echo "$assign_hits"
        local_failed=1
    fi

    if [ "$local_failed" -ne 0 ]; then
        FAILED=1
    fi
}

if [ "$MODE" = "staged" ]; then
    # Get staged files
    staged_files=$(git diff --cached --name-only --diff-filter=d)
    if [ -z "$staged_files" ]; then
        echo "No staged changes to scan."
        exit 0
    fi

    echo "Scanning staged changes..."
    for file in $staged_files; do
        if [ ! -f "$file" ]; then
            continue
        fi

        # Skip this script itself and git hook files
        if [[ "$file" == scripts/check_secrets.sh ]] || [[ "$file" == .git/hooks/* ]]; then
            continue
        fi

        # Get staged changes (only lines added, stripped of the leading '+')
        diff_content=$(git diff --cached "$file" | grep -E '^\+[^+]' | sed 's/^+//' || true)
        if [ -n "$diff_content" ]; then
            check_content "$file" "$diff_content"
        fi
    done
else
    echo "Scanning all repository files..."
    all_files=$(git ls-files)
    for file in $all_files; do
        if [ ! -f "$file" ]; then
            continue
        fi

        # Skip this script itself and git hook files
        if [[ "$file" == scripts/check_secrets.sh ]] || [[ "$file" == .git/hooks/* ]]; then
            continue
        fi

        file_content=$(cat "$file")
        check_content "$file" "$file_content"
    done
fi

if [ $FAILED -ne 0 ]; then
    echo -e "${RED}Scan failed. Please remove absolute paths and secrets before committing.${NC}"
    exit 1
else
    echo -e "${GREEN}Scan passed successfully.${NC}"
    exit 0
fi
