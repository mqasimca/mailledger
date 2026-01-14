# mailledger-oauth

OAuth2 authentication library for email protocols (IMAP/SMTP).

## Features

- ✅ **Authorization Code Flow** with PKCE support (RFC 7636)
- ✅ **Device Flow** for CLI/IoT applications (RFC 8628)
- ✅ **Token Management** with automatic expiration tracking
- ✅ **Token Refresh** capability
- ✅ **SASL Mechanisms**: XOAUTH2 (Google/Microsoft) and OAUTHBEARER (RFC 7628)
- ✅ **Pre-configured Providers**: Gmail, Outlook/Microsoft 365, Yahoo
- ✅ **Custom Provider** support

## Quick Start

### Prerequisites

For Microsoft Outlook/Office 365:
1. Register application at [Azure Portal](https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade)
2. Configure redirect URI (e.g., `http://localhost:8080`)
3. Note your Application (client) ID
4. Obtain necessary permissions (IMAP.AccessAsUser.All, SMTP.Send, offline_access)

For Gmail:
1. Create project at [Google Cloud Console](https://console.cloud.google.com/)
2. Enable Gmail API
3. Configure OAuth consent screen
4. Create OAuth 2.0 credentials
5. Add `https://mail.google.com/` scope

### Authorization Code Flow (Desktop Apps)

```rust
use mailledger_oauth::{Provider, OAuthClient, AuthorizationCodeFlow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure provider (Microsoft Outlook)
    let provider = Provider::microsoft()?;

    // Create OAuth client
    let client = OAuthClient::new("your-client-id", provider)
        .with_redirect_uri("http://localhost:8080");

    // Create authorization flow with PKCE (recommended)
    let flow = AuthorizationCodeFlow::new(client).with_pkce();

    // Generate authorization URL
    let auth_url = flow.authorization_url(None, Some("random-state"))?;
    println!("Visit: {}", auth_url);

    // After user authorizes, you'll receive a code
    // Exchange code for token
    let code = "authorization-code-from-redirect";
    let token = flow.exchange_code(code, None).await?;

    println!("Access token: {}", token.access_token);
    println!("Expires at: {:?}", token.expires_at);

    Ok(())
}
```

### Using with IMAP

```rust
use mailledger_imap::{Client, Config, Security};
use mailledger_oauth::{Provider, OAuthClient, AuthorizationCodeFlow, Token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Obtain OAuth2 token (see above)
    let token: Token = /* ... */;

    // 2. Connect to IMAP server
    let config = Config::new("outlook.office365.com", Security::Tls);
    let stream = mailledger_imap::connection::connect_tls(&config).await?;
    let client = Client::from_stream(stream).await?;

    // 3. Authenticate with OAuth2 (XOAUTH2)
    let email = "user@outlook.com";
    let client = client.authenticate_xoauth2(email, &token).await?;

    // 4. Use authenticated client
    let folders = client.list("", "*").await?;
    println!("Folders: {:#?}", folders);

    Ok(())
}
```

### Token Storage

Store tokens securely using the credential storage:

```rust
use mailledger_core::account::{AccountId, credentials};

// Store token
let account_id = AccountId::new(1);
credentials::store_oauth_token(account_id, &token)?;

// Retrieve token later
let stored_token = credentials::get_oauth_token(account_id)?;

// Delete token
credentials::delete_oauth_token(account_id)?;
```

### Token Refresh

```rust
// Check if token is expired (with 60-second buffer)
if token.is_expired() {
    // Refresh the token
    let new_token = client.refresh_token(&token).await?;

    // Store the new token
    credentials::store_oauth_token(account_id, &new_token)?;
}
```

### Device Flow (CLI/IoT Apps)

For applications that can't easily open a browser:

```rust
use mailledger_oauth::{Provider, OAuthClient, DeviceFlow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::microsoft()?;
    let client = OAuthClient::new("your-client-id", provider);
    let flow = DeviceFlow::new(client);

    // Request device authorization
    let auth = flow.request_device_authorization(None).await?;

    println!("Visit: {}", auth.verification_uri);
    println!("Enter code: {}", auth.user_code);

    // Poll for token (automatically retries)
    let (_, token) = flow.authorize(None, 120).await?;

    println!("Authorized! Token: {}", token.access_token);
    Ok(())
}
```

## Providers

### Microsoft Outlook/Office 365

```rust
let provider = Provider::microsoft()?;
// Scopes:
// - https://outlook.office365.com/IMAP.AccessAsUser.All
// - https://outlook.office365.com/SMTP.Send
// - offline_access (for refresh tokens)
```

### Gmail

```rust
let provider = Provider::google()?;
// Scope: https://mail.google.com/
```

### Yahoo

```rust
let provider = Provider::yahoo()?;
// Scopes: mail-r, mail-w
```

### Custom Provider

```rust
let provider = Provider::new(
    "Custom",
    "https://auth.example.com/authorize",
    "https://auth.example.com/token",
)?
.with_default_scopes(vec!["email".to_string()]);
```

## SASL Mechanisms

### XOAUTH2 (Google/Microsoft)

```rust
use mailledger_oauth::sasl::xoauth2_response;

let auth_string = xoauth2_response("user@example.com", &token.access_token);
// Use with IMAP: AUTHENTICATE XOAUTH2 {auth_string}
```

### OAUTHBEARER (RFC 7628)

```rust
use mailledger_oauth::sasl::oauthbearer_response;

let auth_string = oauthbearer_response("user@example.com", &token.access_token);
// Use with IMAP: AUTHENTICATE OAUTHBEARER {auth_string}
```

## Examples

Run the Outlook OAuth2 example:

```bash
export OAUTH_CLIENT_ID="your-client-id"
export OAUTH_EMAIL="your-email@outlook.com"
cargo run --example outlook_oauth2
```

## Security Notes

1. **Always use PKCE** for desktop/mobile applications (enabled by default with `.with_pkce()`)
2. **Store tokens securely** using system keyring (see `mailledger-core::credentials`)
3. **Never commit** client secrets or tokens to version control
4. **Validate state parameter** to prevent CSRF attacks
5. **Use TLS** for all token exchanges

## Why OAuth2?

**Microsoft will permanently disable Basic Authentication (username/password) for IMAP/SMTP in April 2026.**
OAuth2 is now **mandatory** for accessing Outlook/Microsoft 365 email accounts.

Benefits:
- ✅ No passwords stored in the app
- ✅ Granular permission scopes
- ✅ Token expiration and refresh
- ✅ User can revoke access anytime
- ✅ Complies with modern security standards

## References

- [RFC 6749 - OAuth 2.0](https://tools.ietf.org/html/rfc6749)
- [RFC 7636 - PKCE](https://tools.ietf.org/html/rfc7636)
- [RFC 7628 - SASL OAUTHBEARER](https://tools.ietf.org/html/rfc7628)
- [RFC 8628 - Device Authorization Grant](https://tools.ietf.org/html/rfc8628)
- [Microsoft - Authenticate IMAP with OAuth](https://learn.microsoft.com/en-us/exchange/client-developer/legacy-protocols/how-to-authenticate-an-imap-pop-smtp-application-by-using-oauth)
- [Google - OAuth2 for IMAP](https://developers.google.com/gmail/imap/xoauth2-protocol)
