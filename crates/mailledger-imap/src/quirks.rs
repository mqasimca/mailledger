//! Server quirks and workarounds.
//!
//! Different IMAP servers have varying interpretations of the RFC and
//! non-standard behaviors. This module provides detection and workarounds
//! for common server quirks.

use crate::types::Capability;

/// Known IMAP server types with specific quirks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerType {
    /// Unknown or generic IMAP server.
    #[default]
    Unknown,
    /// Gmail IMAP (imap.gmail.com).
    Gmail,
    /// Microsoft Outlook/Office 365.
    Outlook,
    /// Yahoo Mail.
    Yahoo,
    /// Apple iCloud Mail.
    ICloud,
    /// Fastmail.
    Fastmail,
    /// Dovecot (common open-source server).
    Dovecot,
    /// Courier IMAP.
    Courier,
    /// Cyrus IMAP.
    Cyrus,
}

impl ServerType {
    /// Detects the server type from capabilities and greeting.
    #[must_use]
    pub fn detect(capabilities: &[Capability], greeting: Option<&str>) -> Self {
        // Check for Gmail-specific extensions
        for cap in capabilities {
            if let Capability::Unknown(s) = cap {
                let upper = s.to_uppercase();
                if upper.starts_with("X-GM-") {
                    return Self::Gmail;
                }
                if upper.contains("XLIST") && upper.contains("XYMHIGHESTMODSEQ") {
                    return Self::Yahoo;
                }
            }
        }

        // Check greeting for server identification
        if let Some(greeting) = greeting {
            let lower = greeting.to_lowercase();
            if lower.contains("gimap") || lower.contains("gmail") {
                return Self::Gmail;
            }
            if lower.contains("outlook") || lower.contains("microsoft") {
                return Self::Outlook;
            }
            if lower.contains("dovecot") {
                return Self::Dovecot;
            }
            if lower.contains("courier") {
                return Self::Courier;
            }
            if lower.contains("cyrus") {
                return Self::Cyrus;
            }
            if lower.contains("fastmail") {
                return Self::Fastmail;
            }
            if lower.contains("icloud") || lower.contains("apple") {
                return Self::ICloud;
            }
        }

        Self::Unknown
    }
}

/// Server-specific quirks and workarounds.
#[derive(Debug, Clone, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct ServerQuirks {
    /// The detected server type.
    pub server_type: ServerType,

    /// Gmail uses labels instead of folders, with special semantics.
    pub gmail_labels: bool,

    /// Some servers require INBOX to be uppercase.
    pub inbox_case_sensitive: bool,

    /// Maximum recommended IDLE duration before re-issuing.
    /// Gmail has a 10-minute timeout, most servers 29 minutes.
    pub idle_timeout_secs: u32,

    /// Server supports LITERAL+ for non-synchronizing literals.
    pub literal_plus: bool,

    /// Server may send untagged responses out of order.
    pub unordered_responses: bool,

    /// Server requires explicit EXPUNGE (doesn't auto-expunge on CLOSE).
    pub explicit_expunge: bool,

    /// Server supports the MOVE command natively.
    pub native_move: bool,

    /// Server may include extra whitespace in responses.
    pub lenient_parsing: bool,
}

impl ServerQuirks {
    /// Creates quirks configuration for the detected server type.
    #[must_use]
    pub fn for_server(server_type: ServerType, capabilities: &[Capability]) -> Self {
        let has_move = capabilities.iter().any(|c| matches!(c, Capability::Move));
        let has_literal_plus = capabilities
            .iter()
            .any(|c| matches!(c, Capability::LiteralPlus | Capability::LiteralMinus));

        let base = Self {
            server_type,
            native_move: has_move,
            literal_plus: has_literal_plus,
            lenient_parsing: true, // Enable lenient parsing by default
            ..Default::default()
        };

        match server_type {
            ServerType::Gmail => Self {
                gmail_labels: true,
                inbox_case_sensitive: false,
                idle_timeout_secs: 600, // 10 minutes
                unordered_responses: true,
                ..base
            },
            ServerType::Outlook | ServerType::Fastmail => Self {
                inbox_case_sensitive: false,
                idle_timeout_secs: 1740, // 29 minutes
                ..base
            },
            ServerType::Yahoo | ServerType::ICloud => Self {
                inbox_case_sensitive: false,
                idle_timeout_secs: 1200, // 20 minutes
                ..base
            },
            ServerType::Dovecot => Self {
                inbox_case_sensitive: false,
                idle_timeout_secs: 1740,
                explicit_expunge: false,
                ..base
            },
            ServerType::Courier | ServerType::Cyrus => Self {
                inbox_case_sensitive: true,
                idle_timeout_secs: 1740,
                explicit_expunge: true,
                ..base
            },
            ServerType::Unknown => Self {
                inbox_case_sensitive: false,
                idle_timeout_secs: 600, // Conservative default
                ..base
            },
        }
    }

    /// Returns the recommended IDLE timeout as a Duration.
    #[must_use]
    pub fn idle_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(u64::from(self.idle_timeout_secs))
    }

    /// Normalizes a mailbox name according to server quirks.
    ///
    /// For example, ensures INBOX is uppercase when required.
    #[must_use]
    pub fn normalize_mailbox(&self, mailbox: &str) -> String {
        if !self.inbox_case_sensitive && mailbox.eq_ignore_ascii_case("inbox") {
            return "INBOX".to_string();
        }
        mailbox.to_string()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_gmail() {
        let caps = vec![Capability::Unknown("X-GM-EXT-1".to_string())];
        assert_eq!(ServerType::detect(&caps, None), ServerType::Gmail);
    }

    #[test]
    fn test_detect_from_greeting() {
        let caps = vec![Capability::Imap4Rev1];
        assert_eq!(
            ServerType::detect(&caps, Some("* OK Dovecot ready.")),
            ServerType::Dovecot
        );
    }

    #[test]
    fn test_gmail_quirks() {
        let quirks = ServerQuirks::for_server(ServerType::Gmail, &[]);
        assert!(quirks.gmail_labels);
        assert_eq!(quirks.idle_timeout_secs, 600);
    }

    #[test]
    fn test_normalize_mailbox() {
        let quirks = ServerQuirks::for_server(ServerType::Unknown, &[]);
        assert_eq!(quirks.normalize_mailbox("inbox"), "INBOX");
        assert_eq!(quirks.normalize_mailbox("Sent"), "Sent");
    }
}
