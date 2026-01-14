//! # mailledger-imap
//!
//! A production-quality IMAP client library implementing RFC 9051 (`IMAP4rev2`)
//! with fallback support for RFC 3501 (`IMAP4rev1`).
//!
//! ## Features
//!
//! - **Type-state connection management**: Compile-time enforcement of valid
//!   IMAP state transitions (`NotAuthenticated` → `Authenticated` → `Selected`)
//! - **Full protocol support**: LOGIN, SELECT, FETCH, STORE, COPY, MOVE,
//!   SEARCH, APPEND, EXPUNGE, and more
//! - **IDLE support**: Real-time push notifications via RFC 2177
//! - **TLS via rustls**: Secure connections without OpenSSL dependency
//! - **Server quirks handling**: Built-in workarounds for Gmail, Outlook,
//!   Dovecot, and other common servers
//! - **Sans-I/O parser**: Protocol parsing separated from network I/O
//!
//! ## Quick Start
//!
//! ```ignore
//! use mailledger_imap::{Client, Config, Security, FetchItems};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> mailledger_imap::Result<()> {
//!     // Connect with TLS
//!     let config = Config::new("imap.example.com", Security::Implicit);
//!     let stream = mailledger_imap::connection::connect_tls(&config).await?;
//!     let client = Client::from_stream(stream).await?;
//!
//!     // Authenticate
//!     let mut client = client.login("user@example.com", "password").await?;
//!
//!     // List folders
//!     let folders = client.list("", "*").await?;
//!     for folder in &folders {
//!         println!("Folder: {}", folder.mailbox.as_str());
//!     }
//!
//!     // Select INBOX
//!     let (mut client, status) = client.select("INBOX").await?;
//!     println!("Messages: {}", status.exists);
//!
//!     // Fetch message headers
//!     let messages = client.fetch(
//!         &mailledger_imap::SequenceSet::range(1, 10).unwrap(),
//!         FetchItems::Fast,
//!     ).await?;
//!
//!     // IDLE for real-time updates
//!     if client.supports_idle() {
//!         let mut handle = client.idle().await?;
//!         match handle.wait(Duration::from_secs(30)).await? {
//!             mailledger_imap::IdleEvent::Exists(n) => println!("New count: {n}"),
//!             mailledger_imap::IdleEvent::Timeout => println!("No updates"),
//!             _ => {}
//!         }
//!         handle.done().await?;
//!     }
//!
//!     client.logout().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Connection States
//!
//! The library uses the type-state pattern to enforce valid IMAP operations
//! at compile time:
//!
//! ```text
//! ┌─────────────────────┐
//! │   NotAuthenticated  │ ─── login() ───→ Authenticated
//! └─────────────────────┘
//!            │
//!            ▼
//! ┌─────────────────────┐
//! │    Authenticated    │ ─── select()/examine() ───→ Selected
//! └─────────────────────┘
//!            │
//!            ▼
//! ┌─────────────────────┐
//! │      Selected       │ ─── close() ───→ Authenticated
//! └─────────────────────┘
//! ```
//!
//! ## Modules
//!
//! - [`command`]: IMAP command builders and types
//! - [`connection`]: Connection management and type-state client
//! - [`parser`]: Sans-I/O response parser
//! - [`quirks`]: Server-specific workarounds
//! - [`types`]: Core IMAP types (flags, mailboxes, sequences, etc.)

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

pub mod command;
pub mod connection;
mod error;
pub mod parser;
pub mod quirks;
pub mod types;

pub use command::{Command, FetchAttribute, FetchItems, SearchCriteria, StoreAction, TagGenerator};
pub use connection::{
    Authenticated, Client, Config, ConfigBuilder, FramedStream, IdleEvent, IdleHandle, ImapStream,
    NotAuthenticated, ResponseAccumulator, Security, Selected,
};
pub use error::{Error, Result};
pub use parser::{Response, ResponseParser, UntaggedResponse};
pub use quirks::{ServerQuirks, ServerType};
pub use types::{
    Capability, Flag, Flags, ListResponse, Mailbox, MailboxAttribute, MailboxStatus, ResponseCode,
    SeqNum, SequenceSet, Status, Tag, Uid, UidSet, UidValidity,
};

/// IMAP protocol version supported.
pub const IMAP_VERSION: &str = "IMAP4rev2";
