//! # mailledger-smtp
//!
//! A production-quality SMTP client library implementing RFC 5321.
//!
//! ## Features
//!
//! - **Type-state connection management**: Compile-time enforcement of valid
//!   SMTP state transitions
//! - **Full protocol support**: EHLO, MAIL FROM, RCPT TO, DATA, AUTH, STARTTLS
//! - **TLS support**: Both implicit TLS (port 465) and STARTTLS
//! - **Authentication**: PLAIN, LOGIN, XOAUTH2, OAUTHBEARER
//! - **Extensions**: 8BITMIME, SIZE, PIPELINING, SMTPUTF8
//!
//! ## Quick Start
//!
//! ```ignore
//! use mailledger_smtp::{Client, Address};
//! use mailledger_smtp::connection::connect;
//!
//! #[tokio::main]
//! async fn main() -> mailledger_smtp::Result<()> {
//!     // Connect to SMTP server
//!     let stream = connect("smtp.example.com", 587).await?;
//!     let mut client = Client::from_stream(stream).await?;
//!
//!     // Send EHLO
//!     let client = client.ehlo("client.example.com").await?;
//!
//!     // Upgrade to TLS
//!     let client = client.starttls("smtp.example.com").await?;
//!
//!     // Authenticate
//!     let client = client.auth_plain("user@example.com", "password").await?;
//!
//!     // Send email
//!     let from = Address::new("sender@example.com")?;
//!     let to = Address::new("recipient@example.com")?;
//!
//!     let client = client.mail_from(from).await?;
//!     let client = client.rcpt_to(to).await?;
//!     let client = client.data().await?;
//!
//!     let message = b"Subject: Test\r\n\r\nHello, World!\r\n";
//!     let client = client.send_message(message).await?;
//!
//!     client.quit().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Connection States
//!
//! The library uses the type-state pattern to enforce valid SMTP operations:
//!
//! ```text
//! ┌──────────────┐
//! │  Connected   │ ─── auth_plain() ───→ Authenticated
//! └──────────────┘
//!        │
//!        └─── mail_from() ───→ MailTransaction ───→ RecipientAdded ───→ Data
//! ```
//!
//! ## Modules
//!
//! - [`command`]: SMTP command builders
//! - [`connection`]: Connection management and type-state client
//! - [`parser`]: Response parser
//! - [`types`]: Core SMTP types (addresses, extensions, replies)

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

pub mod command;
pub mod connection;
mod error;
pub mod parser;
pub mod types;

pub use connection::{
    Authenticated, Client, Connected, Data, MailTransaction, RecipientAdded, ServerInfo,
    SmtpConnection,
};
pub use error::{Error, Result};
pub use types::{Address, AuthMechanism, Extension, Mailbox, Reply, ReplyCode};

/// SMTP protocol version supported.
pub const SMTP_VERSION: &str = "SMTP/ESMTP (RFC 5321)";
