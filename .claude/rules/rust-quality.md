# Rust Code Quality Rules

Reference patterns in `.claude/shared/patterns/` for implementation details.

## Error Handling
→ See: `shared/patterns/error-handling.md`

- Libraries: `thiserror` with specific error types
- Application: `anyhow` with context
- NEVER use `unwrap()` or `expect()` in library code

## Type Safety
→ See: `shared/patterns/newtype.md`

- Wrap primitive IDs with newtypes (`Uid`, `SeqNum`, `AccountId`)
- Use `NonZeroU32` where zero is invalid

## State Machines
→ See: `shared/patterns/type-state.md`

- Use type-state pattern for connection states
- Compile-time enforcement of valid transitions

## Construction
→ See: `shared/patterns/builder.md`

- Builder pattern for complex configuration
- Validate in `build()`, not in setters

## Async Code
→ See: `shared/patterns/async-tokio.md`

- Never block the runtime
- Use channels for task communication
- Always handle timeouts

## Testing
→ See: `shared/patterns/testing.md`

- Unit tests in same file with `#[cfg(test)]`
- Property tests with `proptest` for parsers
- Integration tests in `tests/` directory

## File Organization

| Lines | Action |
|-------|--------|
| < 300 | Ideal size |
| 300-400 | Consider splitting if multiple concepts |
| 400-500 | Should split into submodules |
| > 500 | Must split - file is too large |

**Splitting Guidelines:**
- One concept per file (one struct + impl, one enum, one trait)
- Group related types in a submodule with `mod.rs`
- Tests don't count toward limit (they're in `#[cfg(test)]`)
- Prefer many small files over few large files

## Code Reuse (DRY)

**NEVER duplicate code** - extract helper functions instead:

- If code appears 2+ times → extract to a helper function
- If logic is shared across modules → create a `helpers.rs` or `utils.rs`
- If pattern repeats across crates → consider a shared utility crate

**Helper Function Guidelines:**
- Name helpers by what they do, not where they're used
- Keep helpers focused (single responsibility)
- Place helpers near their primary usage
- Use `pub(crate)` for internal helpers, `pub(super)` for module-local

**Examples:**
```rust
// BAD: Duplicated validation
fn create_user(id: u32) { if id == 0 { panic!() } ... }
fn update_user(id: u32) { if id == 0 { panic!() } ... }

// GOOD: Extract helper
fn validate_id(id: u32) -> Result<NonZeroU32> { ... }
fn create_user(id: u32) { let id = validate_id(id)?; ... }
fn update_user(id: u32) { let id = validate_id(id)?; ... }
```

## Quick Reference

| Pattern | When to Use |
|---------|-------------|
| Newtype | IDs, wrapped primitives |
| Type-State | State machine APIs |
| Builder | Many optional fields |
| Repository | Storage abstraction |

## Anti-Patterns

- `clone()` to satisfy borrow checker → understand ownership
- `Box<dyn Error>` → use concrete types
- `unsafe` without `// SAFETY:` → document invariants
- Blocking in async → use `spawn_blocking`
- Duplicated code → extract helper functions (DRY principle)
