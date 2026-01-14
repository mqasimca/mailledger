# Type-State Pattern

Use the type system to enforce valid state transitions at compile time.

## IMAP Connection States

```rust
use std::marker::PhantomData;

// State marker types (zero-sized)
pub struct NotAuthenticated;
pub struct Authenticated;
pub struct Selected;

// Connection parameterized by state
pub struct Connection<S> {
    stream: TlsStream,
    capabilities: Vec<Capability>,
    _state: PhantomData<S>,
}
```

## State-Specific Methods

```rust
// Only available when NOT authenticated
impl Connection<NotAuthenticated> {
    pub async fn login(self, user: &str, pass: &str)
        -> Result<Connection<Authenticated>>
    {
        // ... perform login ...
        Ok(Connection {
            stream: self.stream,
            capabilities: new_caps,
            _state: PhantomData,
        })
    }
}

// Only available when authenticated
impl Connection<Authenticated> {
    pub async fn select(self, mailbox: &str)
        -> Result<Connection<Selected>>
    {
        // ... select mailbox ...
    }

    pub async fn list(&self, pattern: &str) -> Result<Vec<Mailbox>> {
        // ... list mailboxes ...
    }
}

// Only available when mailbox selected
impl Connection<Selected> {
    pub async fn fetch(&self, uids: &[Uid]) -> Result<Vec<Message>> {
        // ... fetch messages ...
    }

    pub async fn close(self) -> Result<Connection<Authenticated>> {
        // ... close mailbox, return to authenticated ...
    }
}
```

## State Diagram

```
┌─────────────────────┐
│  NotAuthenticated   │
└──────────┬──────────┘
           │ login() / authenticate()
           ▼
┌─────────────────────┐
│    Authenticated    │◄─────┐
└──────────┬──────────┘      │
           │ select()        │ close()
           ▼                 │
┌─────────────────────┐      │
│      Selected       │──────┘
└─────────────────────┘
```

## Benefits

- **Compile-time safety**: Can't call `fetch()` without `select()` first
- **Self-documenting**: API shows valid state transitions
- **Zero runtime cost**: PhantomData is zero-sized
