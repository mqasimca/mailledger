#!/usr/bin/env bash
# Pre-Bash Hook: Validates bash commands before execution
# Exit code 0 = allow, 2 = block with feedback

set -euo pipefail

# Read JSON input from stdin
INPUT=$(cat)

# Extract command from tool input
COMMAND=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Block dangerous commands
BLOCKED_PATTERNS=(
    "rm -rf /"
    "rm -rf ~"
    "rm -rf \$HOME"
    "> /dev/sda"
    "mkfs"
    "dd if=/dev/zero"
    ":(){:|:&};:"  # Fork bomb
)

for pattern in "${BLOCKED_PATTERNS[@]}"; do
    if [[ "$COMMAND" == *"$pattern"* ]]; then
        echo '{"block": true, "message": "Blocked dangerous command pattern: '"$pattern"'"}'
        exit 2
    fi
done

# Block force pushes to main/master
if [[ "$COMMAND" =~ git.*push.*--force.*(main|master) ]] || \
   [[ "$COMMAND" =~ git.*push.*-f.*(main|master) ]]; then
    echo '{"block": true, "message": "Force push to main/master is blocked. Use a feature branch."}'
    exit 2
fi

# Allow command
exit 0
