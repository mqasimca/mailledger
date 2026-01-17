# MailLedger

Modern, cross-platform desktop email client built with Rust.

## Features

- **📧 Native Email Client**: Full-featured desktop email experience
- **🔒 Secure Authentication**: Support for OAuth2 (Gmail, Outlook) and traditional login
- **⚡ Real-time Sync**: IDLE push notifications for instant message updates
- **🎨 Modern Interface**: Clean, responsive UI built with iced framework
- **🔐 Secure Storage**: Credentials stored in system keyring
- **🌐 Cross-platform**: Linux, Windows, and macOS support
- **🚀 Custom IMAP/SMTP**: Built on production-quality Rust implementations

## Why OAuth2?

**Microsoft permanently disabled Basic Authentication for IMAP/SMTP in April 2026.** MailLedger provides full OAuth2 support for modern email providers:

- ✅ Microsoft Outlook / Office 365
- ✅ Gmail / Google Workspace
- ✅ Yahoo Mail
- ✅ Custom OAuth2 providers

Traditional username/password authentication is still supported for providers that allow it.

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/mqasimca/mailledger.git
cd mailledger

# Build the application
cargo build --release

# Run MailLedger
cargo run --release -p mailledger
```

### System Requirements

- **Linux**: Wayland compositor (Gnome, KDE Plasma, Sway, etc.)
- **Windows**: Windows 10 or later
- **macOS**: macOS 10.15 or later

## Quick Start

1. **Launch MailLedger**
   ```bash
   cargo run --release -p mailledger
   ```

2. **Add Your Account**
   - Click "Add Account" on first launch
   - Choose OAuth2 (recommended) or traditional login
   - Follow the authentication flow

3. **Configure OAuth2** (if using Gmail/Outlook)
   - For Outlook: Register app at [Azure Portal](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade)
   - For Gmail: Create project at [Google Cloud Console](https://console.cloud.google.com/)
   - Add your client ID to the app settings

4. **Start Using MailLedger**
   - Browse folders in the sidebar
   - Read and compose messages
   - Real-time sync with IDLE support

## Features in Detail

### Email Management
- **Folder Navigation**: Access all your IMAP folders
- **Message List**: View messages with sender, subject, date, and preview
- **Message Reading**: Full HTML and plain text support with inline images
- **Compose**: Create and send new messages
- **Reply/Forward**: Quick response actions

### Security
- **OAuth2 Support**: Secure, token-based authentication
- **System Keyring**: Credentials stored using OS-native secure storage
- **TLS Encryption**: All connections use rustls for secure communication
- **No Password Storage**: OAuth2 means no passwords in the app

### Performance
- **Real-time Updates**: IDLE monitoring for instant message notifications
- **Efficient Sync**: Only fetches what's needed
- **Responsive UI**: Built with iced for smooth 60fps rendering
- **Fast Startup**: Optimized binary with LTO and strip

## Architecture

MailLedger is built on a modular architecture:

- **mailledger** (this crate): GUI application using iced framework
- **[mailledger-imap](https://crates.io/crates/mailledger-imap)**: Full IMAP4rev2 client library
- **[mailledger-smtp](https://crates.io/crates/mailledger-smtp)**: SMTP client for sending emails
- **[mailledger-oauth](https://crates.io/crates/mailledger-oauth)**: OAuth2 authentication library
- **[mailledger-core](https://crates.io/crates/mailledger-core)**: Business logic and services
- **[mailledger-mime](https://crates.io/crates/mailledger-mime)**: MIME parsing and generation

## Development

### Building

```bash
# Debug build
cargo build -p mailledger

# Release build (optimized)
cargo build --release -p mailledger

# Run with logging
RUST_LOG=debug cargo run -p mailledger
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p mailledger-imap
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint with Clippy
cargo clippy --workspace

# Check all together
cargo fmt --all && cargo clippy --workspace && cargo test --workspace
```

## Platform Support

### Linux
- **Display**: Wayland only (no X11)
- **Keyring**: GNOME Keyring or KWallet via Secret Service
- **Tested on**: Gnome, KDE Plasma, Sway

### Windows
- **Minimum**: Windows 10
- **Keyring**: Windows Credential Manager
- **Graphics**: DirectX 12

### macOS
- **Minimum**: macOS 10.15 (Catalina)
- **Keyring**: macOS Keychain
- **Graphics**: Metal

## Configuration

Configuration and data are stored in platform-specific directories:

- **Linux**: `~/.config/mailledger/` and `~/.local/share/mailledger/`
- **Windows**: `%APPDATA%\mailledger\`
- **macOS**: `~/Library/Application Support/mailledger/`

## Roadmap

- [x] OAuth2 authentication
- [x] IMAP folder and message browsing
- [x] Message reading with HTML support
- [x] Real-time IDLE sync
- [x] Dark/light theme
- [ ] Message composition and sending
- [ ] Search functionality
- [ ] Multiple account support
- [ ] Offline mode
- [ ] Message filtering and rules

## Contributing

Contributions are welcome! Please read the [CLAUDE.md](../../CLAUDE.md) for development guidelines.

## License

MIT License - see [LICENSE](../../LICENSE) for details.

## Acknowledgments

Built with:
- [iced](https://github.com/iced-rs/iced) - GUI framework
- [tokio](https://tokio.rs/) - Async runtime
- [rustls](https://github.com/rustls/rustls) - TLS implementation
- [sqlx](https://github.com/launchbadge/sqlx) - Database access
