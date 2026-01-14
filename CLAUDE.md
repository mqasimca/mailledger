# MailLedger

Cross-platform desktop email client with custom IMAP implementation in Rust.

## Quick Reference

| Aspect | Choice |
|--------|--------|
| Language | Rust (Edition 2024) |
| GUI | iced (Elm Architecture) |
| Async | tokio |
| TLS | rustls |
| Platforms | Linux, Windows, macOS |

## Commands

```bash
cargo build --workspace          # Build
cargo test --workspace           # Test
cargo clippy --workspace         # Lint
cargo fmt --all                  # Format
RUST_LOG=debug cargo run -p mailledger  # Run
```

## Workspace

```
crates/
├── mailledger-imap/    # IMAP protocol library
├── mailledger-core/    # Business logic, storage
└── mailledger/         # GUI application
```

## Patterns

See `.claude/shared/patterns/` for detailed implementations:

| Pattern | File | Use Case |
|---------|------|----------|
| Error Handling | `error-handling.md` | thiserror vs anyhow |
| Newtype | `newtype.md` | Type-safe IDs |
| Type-State | `type-state.md` | Connection states |
| Builder | `builder.md` | Complex construction |
| Elm Architecture | `elm-architecture.md` | iced GUI |
| Async | `async-tokio.md` | Tokio patterns |
| Testing | `testing.md` | Unit, property, integration |

## Code Snippets

See `.claude/shared/snippets/` for templates:
- `lib-crate.rs` - New library crate template
- `error-type.rs` - Error enum template
- `iced-app.rs` - iced application template

## Rules

See `.claude/rules/` for guidelines:
- `rust-quality.md` - Rust best practices
- `iced-architecture.md` - GUI architecture

## Critical Rules

1. **NO** `unwrap()` / `expect()` in libraries → use `?`
2. **NO** `unsafe` without `// SAFETY:` comment
3. **NO** blocking in async → use `spawn_blocking`
4. **NO** `clone()` to satisfy borrow checker → understand ownership

## Git

- Imperative commit messages ("Add feature" not "Added")
- Run `cargo fmt && cargo clippy && cargo test` before commit
