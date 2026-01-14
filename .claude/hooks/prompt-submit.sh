#!/usr/bin/env bash
# UserPromptSubmit Hook: Runs when user submits a prompt
# Can inject context or provide early feedback

set -euo pipefail

# Read JSON input from stdin
INPUT=$(cat)

# Extract the prompt text
PROMPT=$(echo "$INPUT" | jq -r '.prompt // empty')

# Provide helpful context based on prompt keywords
if echo "$PROMPT" | grep -qi "imap\|email\|mail"; then
    echo '{"feedback": "IMAP context: See local/01-imap-library.md for implementation plan"}'
fi

if echo "$PROMPT" | grep -qi "gui\|ui\|iced\|view"; then
    echo '{"feedback": "GUI context: See local/02-iced-gui.md for UI architecture"}'
fi

if echo "$PROMPT" | grep -qi "test"; then
    echo '{"feedback": "Testing: Use cargo test --workspace. Property tests use proptest crate."}'
fi

# Success - allow prompt to proceed
echo '{"message": "Success"}'
exit 0
