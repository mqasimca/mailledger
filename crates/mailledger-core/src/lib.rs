//! # mailledger-core
//!
//! Core business logic for `MailLedger` email client.
//!
//! This crate provides:
//! - Account management
//! - Message synchronization
//! - Local storage (`SQLite`)
//! - Domain models
//! - Email services

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

pub mod account;
mod error;
pub mod service;

pub use account::credentials;
pub use account::{Account, AccountId, AccountRepository, ImapConfig, Security, SmtpConfig};
pub use account::{
    CredentialError, CredentialResult, ValidationError, ValidationResult, validate_account,
};
pub use error::{Error, Result};
pub use service::{
    Attachment, AuthClient, Folder, FolderType, IdleEvent, MailServiceError, MessageContent,
    MessageSummary, OutgoingMessage, SelectedClient, SmtpError, connect_and_login,
    fetch_message_content, fetch_messages, idle_monitor, list_folders, mark_read, mark_unread,
    select_folder, send_email, toggle_flag,
};
