//! Message data models.

use super::FolderId;
use chrono::{DateTime, Local};

/// Parses a "from" field into name and email parts.
fn parse_from_field(from: &str) -> (String, String) {
    // Find the last '<' that is not inside quotes
    let mut in_quotes = false;
    let mut last_angle = None;
    let mut last_close_angle = None;

    for (i, c) in from.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            '<' if !in_quotes => last_angle = Some(i),
            '>' if !in_quotes => last_close_angle = Some(i),
            _ => {}
        }
    }

    // Try to parse "Name <email@example.com>" format
    if let Some(start) = last_angle
        && let Some(end) = last_close_angle
        && end > start
    {
        let email = from[start + 1..end].to_string();
        let name = from[..start].trim().trim_matches('"').to_string();
        // If name is empty, use email
        if name.is_empty() {
            return (email.clone(), email);
        }
        return (name, email);
    }
    // Just an email address
    (from.to_string(), from.to_string())
}

/// Formats an RFC 2822 date string to local time.
///
/// Converts dates like "Thu, 15 Jan 2026 19:31:43 +0000" to local timezone
/// and formats as "Thu, 15 Jan 2026 14:31:43" (example for EST).
fn format_date_local(rfc2822_date: &str) -> String {
    // Try to parse the RFC 2822 date
    if let Ok(dt) = DateTime::parse_from_rfc2822(rfc2822_date) {
        // Convert to local timezone
        let local: DateTime<Local> = dt.with_timezone(&Local);
        // Format as "Day, DD Mon YYYY HH:MM:SS"
        return local.format("%a, %d %b %Y %H:%M:%S").to_string();
    }

    // If parsing fails, try parsing as RFC 3339 (another common format)
    if let Ok(dt) = DateTime::parse_from_rfc3339(rfc2822_date) {
        let local: DateTime<Local> = dt.with_timezone(&Local);
        return local.format("%a, %d %b %Y %H:%M:%S").to_string();
    }

    // If all parsing fails, return the original string
    rfc2822_date.to_string()
}

/// Unique identifier for a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(pub u32);

/// Summary of a message for display in the message list.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used as features are implemented
pub struct MessageSummary {
    /// Unique identifier.
    pub id: MessageId,
    /// Folder containing this message.
    pub folder_id: FolderId,
    /// Sender display name.
    pub from_name: String,
    /// Sender email address.
    pub from_email: String,
    /// Message subject.
    pub subject: String,
    /// Short preview of message content.
    pub snippet: String,
    /// Message date as display string.
    pub date: String,
    /// Whether the message has been read.
    pub is_read: bool,
    /// Whether the message is flagged/starred.
    pub is_flagged: bool,
    /// Whether the message has attachments.
    pub has_attachments: bool,
    /// Thread ID for grouping related messages.
    pub thread_id: Option<String>,
    /// Message-ID header.
    pub message_id: Option<String>,
    /// In-Reply-To header.
    pub in_reply_to: Option<String>,
}

/// Full message content for display in the message view.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields will be used as features are implemented
pub struct MessageContent {
    /// Message ID.
    pub id: MessageId,
    /// Sender display name.
    pub from_name: String,
    /// Sender email address.
    pub from_email: String,
    /// Recipients (To).
    pub to: Vec<String>,
    /// CC recipients.
    pub cc: Vec<String>,
    /// Message subject.
    pub subject: String,
    /// Full date string.
    pub date: String,
    /// Plain text body.
    pub body_text: Option<String>,
    /// HTML body (sanitized).
    pub body_html: Option<String>,
    /// Attachments.
    pub attachments: Vec<Attachment>,
}

/// An attachment in a message.
#[derive(Debug, Clone)]
#[allow(dead_code)] // Some fields will be used when inline preview is implemented
pub struct Attachment {
    /// Filename.
    pub filename: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size: u64,
    /// Part number for fetching.
    pub part_number: String,
    /// Content-Transfer-Encoding.
    pub encoding: String,
}

impl MessageSummary {
    /// Creates a message summary from core service data.
    #[must_use]
    pub fn from_core(folder_id: FolderId, core_msg: &mailledger_core::MessageSummary) -> Self {
        // Extract name and email from the "from" field
        // Format can be "Name <email>" or just "email"
        let (from_name, from_email) = parse_from_field(&core_msg.from);

        Self {
            id: MessageId(core_msg.uid.get()),
            folder_id,
            from_name,
            from_email,
            subject: core_msg.subject.clone(),
            snippet: core_msg.snippet.clone(),
            date: format_date_local(&core_msg.date),
            is_read: core_msg.is_read,
            is_flagged: core_msg.is_flagged,
            has_attachments: core_msg.has_attachment,
            thread_id: core_msg.thread_id.clone(),
            message_id: core_msg.message_id.clone(),
            in_reply_to: core_msg.in_reply_to.clone(),
        }
    }

    /// Creates mock messages for testing.
    #[must_use]
    #[allow(dead_code)] // Used for testing/demo
    pub fn mock_messages(folder_id: FolderId) -> Vec<Self> {
        vec![
            Self {
                id: MessageId(1),
                folder_id,
                from_name: "John Doe".into(),
                from_email: "john@example.com".into(),
                subject: "Meeting Tomorrow".into(),
                snippet: "Hey, just wanted to confirm our meeting tomorrow at 3pm...".into(),
                date: "Jan 8".into(),
                is_read: false,
                is_flagged: false,
                has_attachments: false,
                thread_id: None,
                message_id: Some("<msg1@example.com>".into()),
                in_reply_to: None,
            },
            Self {
                id: MessageId(2),
                folder_id,
                from_name: "Jane Smith".into(),
                from_email: "jane@example.com".into(),
                subject: "Project Update".into(),
                snippet: "The project is going well. Here's a quick summary of what we've accomplished...".into(),
                date: "Jan 7".into(),
                is_read: true,
                is_flagged: false,
                has_attachments: true,
                thread_id: None,
                message_id: Some("<msg2@example.com>".into()),
                in_reply_to: None,
            },
            Self {
                id: MessageId(3),
                folder_id,
                from_name: "Bob Wilson".into(),
                from_email: "bob@example.com".into(),
                subject: "Invoice #1234".into(),
                snippet: "Please find attached the invoice for last month's services...".into(),
                date: "Jan 6".into(),
                is_read: true,
                is_flagged: true,
                has_attachments: true,
                thread_id: None,
                message_id: Some("<msg3@example.com>".into()),
                in_reply_to: None,
            },
            Self {
                id: MessageId(4),
                folder_id,
                from_name: "Alice Brown".into(),
                from_email: "alice@example.com".into(),
                subject: "Re: Quarterly Report".into(),
                snippet: "Thanks for sending the report. I've reviewed it and have a few comments...".into(),
                date: "Jan 5".into(),
                is_read: true,
                is_flagged: false,
                has_attachments: false,
                thread_id: Some("<quarterly-report@example.com>".into()),
                message_id: Some("<msg4@example.com>".into()),
                in_reply_to: Some("<quarterly-report@example.com>".into()),
            },
            Self {
                id: MessageId(5),
                folder_id,
                from_name: "Tech News".into(),
                from_email: "news@technews.com".into(),
                subject: "Your Weekly Tech Digest".into(),
                snippet: "This week in tech: New developments in AI, open source updates...".into(),
                date: "Jan 4".into(),
                is_read: false,
                is_flagged: false,
                has_attachments: false,
                thread_id: None,
                message_id: Some("<msg5@example.com>".into()),
                in_reply_to: None,
            },
        ]
    }
}

impl MessageContent {
    /// Creates content from core service data.
    #[must_use]
    pub fn from_core(core_content: &mailledger_core::MessageContent) -> Self {
        let (from_name, from_email) = parse_from_field(&core_content.from);

        Self {
            id: MessageId(core_content.uid.get()),
            from_name,
            from_email,
            to: core_content.to.clone(),
            cc: core_content.cc.clone(),
            subject: core_content.subject.clone(),
            date: format_date_local(&core_content.date),
            body_text: core_content.body_text.clone(),
            body_html: core_content.body_html.clone(),
            attachments: core_content
                .attachments
                .iter()
                .map(Attachment::from_core)
                .collect(),
        }
    }

    /// Creates mock content for a message.
    #[must_use]
    #[allow(dead_code)] // Fallback for when fetch fails
    pub fn mock_content(summary: &MessageSummary) -> Self {
        Self {
            id: summary.id,
            from_name: summary.from_name.clone(),
            from_email: summary.from_email.clone(),
            to: vec!["me@example.com".into()],
            cc: vec![],
            subject: summary.subject.clone(),
            date: format!("{}, 2026 at 10:30 AM", summary.date),
            body_text: Some(format!(
                "Hi,\n\n{}\n\nBest regards,\n{}",
                summary.snippet, summary.from_name
            )),
            body_html: None,
            attachments: vec![],
        }
    }
}

impl Attachment {
    /// Creates an attachment from core service data.
    #[must_use]
    pub fn from_core(core_attachment: &mailledger_core::Attachment) -> Self {
        Self {
            filename: core_attachment.filename.clone(),
            mime_type: core_attachment.mime_type.clone(),
            size: core_attachment.size,
            part_number: core_attachment.part_number.clone(),
            encoding: core_attachment.encoding.clone(),
        }
    }

    /// Returns a human-readable size string (e.g., "1.5 MB").
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Acceptable for display purposes
    pub fn size_display(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;

        if self.size >= MB {
            format!("{:.1} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.1} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_string_new,
    clippy::needless_collect,
    clippy::unreadable_literal,
    clippy::used_underscore_items,
    clippy::similar_names
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_field_simple() {
        let (name, email) = parse_from_field("user@example.com");
        assert_eq!(name, "user@example.com");
        assert_eq!(email, "user@example.com");
    }

    #[test]
    fn test_parse_from_field_with_name() {
        let (name, email) = parse_from_field("John Doe <john@example.com>");
        assert_eq!(name, "John Doe");
        assert_eq!(email, "john@example.com");
    }

    #[test]
    fn test_parse_from_field_quoted_name() {
        let (name, email) = parse_from_field("\"John Doe\" <john@example.com>");
        assert_eq!(name, "John Doe");
        assert_eq!(email, "john@example.com");
    }

    #[test]
    fn test_parse_from_field_angle_brackets_in_quotes() {
        // Test that angle brackets inside quotes don't confuse the parser
        let (name, email) = parse_from_field("\"John <Admin>\" <john@example.com>");
        assert_eq!(name, "John <Admin>");
        assert_eq!(email, "john@example.com");
    }

    #[test]
    fn test_parse_from_field_empty_name() {
        let (name, email) = parse_from_field("<user@example.com>");
        assert_eq!(name, "user@example.com");
        assert_eq!(email, "user@example.com");
    }

    #[test]
    fn test_parse_from_field_complex() {
        let (name, email) = parse_from_field("\"O'Brien, Patrick\" <patrick.obrien@example.com>");
        assert_eq!(name, "O'Brien, Patrick");
        assert_eq!(email, "patrick.obrien@example.com");
    }

    #[test]
    fn test_format_date_local() {
        let date = "Thu, 15 Jan 2026 19:31:43 +0000";
        let formatted = format_date_local(date);
        // Should parse successfully (exact format depends on local timezone)
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_format_date_local_invalid() {
        let date = "invalid date";
        let formatted = format_date_local(date);
        // Should return original on parse failure
        assert_eq!(formatted, "invalid date");
    }
}
