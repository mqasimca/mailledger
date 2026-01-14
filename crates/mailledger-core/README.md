# mailledger-core

Core business logic and services for the MailLedger email client.

## Features

- **Account management**: Create, configure, and manage email accounts
- **Credential storage**: Secure storage via system keyring
- **Email services**: High-level APIs for common email operations
- **Message synchronization**: Sync messages between server and local storage
- **Local storage**: SQLite-based message and account storage

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

## License

MIT License - see [LICENSE](../../LICENSE) for details.
