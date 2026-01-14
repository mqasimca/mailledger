# OAuth2 Deployment Guide

This document explains how OAuth2 authentication works for end users vs developers.

## For End Users (Production)

**End users should NOT need to register Azure AD apps!**

When MailLedger is released, users will:

1. Open MailLedger
2. Click "Add Account"
3. Enter their email address
4. Complete OAuth2 authorization in browser
5. Done!

Behind the scenes, MailLedger uses a **shared Client ID** registered by the MailLedger project.

## How This Works

### Shared Client ID Model

This is the standard approach used by legitimate email clients:

| Email Client | Approach |
|--------------|----------|
| **Mozilla Thunderbird** | One Azure AD app for all Thunderbird users |
| **Mailspring** | One Azure AD app for all Mailspring users |
| **Apple Mail** | One Azure AD app for all Apple Mail users |
| **MailLedger** | One Azure AD app for all MailLedger users |

### Why This Is Safe

- **Client ID is public** - It's okay for everyone to know it
- **No client secret** - "Public clients" don't use secrets
- **User authorization required** - Each user must approve access
- **Tokens are private** - Each user has their own token stored securely

## For MailLedger Project Maintainers

### One-Time Production Setup

The MailLedger project needs to register ONE Azure AD application:

1. **Register the App**
   - Go to https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade
   - Sign in with a Microsoft account (create a dedicated one for MailLedger)
   - Click "New registration"
   - **Name**: `MailLedger`
   - **Supported account types**: "Accounts in any organizational directory and personal Microsoft accounts (Multitenant)"
   - **Redirect URI**: Leave empty (Device Flow doesn't need it)
   - Click "Register"

2. **Configure the App**
   - Note the "Application (client) ID" - this is your shared Client ID
   - Go to "Authentication"
   - Under "Advanced settings" → "Allow public client flows" → Set to **Yes**
   - This enables Device Flow for desktop apps

3. **Set Permissions** (Optional but recommended)
   - Go to "API permissions"
   - Add:
     - `IMAP.AccessAsUser.All` - For IMAP access
     - `SMTP.Send` - For SMTP access
     - `offline_access` - For refresh tokens
   - **No admin consent needed** for personal accounts

4. **Update the Code**
   - Replace `MAILLEDGER_CLIENT_ID` constant in:
     - `crates/mailledger-core/examples/outlook_device_flow.rs`
     - Any production OAuth2 configuration files
   - Commit to repository (Client ID is public, this is safe)

5. **Done!**
   - All users will now use this shared Client ID
   - Each user authorizes individually
   - Tokens are stored per-user

### Example

```rust
// Before (development)
const MAILLEDGER_CLIENT_ID: &str = "YOUR-MAILLEDGER-CLIENT-ID-HERE";

// After (production - example Client ID, use your real one)
const MAILLEDGER_CLIENT_ID: &str = "12345678-90ab-cdef-1234-567890abcdef";
```

## For Developers (Testing)

Developers testing OAuth2 changes should use their own Azure AD app:

1. **Register Your Own Test App** (same steps as production)
2. **Use Environment Variable**
   ```bash
   export OAUTH_CLIENT_ID="your-test-client-id"
   export OAUTH_EMAIL="your-test@outlook.com"
   cargo run --package mailledger-core --example outlook_device_flow
   ```
3. **Never commit your test Client ID** - keep using the placeholder

This way:
- Production uses the shared MailLedger Client ID
- Developers can test with their own apps
- No conflicts between environments

## Security Considerations

### Is it safe to publish the Client ID?

**Yes!** Here's why:

1. **Client ID is public by design**
   - OAuth2 spec allows this for "public clients"
   - Desktop apps, mobile apps, and CLI tools use this model
   - The ID just identifies which app is making requests

2. **Authorization still required**
   - Each user must sign in and approve access
   - Microsoft verifies the user's identity
   - User can revoke access anytime

3. **Tokens are protected**
   - Tokens are stored per-user in system keyring
   - Tokens are encrypted at rest
   - Tokens are never logged or displayed
   - Tokens can't be used by other users

4. **No client secret**
   - Public clients don't use secrets
   - Nothing sensitive to protect
   - Even if someone knows the Client ID, they can't do anything without user authorization

### What users can revoke

Users can revoke MailLedger's access anytime:

1. Go to https://account.microsoft.com/privacy/
2. Click "Apps and services"
3. Find "MailLedger"
4. Click "Remove"

This immediately invalidates their token.

## Rate Limits

Microsoft applies rate limits per Client ID:

- **Current limits** (as of 2026): Generous for email clients
- **If exceeded**: Users see "too many requests" error
- **Mitigation**: Implement exponential backoff and caching

If MailLedger becomes very popular, consider:
- Applying for higher rate limits from Microsoft
- Implementing smart caching to reduce API calls
- Showing clear error messages to users

## Alternative: Let Users Register

Some advanced users might want their own Client ID:

```rust
// Allow override via env var for advanced users
let client_id = env::var("OAUTH_CLIENT_ID")
    .unwrap_or_else(|_| MAILLEDGER_CLIENT_ID.to_string());
```

This could be exposed in advanced settings:
- 99% of users: Use shared MailLedger Client ID
- 1% of users: Use their own (enterprise users, privacy-conscious users)

## Google & Yahoo

The same approach works for Google and Yahoo:

### Google
- Register at https://console.cloud.google.com/
- OAuth consent screen type: "External"
- Publish the app (go through Google verification)
- All users share one Client ID

### Yahoo
- Register at https://developer.yahoo.com/apps/
- Similar process
- Yahoo is less commonly used for new accounts

## Comparison to App Passwords

| Aspect | OAuth2 (Shared Client ID) | App Passwords |
|--------|--------------------------|---------------|
| User setup | Enter email, authorize | Enable 2FA, generate password |
| MailLedger setup | One-time Azure registration | None |
| Security | ✅ Most secure (OAuth2) | ⚠️ Password-like token |
| User experience | ✅ Simple (just email) | ⚠️ Complex (find settings) |
| Future-proof | ✅ Microsoft's standard | ❌ Being deprecated |
| Works after 2026 | ✅ Yes | ❌ No (Basic Auth disabled) |

## Summary

**For Users:**
- Just enter your email and authorize
- MailLedger handles everything else
- No Azure Portal needed

**For MailLedger Project:**
- Register ONE Azure AD app
- Put Client ID in code
- Ship to all users
- Update periodically if Microsoft requires it

**For Developers:**
- Use your own test app via env var
- Don't commit your test Client ID
- Production uses shared ID

This is the standard, secure, and user-friendly approach used by all major email clients.
