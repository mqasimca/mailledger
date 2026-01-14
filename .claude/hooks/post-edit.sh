#!/usr/bin/env bash
# Post-Edit Hook: Auto-format and lint after file changes
# Runs rustfmt on Rust files, provides feedback on issues

set -euo pipefail

# Read JSON input from stdin
INPUT=$(cat)

# Extract file path
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Skip if no file path or file doesn't exist
if [[ -z "$FILE_PATH" ]] || [[ ! -f "$FILE_PATH" ]]; then
    exit 0
fi

# Get absolute path
ABS_PATH=$(realpath "$FILE_PATH" 2>/dev/null || echo "$FILE_PATH")

# Handle Rust files
if [[ "$FILE_PATH" == *.rs ]]; then
    # Auto-format with rustfmt
    if command -v rustfmt &> /dev/null; then
        if rustfmt --edition 2024 "$ABS_PATH" 2>/dev/null; then
            echo '{"feedback": "✓ Formatted with rustfmt"}'
        fi
    fi

    # Quick clippy check on the specific file (non-blocking)
    # This provides early feedback without blocking the workflow
    if command -v cargo &> /dev/null; then
        # Find the crate this file belongs to
        CRATE_DIR=$(dirname "$ABS_PATH")
        while [[ "$CRATE_DIR" != "/" ]] && [[ ! -f "$CRATE_DIR/Cargo.toml" ]]; do
            CRATE_DIR=$(dirname "$CRATE_DIR")
        done

        if [[ -f "$CRATE_DIR/Cargo.toml" ]]; then
            CRATE_NAME=$(grep -m1 '^name' "$CRATE_DIR/Cargo.toml" | sed 's/.*"\(.*\)"/\1/' || echo "")
            if [[ -n "$CRATE_NAME" ]]; then
                # Run cargo check in background, capture output
                CLIPPY_OUTPUT=$(cd "$CRATE_DIR" && cargo clippy -p "$CRATE_NAME" --message-format=short 2>&1 || true)
                if echo "$CLIPPY_OUTPUT" | grep -q "^error"; then
                    ERROR_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "^error" || echo "0")
                    echo "{\"feedback\": \"⚠ Clippy found $ERROR_COUNT error(s). Run 'cargo clippy' for details.\"}"
                fi
            fi
        fi
    fi
fi

# Handle TOML files (Cargo.toml)
if [[ "$FILE_PATH" == *.toml ]]; then
    # Validate TOML syntax
    if command -v taplo &> /dev/null; then
        if ! taplo format --check "$ABS_PATH" 2>/dev/null; then
            taplo format "$ABS_PATH" 2>/dev/null || true
            echo '{"feedback": "✓ Formatted TOML file"}'
        fi
    fi
fi

exit 0
