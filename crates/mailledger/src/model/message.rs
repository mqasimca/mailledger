//! Message data models.

use super::FolderId;
use chrono::{DateTime, Local};

/// Parses a "from" field into name and email parts.
fn parse_from_field(from: &str) -> (String, String) {
    // Try to parse "Name <email@example.com>" format
    if let Some(start) = from.rfind('<')
        && let Some(end) = from.rfind('>')
    {
        let email = from[start + 1..end].to_string();
        let name = from[..start].trim().to_string();
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
        }
    }
}
