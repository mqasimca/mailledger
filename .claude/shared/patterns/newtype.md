# Newtype Pattern

Wrap primitive types to provide type safety and prevent mixing up similar values.

## Basic Newtype

```rust
use std::num::NonZeroU32;

/// Unique identifier for a message within a mailbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uid(NonZeroU32);

impl Uid {
    /// Create a new UID from a non-zero value.
    pub fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    /// Get the raw numeric value.
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

/// Message sequence number (position in mailbox).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SeqNum(NonZeroU32);
```

## Why This Works

```rust
// Without newtypes - easy to mix up:
fn fetch(uid: u32, seq: u32) { }
fetch(seq, uid);  // Compiles but WRONG!

// With newtypes - compiler catches mistakes:
fn fetch(uid: Uid, seq: SeqNum) { }
fetch(seq, uid);  // Compile ERROR!
```

## Common Newtypes for Email

| Type | Wraps | Purpose |
|------|-------|---------|
| `Uid` | `NonZeroU32` | Unique message ID |
| `SeqNum` | `NonZeroU32` | Message position |
| `AccountId` | `String` | Account identifier |
| `FolderId` | `String` | Folder/mailbox identifier |
| `MessageId` | `String` | Email Message-ID header |

## With Parsing

```rust
impl std::str::FromStr for Uid {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n: u32 = s.parse()?;
        let nz = NonZeroU32::new(n)
            .ok_or(ParseError::ZeroUid)?;
        Ok(Self(nz))
    }
}
```
