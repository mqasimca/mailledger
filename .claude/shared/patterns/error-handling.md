# Error Handling Patterns

## Library Errors (thiserror)

Use in: `mailledger-imap`, `mailledger-core`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Rules
- NEVER use `unwrap()` or `expect()` - propagate with `?`
- Each error variant has descriptive `#[error("...")]`
- Use `#[from]` for automatic conversion
- Include source errors with `#[source]`

## Application Errors (anyhow)

Use in: `mailledger` (GUI app)

```rust
use anyhow::{Context, Result};

async fn load_account(id: &str) -> Result<Account> {
    let config = read_config()
        .await
        .context("Failed to read config")?;

    config.accounts
        .get(id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Account not found: {id}"))
}
```

### Rules
- Add context with `.context("description")`
- Use `anyhow::anyhow!()` for ad-hoc errors
- Log errors before displaying to user
