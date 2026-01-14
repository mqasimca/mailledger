# Outlook Setup Guide - Device Flow (2 Minutes)

This guide shows how to set up OAuth2 authentication for your personal Outlook account using the Device Flow method. **No complex setup required!**

## Why Device Flow?

- ✅ No app passwords needed
- ✅ No redirect URIs or local servers
- ✅ Token never expires (auto-refreshes)
- ✅ Most secure method
- ✅ Works great for CLI apps

## Step 1: Register Your App (One-Time, 2 minutes)

1. **Go to Azure Portal**
   Visit: https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade

2. **Click "New registration"**

3. **Fill in the form:**
   - **Name**: `MailLedger` (or any name you like)
   - **Supported account types**: Select **"Personal Microsoft accounts only"**
   - **Redirect URI**: Leave this **empty** (not needed for device flow)

4. **Click "Register"**

5. **Copy your Client ID**
   You'll see an "Application (client) ID" like: `12345678-1234-1234-1234-123456789abc`

   Copy this - you'll need it to run the app.

## Step 2: Configure API Permissions (Optional but Recommended)

By default, your app can access basic Microsoft Graph APIs. For full IMAP/SMTP access:

1. In your app registration, click **"API permissions"** in the left menu
2. Click **"Add a permission"**
3. Select **"APIs my organization uses"** → Search for **"Office 365 Exchange Online"**
4. Select **"Delegated permissions"**
5. Add:
   - `IMAP.AccessAsUser.All` - For reading/writing email
   - `SMTP.Send` - For sending email
   - `offline_access` - For refresh tokens
6. Click **"Add permissions"**

> **Note:** You don't need admin consent for personal accounts. The user (you) will grant permissions when you first sign in.

## Step 3: Run the Example

```bash
# Set your credentials
export OAUTH_CLIENT_ID="12345678-1234-1234-1234-123456789abc"  # From Step 1
export OAUTH_EMAIL="yourname@outlook.com"

# Run the device flow example
cargo run --package mailledger-core --example outlook_device_flow
```

## Step 4: Authorize the App

The app will show you something like:

```
┌─────────────────────────────────────────────────────────────┐
│  PLEASE COMPLETE THESE STEPS:                               │
└─────────────────────────────────────────────────────────────┘

1. Visit: https://microsoft.com/devicelogin
2. Enter code: ABCD-1234

Waiting for you to complete authorization...
```

**What to do:**
1. Open a browser and go to the URL shown
2. Enter the code shown (e.g., `ABCD-1234`)
3. Sign in with your Outlook account
4. Approve the permissions
5. Return to the terminal - it will automatically continue!

## Step 5: Enjoy!

```
✓ Authorization successful!
✓ Token saved to system keyring
✓ Connected to outlook.office365.com:993
✓ Authenticated as yourname@outlook.com

Your folders:
─────────────────────────────────────────
  📁 INBOX
  📁 Drafts
  📁 Sent Items
  📁 Deleted Items
  📁 Junk Email
─────────────────────────────────────────

SUCCESS! Your token is saved and will auto-refresh.
Next time you run this, it will connect immediately!
```

**That's it!** Your token is saved securely in your system keyring (GNOME Keyring on Linux, Keychain on macOS, Credential Manager on Windows).

## Next Time

The second time you run the example, it will:
1. ✅ Find your stored token
2. ✅ Auto-refresh if expired
3. ✅ Connect immediately - no browser needed!

## Troubleshooting

### "OAUTH_CLIENT_ID environment variable not set"

Make sure you exported the environment variables:
```bash
export OAUTH_CLIENT_ID="your-client-id"
export OAUTH_EMAIL="your-email@outlook.com"
```

### "Provider does not support device flow"

This shouldn't happen with Microsoft provider. If you see this, the provider configuration might be incorrect.

### "Authorization timeout"

The code expires after a few minutes. Just run the example again to get a new code.

### Token errors after first authorization

If you change the requested scopes or permissions, you may need to delete the stored token:
```rust
// In your code
credentials::delete_oauth_token(account_id)?;
```

## Security Notes

- ✅ Your token is stored securely in the system keyring
- ✅ The token is never displayed or logged
- ✅ Tokens auto-refresh so they never expire
- ✅ You can revoke access anytime in your Microsoft account settings
- ✅ No passwords are stored (OAuth2 tokens only)

## Where to Revoke Access

If you want to revoke MailLedger's access:
1. Go to https://account.microsoft.com/privacy/
2. Go to "Apps and services"
3. Find "MailLedger" and click "Remove"

## Comparison to App Passwords

| Feature | Device Flow (OAuth2) | App Passwords |
|---------|---------------------|---------------|
| Setup time | 2 minutes (one-time) | 5 minutes (per account) |
| Security | ✅ Most secure (OAuth2) | ⚠️ Less secure (password-like) |
| Expiration | ✅ Never (auto-refresh) | ⚠️ Can be revoked by Microsoft |
| Multi-account | ✅ Easy (one token per account) | ⚠️ Need password per account |
| Future-proof | ✅ Microsoft's recommended method | ❌ Being phased out |
| 2FA required | ✅ Yes (more secure) | ✅ Yes |

## Questions?

See `docs/OAUTH2_IMPLEMENTATION.md` for full technical details about the OAuth2 implementation.
