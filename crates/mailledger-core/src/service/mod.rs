//! Core services for email operations.
//!
//! This module provides the service layer that bridges the GUI
//! with the underlying IMAP and SMTP libraries.

pub mod mail;
pub mod smtp;

pub use mail::{
    Attachment, AuthClient, Folder, FolderType, IdleEvent, MailServiceError, MessageContent,
    MessageSummary, SelectedClient, connect_and_login, fetch_message_content, fetch_messages,
    idle_monitor, list_folders, mark_read, mark_unread, select_folder, toggle_flag,
};
pub use smtp::{OutgoingMessage, SmtpError, send_email};
