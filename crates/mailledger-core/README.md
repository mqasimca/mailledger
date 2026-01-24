# mailledger-core

Core business logic and services for the MailLedger email client.

## Features

- **Account management**: Create, configure, and manage email accounts
- **Credential storage**: Secure storage via system keyring
- **Email services**: High-level APIs for common email operations
- **Message synchronization**: Sync messages between server and local storage
- **Local storage**: SQLite-based message and account storage
- **Offline caching**: Cache messages and content for offline access
- **Contact management**: Store and search email contacts
- **Snooze functionality**: Postpone messages with timed reminders
- **Sender triage**: Screen first-time senders and categorize inbox

## Components

### Account Management

```rust
use mailledger_core::{Account, AccountId, ImapConfig, SmtpConfig, Security};

// Create account configuration
let account = Account {
    id: AccountId::new(1),
    name: "Work Email".to_string(),
    email: "user@company.com".to_string(),
    imap: ImapConfig {
        host: "imap.company.com".to_string(),
        port: 993,
        security: Security::Implicit,
    },
    smtp: SmtpConfig {
        host: "smtp.company.com".to_string(),
        port: 587,
        security: Security::StartTls,
    },
};
```

### Credential Storage

```rust
use mailledger_core::credentials;

// Store credentials securely in system keyring
credentials::store_password(account_id, "password")?;
credentials::store_oauth_token(account_id, &token)?;

// Retrieve credentials
let password = credentials::get_password(account_id)?;
let token = credentials::get_oauth_token(account_id)?;
```

### Email Services

```rust
use mailledger_core::{connect_and_login, list_folders, fetch_messages};

// Connect and authenticate
let client = connect_and_login(&account, &credentials).await?;

// List folders
let folders = list_folders(&client).await?;

// Fetch messages
let messages = fetch_messages(&client, "INBOX", 1..=50).await?;
```

### Offline Caching

```rust
use mailledger_core::{CacheRepository, CachedMessageSummary};

// Initialize cache
let repo = CacheRepository::new("cache.db").await?;

// Cache message summaries for offline access
repo.cache_summaries(&messages).await?;

// Retrieve cached messages when offline
let cached = repo.get_summaries(account_id, "INBOX").await?;
```

### Contact Management

```rust
use mailledger_core::{ContactRepository, Contact};

let repo = ContactRepository::new("contacts.db").await?;

// Add contact
let contact = Contact {
    email: "colleague@company.com".to_string(),
    display_name: Some("John Doe".to_string()),
    ..Default::default()
};
repo.add_or_update(&contact).await?;

// Search contacts
let results = repo.search("john").await?;
```

### Sender Triage

```rust
use mailledger_core::{TriageRepository, InboxCategory};

let repo = TriageRepository::new("triage.db").await?;

// Get pending senders (first-time emailers)
let pending = repo.get_pending_senders(account_id).await?;

// Approve sender to Primary inbox
repo.approve_sender(account_id, "trusted@company.com", InboxCategory::Primary).await?;

// Block sender
repo.block_sender(account_id, "spam@example.com").await?;
```

### Snooze Messages

```rust
use mailledger_core::{SnoozeRepository, SnoozeItem};
use chrono::{Utc, Duration};

let repo = SnoozeRepository::new("snooze.db").await?;

// Snooze message for 1 hour
let snooze_until = Utc::now() + Duration::hours(1);
repo.snooze_message(account_id, "INBOX", uid, snooze_until).await?;

// Get due snoozes
let due = repo.get_due_snoozes(account_id).await?;
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
