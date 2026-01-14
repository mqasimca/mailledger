//! Example: OAuth2 authentication with Microsoft Outlook IMAP
//!
//! This example demonstrates how to:
//! 1. Configure OAuth2 for Microsoft/Outlook
//! 2. Perform the authorization flow with PKCE
//! 3. Exchange the authorization code for an access token
//! 4. Use the token with IMAP XOAUTH2 authentication
//!
//! ## Prerequisites
//!
//! 1. Register an application in Azure AD:
//!    - Go to https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade
//!    - Create a new app registration
//!    - Set redirect URI to `http://localhost:8080` (or your chosen port)
//!    - Note your Application (client) ID
//!
//! 2. Set environment variables:
//!    ```bash
//!    export OAUTH_CLIENT_ID="your-client-id-here"
//!    export OAUTH_EMAIL="your-email@outlook.com"
//!    ```
//!
//! ## Running
//!
//! ```bash
//! cargo run --example outlook_oauth2
//! ```

use mailledger_oauth::{AuthorizationCodeFlow, OAuthClient, Provider};
use std::env;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration from environment
    let client_id =
        env::var("OAUTH_CLIENT_ID").expect("OAUTH_CLIENT_ID environment variable not set");
    let email = env::var("OAUTH_EMAIL").expect("OAUTH_EMAIL environment variable not set");

    println!("MailLedger OAuth2 Example - Microsoft Outlook");
    println!("============================================\n");

    // Step 1: Configure Microsoft OAuth2 provider
    println!("Step 1: Configuring Microsoft OAuth2 provider...");
    let provider = Provider::microsoft()?;
    println!("  Provider: {}", provider.name);
    println!("  Auth URL: {}", provider.auth_url);
    println!("  Scopes: {:?}\n", provider.default_scopes);

    // Step 2: Create OAuth client with PKCE
    println!("Step 2: Creating OAuth2 client with PKCE...");
    let client = OAuthClient::new(&client_id, provider).with_redirect_uri("http://localhost:8080");

    let flow = AuthorizationCodeFlow::new(client).with_pkce();
    println!("  PKCE enabled for enhanced security\n");

    // Step 3: Generate authorization URL
    println!("Step 3: Generating authorization URL...");
    let state = format!("random-state-{}", chrono::Utc::now().timestamp());
    let auth_url = flow.authorization_url(None, Some(&state))?;

    println!("\n┌─────────────────────────────────────────────────────────────┐");
    println!("│  PLEASE VISIT THIS URL TO AUTHORIZE THE APPLICATION:       │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!("\n{}\n", auth_url);

    // In a real application, you would:
    // 1. Open the URL in a browser
    // 2. Spin up a local HTTP server on port 8080 to capture the redirect
    // 3. Extract the authorization code from the redirect URL

    println!("After authorizing, you'll be redirected to:");
    println!(
        "  http://localhost:8080/?code=AUTHORIZATION_CODE&state={}\n",
        state
    );

    // Step 4: Get authorization code from user
    print!("Enter the authorization code from the redirect URL: ");
    io::stdout().flush()?;

    let mut code = String::new();
    io::stdin().read_line(&mut code)?;
    let code = code.trim();

    if code.is_empty() {
        println!("\nNo code entered. Exiting.");
        return Ok(());
    }

    // Step 5: Exchange code for token
    println!("\nStep 5: Exchanging authorization code for access token...");
    let token = flow.exchange_code(code, None).await?;

    println!("✓ Token obtained successfully!");
    println!("  Access token: {}...", &token.access_token[..20]);
    println!("  Token type: {}", token.token_type);
    println!("  Expires at: {:?}", token.expires_at);
    println!("  Has refresh token: {}", token.refresh_token.is_some());
    println!("  Scope: {:?}\n", token.scope);

    // Step 6: Generate SASL XOAUTH2 string
    println!("Step 6: Generating SASL XOAUTH2 authentication string...");
    let auth_string = mailledger_oauth::sasl::xoauth2_response(&email, &token.access_token);
    println!("  Auth string (base64): {}...\n", &auth_string[..50]);

    // Step 7: Show how to use with IMAP
    println!("Step 7: Using with IMAP (pseudo-code):");
    println!("  ```rust");
    println!("  use mailledger_imap::{{Client, Config, Security}};");
    println!();
    println!("  let config = Config::new(\"outlook.office365.com\", Security::Tls);");
    println!("  let stream = mailledger_imap::connection::connect_tls(&config).await?;");
    println!("  let client = Client::from_stream(stream).await?;");
    println!();
    println!("  // Authenticate with OAuth2");
    println!(
        "  let client = client.authenticate_xoauth2(\"{}\", &token).await?;",
        email
    );
    println!("  ```\n");

    // Step 8: Token refresh
    if token.refresh_token.is_some() {
        println!("Step 8: Token can be refreshed when expired:");
        println!("  ```rust");
        println!("  if token.is_expired() {{");
        println!("      let new_token = client.refresh_token(&token).await?;");
        println!("      // Store new_token securely");
        println!("  }}");
        println!("  ```\n");
    }

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│  SUCCESS! You can now use this token with IMAP/SMTP        │");
    println!("└─────────────────────────────────────────────────────────────┘");
    println!("\nToken should be stored securely (e.g., in keyring) for future use.");

    Ok(())
}
