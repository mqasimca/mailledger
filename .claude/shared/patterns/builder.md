# Builder Pattern

Construct complex objects step-by-step with a fluent API.

## Basic Builder

```rust
#[derive(Debug, Clone)]
pub struct FetchCommand {
    sequence: SequenceSet,
    items: Vec<FetchItem>,
    uid: bool,
    changed_since: Option<u64>,
}

#[derive(Debug, Default)]
pub struct FetchCommandBuilder {
    sequence: Option<SequenceSet>,
    items: Vec<FetchItem>,
    uid: bool,
    changed_since: Option<u64>,
}

impl FetchCommandBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sequence(mut self, seq: SequenceSet) -> Self {
        self.sequence = Some(seq);
        self
    }

    pub fn item(mut self, item: FetchItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = FetchItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn uid(mut self) -> Self {
        self.uid = true;
        self
    }

    pub fn changed_since(mut self, modseq: u64) -> Self {
        self.changed_since = Some(modseq);
        self
    }

    pub fn build(self) -> Result<FetchCommand, BuildError> {
        let sequence = self.sequence
            .ok_or(BuildError::MissingField("sequence"))?;

        if self.items.is_empty() {
            return Err(BuildError::MissingField("items"));
        }

        Ok(FetchCommand {
            sequence,
            items: self.items,
            uid: self.uid,
            changed_since: self.changed_since,
        })
    }
}
```

## Usage

```rust
let cmd = FetchCommandBuilder::new()
    .sequence(SequenceSet::range(1, 10))
    .item(FetchItem::Flags)
    .item(FetchItem::Envelope)
    .uid()
    .build()?;
```

## With derive_builder (optional)

```rust
use derive_builder::Builder;

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct ConnectionConfig {
    host: String,
    port: u16,
    #[builder(default = "Security::Implicit")]
    security: Security,
    #[builder(default = "Duration::from_secs(30)")]
    timeout: Duration,
}
```

## When to Use

- Struct has many optional fields
- Construction requires validation
- Want fluent, readable API
- Avoid constructors with many parameters
