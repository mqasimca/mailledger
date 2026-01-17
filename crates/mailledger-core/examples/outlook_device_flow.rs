#![allow(
    clippy::expect_used,
    clippy::doc_markdown,
    clippy::uninlined_format_args,
    clippy::used_underscore_items,
    clippy::too_many_lines
)]
//! Example: OAuth2 Device Flow authentication with Outlook
//!
//! This example demonstrates the easiest way to authenticate with Outlook:
//! 1. Get a code from the app
//! 2. Visit a URL and enter the code
//! 3. Token saved automatically - never expires (auto-refreshes)
//!
//! ## One-Time Setup (2 minutes)
//!
//! 1. Go to https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade
//! 2. Click "New registration"
//!    - Name: MailLedger (or anything)
//!    - Supported accounts: "Personal Microsoft accounts only"
//!    - Redirect URI: Leave empty (not needed for device flow)
//! 3. Click "Register"
//! 4. Copy the "Application (client) ID"
//!
//! ## Running
//!
//! ```bash
//! export OAUTH_CLIENT_ID="your-client-id-from-azure"
//! export OAUTH_EMAIL="your-email@outlook.com"
//! cargo run --package mailledger-core --example outlook_device_flow
//! ```

use mailledger_core::account::{AccountId, credentials};
use mailledger_imap::Client;
use mailledger_oauth::{DeviceFlow, OAuthClient, Provider};
use std::env;

// In production, MailLedger would register ONE Azure AD app and use this Client ID
// for all users. This is how Thunderbird, Mailspring, and other email clients work.
const MAILLEDGER_CLIENT_ID: &str = "YOUR-MAILLEDGER-CLIENT-ID-HERE";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get configuration
    // In production: Just prompt for email, use MAILLEDGER_CLIENT_ID
    // In development: Use env var to test with your own Azure app
    let client_id = env::var("OAUTH_CLIENT_ID").unwrap_or_else(|_| {
        if MAILLEDGER_CLIENT_ID == "YOUR-MAILLEDGER-CLIENT-ID-HERE" {
            eprintln!("ERROR: No OAuth2 Client ID configured!");
            eprintln!();
            eprintln!("For development/testing:");
            eprintln!("  1. Register an Azure AD app at:");
            eprintln!("     https://portal.azure.com/#blade/Microsoft_AAD_RegisteredApps/ApplicationsListBlade");
            eprintln!("  2. Set environment variables:");
            eprintln!("     export OAUTH_CLIENT_ID=\"your-client-id\"");
            eprintln!("     export OAUTH_EMAIL=\"your@outlook.com\"");
            eprintln!();
            eprintln!("For production deployment:");
            eprintln!("  MailLedger project should register ONE Azure AD app");
            eprintln!("  and replace MAILLEDGER_CLIENT_ID constant in this code.");
            std::process::exit(1);
        }
        MAILLEDGER_CLIENT_ID.to_string()
    });

    let email = env::var("OAUTH_EMAIL").expect("OAUTH_EMAIL environment variable not set");

    println!("MailLedger - Outlook Device Flow Authentication");
    println!("================================================\n");

    // Use a consistent account ID based on email for token storage
    let account_id = AccountId(
        email
            .chars()
            .fold(0i64, |acc, c| acc.wrapping_add(c as i64)),
    );

    // Check if we already have a token
    println!("Checking for stored token...");
    let mut token = credentials::get_oauth_token(account_id)?;

    if let Some(ref existing_token) = token {
        println!("✓ Found stored token");

        // Check if token needs refresh
        if existing_token.is_expired() {
            println!("Token expired, refreshing...");
            let provider = Provider::microsoft()?;
            let oauth_client = OAuthClient::new(&client_id, provider);
            let new_token = oauth_client.refresh_token(existing_token).await?;
            credentials::store_oauth_token(account_id, &new_token)?;
            token = Some(new_token);
            println!("✓ Token refreshed and saved");
        } else {
            println!("✓ Token is still valid");
        }
    } else {
        println!("No token found, starting OAuth2 Device Flow...\n");

        // Configure OAuth2 with Microsoft provider
        let provider = Provider::microsoft()?;
        let oauth_client = OAuthClient::new(&client_id, provider);
        let flow = DeviceFlow::new(oauth_client);

        // Request device authorization
        let auth = flow.request_device_authorization(None).await?;

        // Display instructions to user
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│  PLEASE COMPLETE THESE STEPS:                               │");
        println!("└─────────────────────────────────────────────────────────────┘");
        println!();
        println!("1. Visit: {}", auth.verification_uri);
        println!("2. Enter code: {}", auth.user_code);
        println!();
        println!("Waiting for you to complete authorization...");
        println!("(Code expires in {} seconds)\n", auth.expires_in);

        // Poll for token (30 attempts = ~2.5 minutes)
        let interval = std::time::Duration::from_secs(u64::from(auth.interval));
        let max_attempts = 30;

        let mut attempts = 0;
        loop {
            if attempts >= max_attempts {
                println!("✗ Timeout - please try again");
                return Err("Authorization timeout".into());
            }

            match flow.poll_for_token(&auth.device_code, interval).await {
                Ok(new_token) => {
                    println!("✓ Authorization successful!");
                    credentials::store_oauth_token(account_id, &new_token)?;
                    println!("✓ Token saved to system keyring");
                    token = Some(new_token);
                    break;
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("authorization_pending") {
                        print!(".");
                        std::io::Write::flush(&mut std::io::stdout())?;
                        attempts += 1;
                    } else if err_str.contains("slow_down") {
                        println!("\n(Slowing down polling...)");
                        attempts += 1;
                    } else {
                        return Err(e.into());
                    }
                }
            }
        }
        println!();
    }

    let token = token.expect("Token should be available at this point");

    // Connect to Outlook IMAP
    println!("Connecting to Outlook IMAP...");
    let stream = mailledger_imap::connection::connect_tls("outlook.office365.com", 993).await?;
    println!("✓ Connected to outlook.office365.com:993");

    // Authenticate with OAuth2
    println!("Authenticating with OAuth2...");
    let client = Client::from_stream(stream).await?;
    let mut client = client.authenticate_xoauth2(&email, &token).await?;
    println!("✓ Authenticated as {}\n", email);

    // List folders
    println!("Your folders:");
    println!("─────────────────────────────────────");
    let folders = client.list("", "*").await?;
    for folder in &folders {
        println!("  📁 {}", folder.mailbox);
    }
    println!("─────────────────────────────────────");
    println!("Total: {} folders\n", folders.len());

    // Logout
    println!("Disconnecting...");
    client.logout().await?;
    println!("✓ Done!\n");

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│  SUCCESS! Your token is saved and will auto-refresh.       │");
    println!("│  Next time you run this, it will connect immediately!      │");
    println!("└─────────────────────────────────────────────────────────────┘");

    Ok(())
}
