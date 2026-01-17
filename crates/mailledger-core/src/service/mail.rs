//! Mail service for IMAP operations.
//!
//! Provides high-level email operations like fetching folders,
//! messages, and managing mail state.

use mailledger_imap::command::{FetchAttribute, FetchItems, StoreAction};
use mailledger_imap::connection::{Client, ImapStream, connect_tls};
use mailledger_imap::parser::{Address, FetchItem};
use mailledger_imap::types::{Flag, Flags, MailboxStatus, Uid, UidSet};

use crate::account::Account;

/// Errors that can occur during mail operations.
#[derive(Debug, thiserror::Error)]
pub enum MailServiceError {
    /// Connection failed.
    #[error("Connection failed: {0}")]
    Connection(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Operation failed.
    #[error("Operation failed: {0}")]
    Operation(String),

    /// Security mode not supported.
    #[error("Security mode not supported: only SSL/TLS is currently supported")]
    UnsupportedSecurity,
}

/// A folder in the mailbox.
#[derive(Debug, Clone)]
pub struct Folder {
    /// Folder name.
    pub name: String,
    /// Full path (including hierarchy).
    pub path: String,
    /// Whether the folder is selectable.
    pub selectable: bool,
    /// Whether this folder has children.
    pub has_children: bool,
    /// Folder attributes (inbox, sent, drafts, etc.).
    pub folder_type: FolderType,
    /// Number of unread messages (if known).
    pub unread_count: Option<u32>,
    /// Total message count (if known).
    pub total_count: Option<u32>,
}

/// Type of folder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderType {
    /// Inbox folder.
    Inbox,
    /// Sent mail folder.
    Sent,
    /// Drafts folder.
    Drafts,
    /// Trash folder.
    Trash,
    /// Spam/junk folder.
    Spam,
    /// Archive folder.
    Archive,
    /// Regular folder.
    Regular,
}

impl FolderType {
    /// Detect folder type from name and attributes.
    #[must_use]
    pub fn from_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower == "inbox" {
            Self::Inbox
        } else if lower.contains("sent") {
            Self::Sent
        } else if lower.contains("draft") {
            Self::Drafts
        } else if lower.contains("trash") || lower.contains("deleted") {
            Self::Trash
        } else if lower.contains("spam") || lower.contains("junk") {
            Self::Spam
        } else if lower.contains("archive") {
            Self::Archive
        } else {
            Self::Regular
        }
    }
}

/// Summary of an email message.
#[derive(Debug, Clone)]
pub struct MessageSummary {
    /// Unique identifier.
    pub uid: Uid,
    /// Message subject.
    pub subject: String,
    /// Sender address.
    pub from: String,
    /// Recipient address.
    pub to: String,
    /// Date as string.
    pub date: String,
    /// Whether the message has been read.
    pub is_read: bool,
    /// Whether the message is flagged.
    pub is_flagged: bool,
    /// Whether the message has attachments.
    pub has_attachment: bool,
    /// Preview snippet of the message body.
    pub snippet: String,
}

/// Full content of an email message.
#[derive(Debug, Clone)]
pub struct MessageContent {
    /// Unique identifier.
    pub uid: Uid,
    /// Message subject.
    pub subject: String,
    /// Sender address.
    pub from: String,
    /// Recipient addresses.
    pub to: Vec<String>,
    /// CC addresses.
    pub cc: Vec<String>,
    /// Date as string.
    pub date: String,
    /// Plain text body.
    pub body_text: Option<String>,
    /// HTML body.
    pub body_html: Option<String>,
    /// List of attachments.
    pub attachments: Vec<Attachment>,
}

/// An email attachment.
#[derive(Debug, Clone)]
pub struct Attachment {
    /// Filename.
    pub filename: String,
    /// MIME type.
    pub mime_type: String,
    /// Size in bytes.
    pub size: u64,
}

/// Type alias for authenticated IMAP client with TLS stream.
pub type AuthClient = Client<ImapStream, mailledger_imap::connection::Authenticated>;

/// Type alias for selected IMAP client with TLS stream.
pub type SelectedClient = Client<ImapStream, mailledger_imap::connection::Selected>;

/// Connect to an IMAP server and authenticate.
///
/// # Errors
///
/// Returns an error if connection or authentication fails.
pub async fn connect_and_login(account: &Account) -> Result<AuthClient, MailServiceError> {
    // Check security mode
    if account.imap.security != crate::Security::Tls {
        return Err(MailServiceError::UnsupportedSecurity);
    }

    // Connect with TLS
    let stream = connect_tls(&account.imap.host, account.imap.port)
        .await
        .map_err(|e| MailServiceError::Connection(e.to_string()))?;

    // Create client and read greeting
    let client: Client<ImapStream, mailledger_imap::connection::NotAuthenticated> =
        Client::from_stream(stream)
            .await
            .map_err(|e| MailServiceError::Connection(e.to_string()))?;

    // Authenticate - try LOGIN first, fallback to AUTHENTICATE PLAIN if needed
    let auth_client = if !client.login_disabled() {
        // Try LOGIN command first (more compatible with some servers)
        match client
            .login(&account.imap.username, &account.imap.password)
            .await
        {
            Ok(authenticated) => authenticated,
            Err(e) => {
                return Err(MailServiceError::Authentication(e.to_string()));
            }
        }
    } else if client.supports_auth_plain() {
        // Fallback to AUTHENTICATE PLAIN if LOGIN is disabled
        client
            .authenticate_plain(&account.imap.username, &account.imap.password)
            .await
            .map_err(|e| MailServiceError::Authentication(e.to_string()))?
    } else {
        return Err(MailServiceError::Authentication(
            "No supported authentication method available".to_string(),
        ));
    };

    Ok(auth_client)
}

/// List all folders from an authenticated client.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn list_folders(client: &mut AuthClient) -> Result<Vec<Folder>, MailServiceError> {
    let mailboxes = client
        .list("", "*")
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))?;

    let folders = mailboxes
        .into_iter()
        .map(|mb| {
            let mailbox_name = mb.mailbox.as_str();
            let name = mailbox_name
                .rsplit_once('/')
                .map_or_else(|| mailbox_name.to_string(), |(_, n)| n.to_string());

            Folder {
                name,
                path: mailbox_name.to_string(),
                selectable: !mb
                    .attributes
                    .iter()
                    .any(|a| matches!(a, mailledger_imap::types::MailboxAttribute::NoSelect)),
                has_children: mb
                    .attributes
                    .iter()
                    .any(|a| matches!(a, mailledger_imap::types::MailboxAttribute::HasChildren)),
                folder_type: FolderType::from_name(mailbox_name),
                unread_count: None,
                total_count: None,
            }
        })
        .collect();

    Ok(folders)
}

/// Select a folder and return a selected client.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn select_folder(
    client: AuthClient,
    folder_path: &str,
) -> Result<(SelectedClient, MailboxStatus), MailServiceError> {
    client
        .select(folder_path)
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))
}

/// Fetch message summaries from the selected folder.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn fetch_messages(
    client: &mut SelectedClient,
    uid_set: &UidSet,
) -> Result<Vec<MessageSummary>, MailServiceError> {
    // Fetch envelope, flags, and UID
    let fetch_items = FetchItems::Items(vec![
        FetchAttribute::Uid,
        FetchAttribute::Flags,
        FetchAttribute::Envelope,
        FetchAttribute::Body {
            section: Some("TEXT".to_string()),
            peek: true,
            partial: Some((0, 200)),
        },
    ]);

    let responses = client
        .uid_fetch(uid_set, fetch_items)
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))?;

    let mut messages = Vec::new();
    for (_seq_num, items) in responses {
        let mut uid = None;
        let mut envelope = None;
        let mut flags = Flags::default();
        let mut body_text: Option<Vec<u8>> = None;

        // Extract items from the response
        for item in items {
            match item {
                FetchItem::Uid(u) => uid = Some(u),
                FetchItem::Envelope(e) => envelope = Some(e),
                FetchItem::Flags(f) => flags = f,
                FetchItem::Body { data, .. } => body_text = data,
                _ => {}
            }
        }

        if let Some(uid) = uid {
            let envelope = envelope.as_deref();

            messages.push(MessageSummary {
                uid,
                subject: envelope.and_then(|e| e.subject.clone()).unwrap_or_default(),
                from: envelope
                    .and_then(|e| e.from.first())
                    .map(format_address)
                    .unwrap_or_default(),
                to: envelope
                    .and_then(|e| e.to.first())
                    .map(format_address)
                    .unwrap_or_default(),
                date: envelope.and_then(|e| e.date.clone()).unwrap_or_default(),
                is_read: flags.contains(&Flag::Seen),
                is_flagged: flags.contains(&Flag::Flagged),
                has_attachment: false, // Would need BODYSTRUCTURE to detect
                snippet: body_text
                    .as_ref()
                    .map(|b| truncate_text(&extract_text_snippet(b), 100))
                    .unwrap_or_default(),
            });
        }
    }

    Ok(messages)
}

/// Mark a message as read.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn mark_read(client: &mut SelectedClient, uid: Uid) -> Result<(), MailServiceError> {
    add_flag(client, uid, Flag::Seen).await
}

/// Mark a message as unread.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn mark_unread(client: &mut SelectedClient, uid: Uid) -> Result<(), MailServiceError> {
    remove_flag(client, uid, Flag::Seen).await
}

/// Toggle flagged status.
///
/// # Errors
///
/// Returns an error if the operation fails.
pub async fn toggle_flag(
    client: &mut SelectedClient,
    uid: Uid,
    flagged: bool,
) -> Result<(), MailServiceError> {
    if flagged {
        add_flag(client, uid, Flag::Flagged).await
    } else {
        remove_flag(client, uid, Flag::Flagged).await
    }
}

/// Add a flag to a message.
async fn add_flag(
    client: &mut SelectedClient,
    uid: Uid,
    flag: Flag,
) -> Result<(), MailServiceError> {
    let uid_set = UidSet::single(uid);
    client
        .uid_store(&uid_set, StoreAction::AddFlags(vec![flag]))
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))?;
    Ok(())
}

/// Remove a flag from a message.
async fn remove_flag(
    client: &mut SelectedClient,
    uid: Uid,
    flag: Flag,
) -> Result<(), MailServiceError> {
    let uid_set = UidSet::single(uid);
    client
        .uid_store(&uid_set, StoreAction::RemoveFlags(vec![flag]))
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))?;
    Ok(())
}

/// Format an address for display.
fn format_address(addr: &Address) -> String {
    if let Some(ref name) = addr.name
        && !name.is_empty()
    {
        return name.clone();
    }

    match (&addr.mailbox, &addr.host) {
        (Some(m), Some(h)) => format!("{m}@{h}"),
        (Some(m), None) => m.clone(),
        _ => String::new(),
    }
}

/// Truncate text to a maximum length.
fn truncate_text(text: &str, max_len: usize) -> String {
    let cleaned: String = text
        .chars()
        .filter(|c| !c.is_control())
        .take(max_len)
        .collect();

    if text.chars().count() > max_len {
        format!("{cleaned}...")
    } else {
        cleaned
    }
}

/// Extract readable text from raw message body data.
///
/// This handles both single-part messages and multipart messages by
/// extracting the first text/plain content from MIME parts.
fn extract_text_snippet(raw_body: &[u8]) -> String {
    let body_str = String::from_utf8_lossy(raw_body);
    let trimmed = body_str.trim();

    // Check if this is multipart (starts with MIME boundary)
    if trimmed.starts_with("--") {
        // Extract the boundary from the first line
        let first_line_end = trimmed.find('\n').unwrap_or(trimmed.len());
        let boundary = trimmed[2..first_line_end].trim().trim_end_matches('\r');

        // Find text/plain part
        if let Some(text) = extract_text_plain_from_multipart(&body_str, boundary) {
            return text;
        }
    }

    // Not multipart or no text/plain found - return as-is
    body_str.to_string()
}

/// Extract the text/plain content from a multipart body.
fn extract_text_plain_from_multipart(body: &str, boundary: &str) -> Option<String> {
    let delimiter = format!("--{boundary}");

    for part in body.split(&delimiter) {
        let trimmed = part.trim();

        // Skip empty parts and closing boundary
        if trimmed.is_empty() || trimmed == "--" || trimmed.starts_with("--") {
            continue;
        }

        // Split this part into headers and content
        let (part_headers, part_content) = split_headers_body(trimmed);

        // Check if this is text/plain
        let content_type = get_header(&part_headers, "content-type")
            .unwrap_or("")
            .to_lowercase();

        if content_type.is_empty() || content_type.contains("text/plain") {
            // Found text/plain part - decode it
            return Some(decode_part(&part_content, &part_headers));
        }

        // Check for nested multipart/alternative
        if content_type.contains("multipart/alternative")
            && let Some(nested_boundary) = extract_boundary(&part_headers)
            && let Some(text) = extract_text_plain_from_multipart(&part_content, &nested_boundary)
        {
            return Some(text);
        }
    }

    None
}

/// Event received from IDLE monitoring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleEvent {
    /// New messages in the mailbox.
    NewMail(u32),
    /// A message was expunged.
    Expunge,
    /// Flags changed on a message.
    FlagsChanged,
    /// Connection timed out (should restart IDLE).
    Timeout,
    /// Connection was lost.
    Disconnected(String),
}

/// Fetch full content for a single message.
///
/// # Errors
///
/// Returns an error if the fetch operation fails.
pub async fn fetch_message_content(
    client: &mut SelectedClient,
    uid: Uid,
) -> Result<Option<MessageContent>, MailServiceError> {
    let uid_set = UidSet::single(uid);

    // Fetch full body and envelope
    let fetch_items = FetchItems::Items(vec![
        FetchAttribute::Uid,
        FetchAttribute::Flags,
        FetchAttribute::Envelope,
        FetchAttribute::Body {
            section: None, // Full message
            peek: true,
            partial: None,
        },
    ]);

    let responses = client
        .uid_fetch(&uid_set, fetch_items)
        .await
        .map_err(|e| MailServiceError::Operation(e.to_string()))?;

    for (_seq_num, items) in responses {
        let mut msg_uid = None;
        let mut envelope = None;
        let mut body_data: Option<Vec<u8>> = None;

        for item in items {
            match item {
                FetchItem::Uid(u) => msg_uid = Some(u),
                FetchItem::Envelope(e) => envelope = Some(e),
                FetchItem::Body { data, .. } => body_data = data,
                _ => {}
            }
        }

        if let Some(uid) = msg_uid {
            let envelope = envelope.as_deref();

            // Parse the body to extract text/html parts
            let (body_text, body_html) =
                body_data.map_or((None, None), |raw_body| parse_message_body(&raw_body));

            return Ok(Some(MessageContent {
                uid,
                subject: envelope.and_then(|e| e.subject.clone()).unwrap_or_default(),
                from: envelope
                    .and_then(|e| e.from.first())
                    .map(format_address)
                    .unwrap_or_default(),
                to: envelope
                    .map(|e| e.to.iter().map(format_address).collect())
                    .unwrap_or_default(),
                cc: envelope
                    .map(|e| e.cc.iter().map(format_address).collect())
                    .unwrap_or_default(),
                date: envelope.and_then(|e| e.date.clone()).unwrap_or_default(),
                body_text,
                body_html,
                attachments: Vec::new(), // TODO: Parse attachments from body structure
            }));
        }
    }

    Ok(None)
}

/// Parse raw message body to extract text and HTML parts.
///
/// This handles both single-part and multipart MIME messages.
#[allow(clippy::option_if_let_else)] // Early return pattern is clearer here
fn parse_message_body(raw_body: &[u8]) -> (Option<String>, Option<String>) {
    let body_str = String::from_utf8_lossy(raw_body);

    // Split headers from body (headers end at first blank line)
    let (headers, body) = split_headers_body(&body_str);

    // Check for multipart boundary in Content-Type header
    if let Some(boundary) = extract_boundary(&headers) {
        let mut text_body = None;
        let mut html_body = None;

        // Parse multipart body
        let parts = split_multipart(&body, &boundary);
        for part in parts {
            let (part_headers, part_body) = split_headers_body(&part);

            // Get content type of this part
            let content_type = get_header(&part_headers, "content-type")
                .unwrap_or_default()
                .to_lowercase();

            let decoded_body = decode_part(&part_body, &part_headers);

            if content_type.contains("text/plain") && text_body.is_none() {
                text_body = Some(decoded_body);
            } else if content_type.contains("text/html") && html_body.is_none() {
                html_body = Some(decoded_body);
            } else if content_type.contains("multipart/alternative") {
                // Nested multipart - recursively parse
                if let Some(nested_boundary) = extract_boundary(&part_headers) {
                    let nested_parts = split_multipart(&part_body, &nested_boundary);
                    for nested_part in nested_parts {
                        let (np_headers, np_body) = split_headers_body(&nested_part);
                        let np_ct = get_header(&np_headers, "content-type")
                            .unwrap_or_default()
                            .to_lowercase();
                        let np_decoded = decode_part(&np_body, &np_headers);

                        if np_ct.contains("text/plain") && text_body.is_none() {
                            text_body = Some(np_decoded);
                        } else if np_ct.contains("text/html") && html_body.is_none() {
                            html_body = Some(np_decoded);
                        }
                    }
                }
            }
        }

        (text_body, html_body)
    } else {
        // Single-part message
        let content_type = get_header(&headers, "content-type")
            .unwrap_or_default()
            .to_lowercase();

        let decoded = decode_part(&body, &headers);

        if content_type.contains("text/html") {
            (None, Some(decoded))
        } else {
            // Default to text/plain
            (Some(decoded), None)
        }
    }
}

/// Split message into headers and body at the first blank line.
#[allow(clippy::option_if_let_else)] // Chained if-let is clearer here
fn split_headers_body(message: &str) -> (String, String) {
    // Look for \r\n\r\n or \n\n separator
    if let Some(idx) = message.find("\r\n\r\n") {
        (message[..idx].to_string(), message[idx + 4..].to_string())
    } else if let Some(idx) = message.find("\n\n") {
        (message[..idx].to_string(), message[idx + 2..].to_string())
    } else {
        // No body found
        (message.to_string(), String::new())
    }
}

/// Extract boundary parameter from Content-Type header.
fn extract_boundary(headers: &str) -> Option<String> {
    let content_type = get_header(headers, "content-type")?;

    // Look for boundary="value" or boundary=value
    let lower = content_type.to_lowercase();
    if let Some(idx) = lower.find("boundary=") {
        let boundary_start = idx + 9;
        let rest = &content_type[boundary_start..];

        if let Some(stripped) = rest.strip_prefix('"') {
            // Quoted boundary - find closing quote
            let end = stripped.find('"')?;
            Some(stripped[..end].to_string())
        } else {
            // Unquoted boundary - take until space, semicolon, or end
            let end = rest
                .find(|c: char| c.is_whitespace() || c == ';')
                .unwrap_or(rest.len());
            Some(rest[..end].to_string())
        }
    } else {
        None
    }
}

/// Get a header value from raw headers.
fn get_header<'a>(headers: &'a str, name: &str) -> Option<&'a str> {
    let name_lower = name.to_lowercase();

    for line in headers.lines() {
        // Handle line continuations (skip lines starting with whitespace as they're continuations)
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }

        if let Some(colon_idx) = line.find(':') {
            let header_name = line[..colon_idx].trim().to_lowercase();
            if header_name == name_lower {
                return Some(line[colon_idx + 1..].trim());
            }
        }
    }
    None
}

/// Split multipart body into parts using boundary.
fn split_multipart(body: &str, boundary: &str) -> Vec<String> {
    let delimiter = format!("--{boundary}");
    let end_delimiter = format!("--{boundary}--");

    let mut parts = Vec::new();

    for part in body.split(&delimiter) {
        let trimmed = part.trim();

        // Skip empty parts and the final closing boundary
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        // Remove any trailing end-delimiter
        let clean = trimmed
            .strip_suffix(&format!("--{boundary}--"))
            .unwrap_or(trimmed);
        let clean = clean.strip_suffix(&end_delimiter).unwrap_or(clean);

        if !clean.trim().is_empty() {
            parts.push(clean.to_string());
        }
    }

    parts
}

/// Decode part body based on Content-Transfer-Encoding header.
fn decode_part(body: &str, headers: &str) -> String {
    let encoding = get_header(headers, "content-transfer-encoding")
        .unwrap_or("7bit")
        .to_lowercase();

    match encoding.as_str() {
        "base64" => {
            // Remove whitespace and decode
            let cleaned: String = body.chars().filter(|c| !c.is_whitespace()).collect();
            mailledger_mime::encoding::decode_base64(&cleaned)
                .and_then(|bytes| String::from_utf8(bytes).map_err(Into::into))
                .unwrap_or_else(|_| body.to_string())
        }
        "quoted-printable" => mailledger_mime::encoding::decode_quoted_printable(body)
            .unwrap_or_else(|_| body.to_string()),
        _ => body.to_string(),
    }
}

/// Start IDLE monitoring on a folder.
///
/// This function connects to the IMAP server, selects the specified folder,
/// enters IDLE mode, waits for an event, and returns the event.
///
/// The caller should restart IDLE monitoring after handling the event.
///
/// # Errors
///
/// Returns an error if connection or IDLE fails.
pub async fn idle_monitor(
    account: &Account,
    folder_path: &str,
    timeout_secs: u64,
) -> Result<IdleEvent, MailServiceError> {
    use mailledger_imap::IdleEvent as ImapIdleEvent;
    use std::time::Duration;

    // Connect and authenticate
    let auth_client = connect_and_login(account).await?;

    // Select the folder
    let (mut selected_client, _status) = select_folder(auth_client, folder_path).await?;

    // Enter IDLE mode
    let mut idle_handle = selected_client
        .idle()
        .await
        .map_err(|e| MailServiceError::Operation(format!("IDLE failed: {e}")))?;

    // Wait for an event
    let timeout_duration = Duration::from_secs(timeout_secs);
    let event = idle_handle
        .wait(timeout_duration)
        .await
        .map_err(|e| MailServiceError::Operation(format!("IDLE wait failed: {e}")))?;

    // Exit IDLE mode
    idle_handle
        .done()
        .await
        .map_err(|e| MailServiceError::Operation(format!("IDLE done failed: {e}")))?;

    // Convert to our event type
    Ok(match event {
        ImapIdleEvent::Exists(count) => IdleEvent::NewMail(count),
        ImapIdleEvent::Expunge(_) => IdleEvent::Expunge,
        ImapIdleEvent::Fetch { .. } => IdleEvent::FlagsChanged,
        ImapIdleEvent::Recent(_) => IdleEvent::NewMail(0),
        ImapIdleEvent::Timeout => IdleEvent::Timeout,
    })
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

    // ===== FolderType::from_name tests =====

    mod folder_type {
        use super::*;

        #[test]
        fn test_inbox() {
            assert_eq!(FolderType::from_name("INBOX"), FolderType::Inbox);
            assert_eq!(FolderType::from_name("inbox"), FolderType::Inbox);
            assert_eq!(FolderType::from_name("Inbox"), FolderType::Inbox);
        }

        #[test]
        fn test_sent() {
            assert_eq!(FolderType::from_name("Sent"), FolderType::Sent);
            assert_eq!(FolderType::from_name("Sent Mail"), FolderType::Sent);
            assert_eq!(FolderType::from_name("Sent Items"), FolderType::Sent);
            assert_eq!(FolderType::from_name("[Gmail]/Sent Mail"), FolderType::Sent);
        }

        #[test]
        fn test_drafts() {
            assert_eq!(FolderType::from_name("Drafts"), FolderType::Drafts);
            assert_eq!(FolderType::from_name("Draft"), FolderType::Drafts);
            assert_eq!(FolderType::from_name("[Gmail]/Drafts"), FolderType::Drafts);
        }

        #[test]
        fn test_trash() {
            assert_eq!(FolderType::from_name("Trash"), FolderType::Trash);
            assert_eq!(FolderType::from_name("Deleted Items"), FolderType::Trash);
            assert_eq!(FolderType::from_name("[Gmail]/Trash"), FolderType::Trash);
        }

        #[test]
        fn test_spam() {
            assert_eq!(FolderType::from_name("Spam"), FolderType::Spam);
            assert_eq!(FolderType::from_name("Junk"), FolderType::Spam);
            assert_eq!(FolderType::from_name("Junk E-mail"), FolderType::Spam);
            assert_eq!(FolderType::from_name("[Gmail]/Spam"), FolderType::Spam);
        }

        #[test]
        fn test_archive() {
            assert_eq!(FolderType::from_name("Archive"), FolderType::Archive);
            assert_eq!(FolderType::from_name("All Mail"), FolderType::Regular);
            assert_eq!(
                FolderType::from_name("[Gmail]/All Mail"),
                FolderType::Regular
            );
        }

        #[test]
        fn test_regular() {
            assert_eq!(FolderType::from_name("Work"), FolderType::Regular);
            assert_eq!(FolderType::from_name("Personal"), FolderType::Regular);
            assert_eq!(FolderType::from_name("Projects/2024"), FolderType::Regular);
        }
    }

    // ===== format_address tests =====

    mod format_address_tests {
        use super::*;

        /// Helper to create an Address for testing.
        fn make_address(name: Option<&str>, mailbox: Option<&str>, host: Option<&str>) -> Address {
            Address {
                name: name.map(ToString::to_string),
                adl: None,
                mailbox: mailbox.map(ToString::to_string),
                host: host.map(ToString::to_string),
            }
        }

        #[test]
        fn test_with_name() {
            let addr = make_address(Some("John Doe"), Some("john"), Some("example.com"));
            assert_eq!(format_address(&addr), "John Doe");
        }

        #[test]
        fn test_without_name_full_email() {
            let addr = make_address(None, Some("jane"), Some("example.org"));
            assert_eq!(format_address(&addr), "jane@example.org");
        }

        #[test]
        fn test_empty_name_uses_email() {
            let addr = make_address(Some(""), Some("user"), Some("domain.com"));
            assert_eq!(format_address(&addr), "user@domain.com");
        }

        #[test]
        fn test_mailbox_only() {
            let addr = make_address(None, Some("local"), None);
            assert_eq!(format_address(&addr), "local");
        }

        #[test]
        fn test_empty_address() {
            let addr = make_address(None, None, None);
            assert_eq!(format_address(&addr), "");
        }
    }

    // ===== truncate_text tests =====

    mod truncate_text_tests {
        use super::*;

        #[test]
        fn test_short_text() {
            assert_eq!(truncate_text("Hello", 10), "Hello");
        }

        #[test]
        fn test_exact_length() {
            assert_eq!(truncate_text("1234567890", 10), "1234567890");
        }

        #[test]
        fn test_long_text() {
            assert_eq!(truncate_text("Hello, World!", 5), "Hello...");
        }

        #[test]
        fn test_empty_text() {
            assert_eq!(truncate_text("", 10), "");
        }

        #[test]
        fn test_removes_control_chars() {
            assert_eq!(truncate_text("Hello\x00World", 20), "HelloWorld");
        }

        #[test]
        fn test_newlines_removed() {
            assert_eq!(truncate_text("Line1\nLine2", 20), "Line1Line2");
        }
    }

    // ===== split_headers_body tests =====

    mod split_headers_body_tests {
        use super::*;

        #[test]
        fn test_crlf_separator() {
            let message = "From: test@example.com\r\nSubject: Test\r\n\r\nBody text";
            let (headers, body) = split_headers_body(message);
            assert_eq!(headers, "From: test@example.com\r\nSubject: Test");
            assert_eq!(body, "Body text");
        }

        #[test]
        fn test_lf_separator() {
            let message = "From: test@example.com\nSubject: Test\n\nBody text";
            let (headers, body) = split_headers_body(message);
            assert_eq!(headers, "From: test@example.com\nSubject: Test");
            assert_eq!(body, "Body text");
        }

        #[test]
        fn test_headers_only() {
            let message = "From: test@example.com\r\nSubject: Test";
            let (headers, body) = split_headers_body(message);
            assert_eq!(headers, message);
            assert_eq!(body, "");
        }

        #[test]
        fn test_empty_body() {
            let message = "Subject: Empty\r\n\r\n";
            let (headers, body) = split_headers_body(message);
            assert_eq!(headers, "Subject: Empty");
            assert_eq!(body, "");
        }
    }

    // ===== get_header tests =====

    mod get_header_tests {
        use super::*;

        #[test]
        fn test_find_header() {
            let headers = "From: sender@example.com\r\nTo: receiver@example.com\r\nSubject: Test";
            assert_eq!(get_header(headers, "From"), Some("sender@example.com"));
            assert_eq!(get_header(headers, "To"), Some("receiver@example.com"));
            assert_eq!(get_header(headers, "Subject"), Some("Test"));
        }

        #[test]
        fn test_case_insensitive() {
            let headers = "Content-Type: text/plain";
            assert_eq!(get_header(headers, "content-type"), Some("text/plain"));
            assert_eq!(get_header(headers, "CONTENT-TYPE"), Some("text/plain"));
            assert_eq!(get_header(headers, "Content-Type"), Some("text/plain"));
        }

        #[test]
        fn test_missing_header() {
            let headers = "From: test@example.com";
            assert_eq!(get_header(headers, "Subject"), None);
        }

        #[test]
        fn test_value_trimmed() {
            let headers = "Subject:   Test Subject   ";
            assert_eq!(get_header(headers, "Subject"), Some("Test Subject"));
        }

        #[test]
        fn test_empty_headers() {
            assert_eq!(get_header("", "Subject"), None);
        }
    }

    // ===== extract_boundary tests =====

    mod extract_boundary_tests {
        use super::*;

        #[test]
        fn test_quoted_boundary() {
            let headers = "Content-Type: multipart/mixed; boundary=\"----=_Part_123\"";
            assert_eq!(
                extract_boundary(headers),
                Some("----=_Part_123".to_string())
            );
        }

        #[test]
        fn test_unquoted_boundary() {
            let headers = "Content-Type: multipart/alternative; boundary=simple_boundary";
            assert_eq!(
                extract_boundary(headers),
                Some("simple_boundary".to_string())
            );
        }

        #[test]
        fn test_boundary_with_semicolon_after() {
            let headers = "Content-Type: multipart/mixed; boundary=boundary123; charset=utf-8";
            assert_eq!(extract_boundary(headers), Some("boundary123".to_string()));
        }

        #[test]
        fn test_no_boundary() {
            let headers = "Content-Type: text/plain; charset=utf-8";
            assert_eq!(extract_boundary(headers), None);
        }

        #[test]
        fn test_no_content_type() {
            let headers = "Subject: Test";
            assert_eq!(extract_boundary(headers), None);
        }
    }

    // ===== split_multipart tests =====

    mod split_multipart_tests {
        use super::*;

        #[test]
        fn test_two_parts() {
            let body = "--boundary\r\nPart 1\r\n--boundary\r\nPart 2\r\n--boundary--";
            let parts = split_multipart(body, "boundary");
            assert_eq!(parts.len(), 2);
            assert!(parts[0].contains("Part 1"));
            assert!(parts[1].contains("Part 2"));
        }

        #[test]
        fn test_single_part() {
            let body = "--boundary\r\nOnly part\r\n--boundary--";
            let parts = split_multipart(body, "boundary");
            assert_eq!(parts.len(), 1);
            assert!(parts[0].contains("Only part"));
        }

        #[test]
        fn test_empty_body() {
            let body = "--boundary\r\n--boundary--";
            let parts = split_multipart(body, "boundary");
            assert!(parts.is_empty());
        }

        #[test]
        fn test_complex_boundary() {
            let body = "--=_Part_123\r\nContent\r\n--=_Part_123--";
            let parts = split_multipart(body, "=_Part_123");
            assert_eq!(parts.len(), 1);
        }
    }

    // ===== decode_part tests =====

    mod decode_part_tests {
        use super::*;

        #[test]
        fn test_7bit_passthrough() {
            let body = "Hello, World!";
            let headers = "Content-Transfer-Encoding: 7bit";
            assert_eq!(decode_part(body, headers), "Hello, World!");
        }

        #[test]
        fn test_8bit_passthrough() {
            let body = "Hello, Wörld!";
            let headers = "Content-Transfer-Encoding: 8bit";
            assert_eq!(decode_part(body, headers), "Hello, Wörld!");
        }

        #[test]
        fn test_no_encoding_header() {
            let body = "Plain text";
            let headers = "Content-Type: text/plain";
            assert_eq!(decode_part(body, headers), "Plain text");
        }

        #[test]
        fn test_base64_decode() {
            let body = "SGVsbG8sIFdvcmxkIQ==";
            let headers = "Content-Transfer-Encoding: base64";
            assert_eq!(decode_part(body, headers), "Hello, World!");
        }

        #[test]
        fn test_base64_with_whitespace() {
            let body = "SGVs\r\nbG8s\r\nIFdvcmxkIQ==";
            let headers = "Content-Transfer-Encoding: base64";
            assert_eq!(decode_part(body, headers), "Hello, World!");
        }

        #[test]
        fn test_quoted_printable() {
            let body = "Hello=20World";
            let headers = "Content-Transfer-Encoding: quoted-printable";
            assert_eq!(decode_part(body, headers), "Hello World");
        }
    }

    // ===== parse_message_body tests =====

    mod parse_message_body_tests {
        use super::*;

        #[test]
        fn test_plain_text_only() {
            let raw = b"Content-Type: text/plain\r\n\r\nHello, World!";
            let (text, html) = parse_message_body(raw);
            assert_eq!(text, Some("Hello, World!".to_string()));
            assert_eq!(html, None);
        }

        #[test]
        fn test_html_only() {
            let raw = b"Content-Type: text/html\r\n\r\n<p>Hello</p>";
            let (text, html) = parse_message_body(raw);
            assert_eq!(text, None);
            assert_eq!(html, Some("<p>Hello</p>".to_string()));
        }

        #[test]
        fn test_no_content_type_defaults_to_text() {
            let raw = b"Subject: Test\r\n\r\nPlain body";
            let (text, html) = parse_message_body(raw);
            assert_eq!(text, Some("Plain body".to_string()));
            assert_eq!(html, None);
        }

        #[test]
        fn test_multipart_alternative() {
            let raw = concat!(
                "Content-Type: multipart/alternative; boundary=\"boundary\"\r\n",
                "\r\n",
                "--boundary\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "Plain text version\r\n",
                "--boundary\r\n",
                "Content-Type: text/html\r\n",
                "\r\n",
                "<p>HTML version</p>\r\n",
                "--boundary--"
            );
            let (text, html) = parse_message_body(raw.as_bytes());
            assert_eq!(text, Some("Plain text version".to_string()));
            assert_eq!(html, Some("<p>HTML version</p>".to_string()));
        }

        #[test]
        fn test_multipart_with_base64() {
            let raw = concat!(
                "Content-Type: multipart/mixed; boundary=\"bound\"\r\n",
                "\r\n",
                "--bound\r\n",
                "Content-Type: text/plain\r\n",
                "Content-Transfer-Encoding: base64\r\n",
                "\r\n",
                "SGVsbG8=\r\n",
                "--bound--"
            );
            let (text, html) = parse_message_body(raw.as_bytes());
            assert_eq!(text, Some("Hello".to_string()));
            assert_eq!(html, None);
        }

        #[test]
        fn test_empty_body() {
            let raw = b"Subject: Empty\r\n\r\n";
            let (text, html) = parse_message_body(raw);
            assert_eq!(text, Some(String::new()));
            assert_eq!(html, None);
        }
    }

    // ===== extract_text_snippet tests =====

    mod extract_text_snippet_tests {
        use super::*;

        #[test]
        fn test_plain_text_body() {
            let raw = b"Hello, this is a simple message.";
            assert_eq!(
                extract_text_snippet(raw),
                "Hello, this is a simple message."
            );
        }

        #[test]
        fn test_multipart_extracts_text_plain() {
            // This simulates what BODY[TEXT] returns for a multipart message
            let raw = concat!(
                "--000000000000abc123\r\n",
                "Content-Type: text/plain; charset=\"UTF-8\"\r\n",
                "\r\n",
                "This is the plain text content.\r\n",
                "--000000000000abc123\r\n",
                "Content-Type: text/html; charset=\"UTF-8\"\r\n",
                "\r\n",
                "<p>This is HTML</p>\r\n",
                "--000000000000abc123--"
            );
            let result = extract_text_snippet(raw.as_bytes());
            assert!(result.contains("This is the plain text content"));
            assert!(!result.contains("--000000000000abc123"));
            assert!(!result.contains("<p>"));
        }

        #[test]
        fn test_multipart_with_nested_alternative() {
            let raw = concat!(
                "--outer\r\n",
                "Content-Type: multipart/alternative; boundary=\"inner\"\r\n",
                "\r\n",
                "--inner\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "Nested plain text\r\n",
                "--inner\r\n",
                "Content-Type: text/html\r\n",
                "\r\n",
                "<p>Nested HTML</p>\r\n",
                "--inner--\r\n",
                "--outer--"
            );
            let result = extract_text_snippet(raw.as_bytes());
            assert!(result.contains("Nested plain text"));
        }

        #[test]
        fn test_multipart_base64_encoded() {
            let raw = concat!(
                "--boundary\r\n",
                "Content-Type: text/plain; charset=\"UTF-8\"\r\n",
                "Content-Transfer-Encoding: base64\r\n",
                "\r\n",
                "SGVsbG8gV29ybGQ=\r\n", // "Hello World" in base64
                "--boundary--"
            );
            let result = extract_text_snippet(raw.as_bytes());
            assert_eq!(result, "Hello World");
        }
    }
}
