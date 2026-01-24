//! Cache data models.

use chrono::{DateTime, Utc};

use crate::AccountId;

/// A cached message summary for offline display.
#[derive(Debug, Clone)]
pub struct CachedMessageSummary {
    /// Account ID this message belongs to.
    pub account_id: AccountId,
    /// Folder path where the message is stored.
    pub folder_path: String,
    /// Message UID.
    pub uid: u32,
    /// Sender name.
    pub from_name: String,
    /// Sender email.
    pub from_email: String,
    /// Message subject.
    pub subject: String,
    /// Message snippet (preview text).
    pub snippet: String,
    /// Message date.
    pub date: String,
    /// Whether the message has been read.
    pub is_read: bool,
    /// Whether the message is flagged/starred.
    pub is_flagged: bool,
    /// Whether the message has attachments.
    pub has_attachments: bool,
    /// When the message was cached.
    pub cached_at: DateTime<Utc>,
}

/// Cached message content for offline viewing.
#[derive(Debug, Clone)]
pub struct CachedMessageContent {
    /// Account ID this message belongs to.
    pub account_id: AccountId,
    /// Folder path where the message is stored.
    pub folder_path: String,
    /// Message UID.
    pub uid: u32,
    /// Full sender information.
    pub from: String,
    /// Recipients (To).
    pub to: String,
    /// CC recipients.
    pub cc: String,
    /// Message subject.
    pub subject: String,
    /// Message date.
    pub date: String,
    /// Plain text body.
    pub body_text: Option<String>,
    /// HTML body.
    pub body_html: Option<String>,
    /// Serialized attachments info (JSON).
    pub attachments_json: Option<String>,
    /// When the content was cached.
    pub cached_at: DateTime<Utc>,
}
