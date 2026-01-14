# OAuth2 for Email Clients - Quick Summary

## How It Works for Users

**Other email clients (Thunderbird, Mailspring, etc.):**

1. User opens email client
2. User enters: `theirname@outlook.com`
3. Browser opens → User signs in → Approves access
4. Done! Email works.

**MailLedger will work the same way:**

1. User opens MailLedger
2. User enters: `theirname@outlook.com`
3. Browser opens → User signs in → Approves access
4. Done! Email works.

## How It Works Behind the Scenes

### The Shared Client ID Model

```
┌─────────────────────────────────────────────────────────┐
│  MailLedger Project (ONE TIME)                          │
│  ─────────────────────────────────────────────────────  │
│  1. Registers ONE Azure AD app                          │
│  2. Gets Client ID: "12345678-90ab-cdef-1234..."        │
│  3. Puts Client ID in MailLedger code                   │
│  4. Ships to users                                      │
└─────────────────────────────────────────────────────────┘
                         │
                         │ Built into app
                         ▼
┌─────────────────────────────────────────────────────────┐
│  All MailLedger Users                                   │
│  ─────────────────────────────────────────────────────  │
│  Alice adds her Outlook: alice@outlook.com              │
│  Bob adds his Outlook: bob@outlook.com                  │
│  Carol adds her Outlook: carol@outlook.com              │
│                                                          │
│  ALL use the same Client ID (from MailLedger)           │
│  EACH gets their own private token                      │
└─────────────────────────────────────────────────────────┘
```

### Why This Is Secure

- ✅ **Client ID is public** - Safe to share (by design)
- ✅ **Each user authorizes** - Must sign in with their Microsoft account
- ✅ **Each user's token is private** - Stored in their system keyring
- ✅ **Tokens can't be shared** - Each token only works for one user
- ✅ **Users can revoke** - Anytime in Microsoft account settings

## Comparison to Current Setup

### Current (Example Code)
```bash
# Every developer must register their own Azure app
export OAUTH_CLIENT_ID="my-personal-client-id"
cargo run --example outlook_device_flow
```
❌ Users would need to do this too (not acceptable)

### Production (After MailLedger registers app)
```rust
// In code
const MAILLEDGER_CLIENT_ID: &str = "12345678-90ab-cdef-1234-567890abcdef";

// Users just run the app
cargo run --package mailledger-core --example outlook_device_flow
```
✅ Users never see Azure Portal
✅ Just enter email and authorize

## What Needs to Happen

### For MailLedger Project (ONE TIME):

1. **Create a Microsoft account** for MailLedger project
2. **Register Azure AD app**:
   - Name: "MailLedger"
   - Type: Public client (for desktop apps)
   - Permissions: IMAP, SMTP, offline_access
3. **Get Client ID** from Azure Portal
4. **Update code**: Replace `MAILLEDGER_CLIENT_ID` constant
5. **Ship to users**

### For Users (EVERY TIME):
1. Install MailLedger
2. Add account → Enter email
3. Browser opens → Sign in → Approve
4. Done!

## What Happens If...

### "What if MailLedger's Client ID gets blocked?"
- Very unlikely (Microsoft wants legitimate email clients)
- If it happens: Register a new one, update in next release
- Users re-authorize with new release

### "What if a user loses their token?"
- Stored in system keyring (survives restarts)
- If keyring is cleared: User re-authorizes (takes 30 seconds)
- No data loss, just need to sign in again

### "What if someone steals the Client ID?"
- It's public anyway (not a secret)
- They still can't access anyone's email
- Each user must authorize with their own Microsoft account
- Attacker would need the user's Microsoft password (which they don't have)

### "What if rate limits are hit?"
- Microsoft has generous limits for email clients
- Only matters if MailLedger has millions of users
- Can request higher limits from Microsoft
- Can implement caching to reduce API calls

## Examples from Other Clients

### Thunderbird
- **Client ID**: Public (visible in source code)
- **Users**: Millions
- **Setup**: Just enter email
- **Works**: Yes, for 10+ years

### Mailspring
- **Client ID**: Public (visible in source code)
- **Users**: Thousands
- **Setup**: Just enter email
- **Works**: Yes

### MailLedger
- **Client ID**: Will be public (in source code)
- **Users**: Growing
- **Setup**: Just enter email
- **Works**: Yes ✅

## Bottom Line

**Current state:**
- OAuth2 fully implemented ✅
- Device Flow working ✅
- Token storage working ✅
- IMAP authentication working ✅

**What's missing:**
- MailLedger project needs to register ONE Azure AD app
- Update `MAILLEDGER_CLIENT_ID` in code
- Then users can use it without Azure Portal!

**For now (testing):**
- Developers use their own test apps via env var
- This keeps development separate from production

See `OAUTH2_DEPLOYMENT.md` for full technical details.
