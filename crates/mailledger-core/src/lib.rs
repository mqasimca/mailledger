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
//! - **Email Triage System** - HEY-inspired sender screening and inbox organization
//! - **Contact Management** - Address book and autocomplete
//! - **Snooze/Reminders** - Snooze messages to reappear later
//! - **Offline Cache** - Message caching for offline viewing

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

pub mod account;
pub mod cache;
pub mod contacts;
mod error;
pub mod service;
pub mod snooze;
pub mod triage;

pub use account::credentials;
pub use account::{Account, AccountId, AccountRepository, ImapConfig, Security, SmtpConfig};
pub use account::{
    CredentialError, CredentialResult, ValidationError, ValidationResult, validate_account,
};
pub use cache::{CacheRepository, CachedMessageContent, CachedMessageSummary};
pub use contacts::{Contact, ContactRepository};
pub use error::{Error, Result};
pub use service::{
    Attachment, AuthClient, Folder, FolderType, IdleEvent, MailServiceError, MessageContent,
    MessageSummary, OutgoingMessage, SearchCriteria, SelectedClient, SmtpError, archive_message,
    connect_and_login, download_attachment, fetch_message_content, fetch_messages, idle_monitor,
    list_folders, mark_read, mark_unread, search_messages, select_folder, send_email, toggle_flag,
};
pub use snooze::{SnoozeDuration, SnoozeRepository, SnoozedMessage};
pub use triage::{InboxCategory, ScreenedSender, SenderDecision, TriageRepository};
