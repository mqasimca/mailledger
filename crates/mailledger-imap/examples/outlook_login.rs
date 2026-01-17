#![allow(clippy::expect_used, clippy::doc_markdown, clippy::uninlined_format_args)]
//! Example: Connect to Outlook IMAP with username and password
//!
//! This example demonstrates basic LOGIN authentication with Microsoft Outlook.
//!
//! ## Prerequisites
//!
//! 1. Go to https://account.microsoft.com/security
//! 2. Enable "Two-step verification" (if not already enabled)
//! 3. Go to "Advanced security options" → "App passwords"
//! 4. Generate a new app password for "Mail"
//! 5. Use this app password (not your regular password)
//!
//! ## Running
//!
//! ```bash
//! cargo run --package mailledger-imap --example outlook_login
//! ```

use mailledger_imap::Client;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("MailLedger - Outlook IMAP Login Test");
    println!("=====================================\n");

    // Get credentials
    print!("Email address: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;
    let email = email.trim();

    print!("App password: ");
    io::stdout().flush()?;
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    println!("\nConnecting to outlook.office365.com:993...");

    // Connect with TLS
    let stream = mailledger_imap::connection::connect_tls("outlook.office365.com", 993).await?;
    println!("✓ Connected");

    // Create client and authenticate
    println!("Authenticating as {}...", email);
    let client = Client::from_stream(stream).await?;
    let mut client = client.login(email, password).await?;
    println!("✓ Authenticated successfully!\n");

    // List folders
    println!("Listing folders:");
    let folders = client.list("", "*").await?;
    for folder in folders {
        println!("  - {}", folder.mailbox);
    }

    // Logout
    println!("\nDisconnecting...");
    client.logout().await?;
    println!("✓ Disconnected");

    Ok(())
}
