//! Core services for email operations.
//!
//! This module provides the service layer that bridges the GUI
//! with the underlying IMAP and SMTP libraries.

pub mod mail;
pub mod smtp;

pub use mail::{
    Attachment, AuthClient, Folder, FolderType, IdleEvent, MailServiceError, MessageContent,
    MessageSummary, SearchCriteria, SelectedClient, archive_message, connect_and_login,
    download_attachment, fetch_message_content, fetch_messages, idle_monitor, list_folders,
    mark_read, mark_unread, search_messages, select_folder, toggle_flag,
};
pub use smtp::{OutgoingMessage, SmtpError, send_email};
