# Shared Resources

Reusable patterns, snippets, and templates for the MailLedger project.

## Structure

```
shared/
├── patterns/           # Design pattern documentation
│   ├── error-handling.md
│   ├── newtype.md
│   ├── type-state.md
│   ├── builder.md
│   ├── elm-architecture.md
│   ├── async-tokio.md
│   └── testing.md
├── snippets/           # Code templates
│   ├── lib-crate.rs
│   ├── error-type.rs
│   └── iced-app.rs
└── templates/          # File templates (future)
```

## Patterns

| Pattern | Description | When to Use |
|---------|-------------|-------------|
| **Error Handling** | thiserror vs anyhow | Every crate |
| **Newtype** | Type-safe wrappers | IDs, amounts, indices |
| **Type-State** | Compile-time state machines | APIs with state |
| **Builder** | Fluent object construction | Complex config |
| **Elm Architecture** | Model-View-Update | iced GUI |
| **Async/Tokio** | Async runtime patterns | All async code |
| **Testing** | Test organization | Every module |

## Snippets

| Snippet | Use For |
|---------|---------|
| `lib-crate.rs` | Starting a new library crate |
| `error-type.rs` | Defining error enums |
| `iced-app.rs` | iced application boilerplate |

## Usage

Reference in code comments:
```rust
// Pattern: .claude/shared/patterns/type-state.md
```

Reference in other docs:
```markdown
→ See: `shared/patterns/error-handling.md`
```
