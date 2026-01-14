# iced GUI Architecture Rules

Reference patterns in `.claude/shared/patterns/` for implementation details.

## The Elm Architecture
→ See: `shared/patterns/elm-architecture.md`

| Component | Purpose |
|-----------|---------|
| Model | Single source of truth |
| Message | Event enum |
| Update | State transition logic |
| View | Render to widgets |

## Async Operations
→ See: `shared/patterns/async-tokio.md`

- Use `Task::perform()` for async commands
- Handle results via Message variants
- Never block in update or view

## File Organization

```
src/
├── main.rs          # Entry, Application trait
├── app.rs           # Model, update
├── message.rs       # Message enum
├── model/           # Sub-state structs
├── view/            # View functions
├── style/           # Theme, widget styles
└── services/        # Async operations
```

## Widget Guidelines

- Use `column![]`, `row![]` macros
- Set explicit sizes: `Length::Fixed` or `Length::Fill`
- Use `spacing()` and `padding()` consistently
- Keep colors in theme, not hardcoded

## Message Design

```rust
enum Message {
    Navigate(Route),
    Account(AccountMessage),
    Mail(MailMessage),
}
```

Group related messages with nested enums.

## Required Lifetime

```rust
fn view(&self) -> Element<'_, Message>
//                        ^^^ required
```
