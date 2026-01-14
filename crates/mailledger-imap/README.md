# mailledger-imap

Production-quality IMAP client library implementing RFC 9051 (IMAP4rev2) with fallback support for RFC 3501 (IMAP4rev1).

## Features

- **Type-state connection management**: Compile-time enforcement of valid IMAP state transitions (`NotAuthenticated` → `Authenticated` → `Selected`)
- **Full protocol support**: LOGIN, SELECT, FETCH, STORE, COPY, MOVE, SEARCH, APPEND, EXPUNGE, and more
- **IDLE support**: Real-time push notifications via RFC 2177
- **TLS via rustls**: Secure connections without OpenSSL dependency
- **Server quirks handling**: Built-in workarounds for Gmail, Outlook, Dovecot, and other common servers
- **Sans-I/O parser**: Protocol parsing separated from network I/O

## Quick Start

```rust
use mailledger_imap::{Client, Config, Security, FetchItems};

#[tokio::main]
async fn main() -> mailledger_imap::Result<()> {
    // Connect with TLS
    let config = Config::new("imap.example.com", Security::Implicit);
    let stream = mailledger_imap::connection::connect_tls(&config).await?;
    let client = Client::from_stream(stream).await?;

    // Authenticate
    let mut client = client.login("user@example.com", "password").await?;

    // List folders
    let folders = client.list("", "*").await?;
    for folder in &folders {
        println!("Folder: {}", folder.mailbox.as_str());
    }

    // Select INBOX
    let (mut client, status) = client.select("INBOX").await?;
    println!("Messages: {}", status.exists);

    client.logout().await?;
    Ok(())
}
```

## Connection States

The library uses the type-state pattern to enforce valid IMAP operations at compile time:

```text
┌─────────────────────┐
│   NotAuthenticated  │ ─── login() ───→ Authenticated
└─────────────────────┘
           │
           ▼
┌─────────────────────┐
│    Authenticated    │ ─── select()/examine() ───→ Selected
└─────────────────────┘
           │
           ▼
┌─────────────────────┐
│      Selected       │ ─── close() ───→ Authenticated
└─────────────────────┘
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
