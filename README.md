# MailLedger

Cross-platform desktop email client with custom IMAP implementation in Rust.

## Features

- **Custom IMAP Implementation**: Full RFC 9051 (IMAP4rev2) support with RFC 3501 fallback
- **OAuth2 Authentication**: Support for Gmail, Outlook, Yahoo with PKCE
- **Modern GUI**: Built with [iced](https://github.com/iced-rs/iced) (Elm Architecture)
- **Pure Rust TLS**: No OpenSSL dependency (uses rustls)
- **Cross-Platform**: Linux, Windows, macOS

## Workspace Crates

| Crate | Description |
|-------|-------------|
| [`mailledger`](crates/mailledger) | Desktop GUI application |
| [`mailledger-imap`](crates/mailledger-imap) | IMAP client library (RFC 9051) |
| [`mailledger-smtp`](crates/mailledger-smtp) | SMTP client library (RFC 5321) |
| [`mailledger-oauth`](crates/mailledger-oauth) | OAuth2 authentication |
| [`mailledger-mime`](crates/mailledger-mime) | MIME parsing and generation |
| [`mailledger-core`](crates/mailledger-core) | Core business logic and storage |

## Quick Start

```bash
# Build all crates
cargo build --workspace

# Run the application
RUST_LOG=debug cargo run -p mailledger

# Run tests
cargo test --workspace

# Check code quality
cargo clippy --workspace
cargo fmt --all --check
```

## Requirements

- Rust 1.88+ (Edition 2024)
- Linux: Wayland compositor (X11 not supported)

## Architecture

```
mailledger (GUI)
    |
    +-- mailledger-core (business logic, storage)
            |
            +-- mailledger-imap (IMAP protocol)
            +-- mailledger-smtp (SMTP protocol)
            +-- mailledger-oauth (OAuth2)
            +-- mailledger-mime (MIME parsing)
```

## License

MIT License - see [LICENSE](LICENSE) for details.
