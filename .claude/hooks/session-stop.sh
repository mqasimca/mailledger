#!/usr/bin/env bash
# Session Stop Hook: Runs validation checks when Claude finishes responding
# Ensures code quality before session ends

set -euo pipefail

# Read JSON input from stdin
INPUT=$(cat)

# Extract stop reason
STOP_REASON=$(echo "$INPUT" | jq -r '.stop_reason // "unknown"')

# Only run full checks on explicit stop (not interrupts)
if [[ "$STOP_REASON" == "end_turn" ]]; then
    # Check if any Rust files were modified in this session
    # by looking at recently modified files

    MODIFIED_RS=$(find . -name "*.rs" -mmin -5 2>/dev/null | head -5 || true)

    if [[ -n "$MODIFIED_RS" ]]; then
        echo '{"feedback": "Running quality checks on modified Rust files..."}'

        # Quick format check (non-blocking)
        if ! cargo fmt --check 2>/dev/null; then
            echo '{"feedback": "⚠ Some files need formatting. Run: cargo fmt --all"}'
        fi

        # Quick clippy check (non-blocking, just report)
        CLIPPY_ERRORS=$(cargo clippy --workspace --message-format=short 2>&1 | grep -c "^error" || echo "0")
        if [[ "$CLIPPY_ERRORS" -gt 0 ]]; then
            echo "{\"feedback\": \"⚠ Clippy found $CLIPPY_ERRORS error(s). Run: cargo clippy --workspace\"}"
        fi

        # Check if tests pass (non-blocking)
        if ! cargo test --workspace --quiet 2>/dev/null; then
            echo '{"feedback": "⚠ Some tests are failing. Run: cargo test --workspace"}'
        else
            echo '{"feedback": "✓ All tests passing"}'
        fi
    fi
fi

exit 0
