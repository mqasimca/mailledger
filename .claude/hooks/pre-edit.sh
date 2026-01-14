#!/usr/bin/env bash
# Pre-Edit Hook: Validates file edits before execution
# Exit code 0 = allow, 2 = block with feedback

set -euo pipefail

# Read JSON input from stdin
INPUT=$(cat)

# Extract file path and content
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
NEW_STRING=$(echo "$INPUT" | jq -r '.tool_input.new_string // .tool_input.content // empty')

# Skip if no file path
if [[ -z "$FILE_PATH" ]]; then
    exit 0
fi

# Block editing lock files
LOCK_FILES=("Cargo.lock" "package-lock.json" "yarn.lock" "pnpm-lock.yaml")
FILENAME=$(basename "$FILE_PATH")

for lock in "${LOCK_FILES[@]}"; do
    if [[ "$FILENAME" == "$lock" ]]; then
        echo '{"block": true, "message": "Lock files should not be edited directly. Run the package manager instead."}'
        exit 2
    fi
done

# Block unsafe code in Rust files without SAFETY comment
if [[ "$FILE_PATH" == *.rs ]]; then
    if echo "$NEW_STRING" | grep -q 'unsafe {'; then
        if ! echo "$NEW_STRING" | grep -q '// SAFETY:'; then
            echo '{"block": true, "message": "Rust unsafe blocks require a // SAFETY: comment explaining invariants."}'
            exit 2
        fi
    fi

    # Warn about unwrap() in library code (crates other than main app)
    if [[ "$FILE_PATH" == *"mailledger-imap"* ]] || [[ "$FILE_PATH" == *"mailledger-core"* ]]; then
        if echo "$NEW_STRING" | grep -qE '\.(unwrap|expect)\(\)'; then
            echo '{"feedback": "Warning: unwrap()/expect() in library code. Consider propagating errors with ? operator."}'
        fi
    fi
fi

# Allow edit
exit 0
