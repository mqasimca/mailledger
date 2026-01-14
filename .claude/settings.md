# Claude Code Configuration

This document describes the `.claude` folder structure and configuration for the MailLedger project.

## Folder Structure

```
.claude/
├── settings.json         # Main configuration (hooks, env, permissions)
├── settings.local.json   # Personal overrides (gitignored)
├── settings.md           # This documentation
├── hooks/                # Shell scripts for lifecycle hooks
│   ├── pre-bash.sh       # Validates bash commands
│   ├── pre-edit.sh       # Validates file edits
│   ├── post-edit.sh      # Auto-format after edits
│   ├── prompt-submit.sh  # Context injection on prompt
│   └── session-stop.sh   # Quality checks on stop
├── commands/             # Custom slash commands
│   ├── check.md          # /check - run all quality checks
│   ├── fix.md            # /fix - auto-fix issues
│   ├── test.md           # /test - run test suite
│   └── build.md          # /build - build project
├── rules/                # Coding guidelines
│   ├── rust-quality.md   # Rust best practices
│   └── iced-architecture.md  # GUI architecture rules
└── agents/               # Custom subagents (if needed)
```

## Hook Events

| Event | When | Purpose |
|-------|------|---------|
| `PreToolUse` | Before tool runs | Block dangerous commands, validate edits |
| `PostToolUse` | After tool completes | Auto-format, run quick lints |
| `UserPromptSubmit` | Prompt submitted | Inject context, suggest resources |
| `Stop` | Claude finishes | Run full quality checks |

## Hook Exit Codes

| Code | Effect |
|------|--------|
| `0` | Success, allow action |
| `1` | Error, but allow action |
| `2` | Block action, send feedback to Claude |

## Hook JSON Output

```json
{
  "block": true,              // Block the action (PreToolUse only)
  "message": "User message",  // Show to user
  "feedback": "Claude info",  // Send to Claude (non-blocking)
  "suppressOutput": false     // Hide command output
}
```

## Custom Commands

Use with `/project:<command>` or just `/<command>`:

- `/check` - Run cargo fmt, clippy, and tests
- `/fix` - Auto-fix formatting and clippy issues
- `/test` - Run test suite with options
- `/build` - Build project (debug/release)

## Permissions

The `permissions.deny` array blocks access to sensitive files:
- `.env` files
- Secrets directories
- Private keys (`.pem`, `.key`)

## Environment Variables

Set in `env` section of settings.json:
- `RUST_BACKTRACE=1` - Full backtraces on panic
- `RUST_LOG=debug` - Debug logging level

## Local Overrides

Create `.claude/settings.local.json` for personal settings:
- Not committed to git
- Overrides settings.json values
- Good for different timeout values or additional hooks
