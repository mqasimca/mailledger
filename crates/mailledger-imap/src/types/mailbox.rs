//! Mailbox types.

use super::{Flags, SeqNum, Uid, UidValidity};

/// Mailbox name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mailbox(pub String);

impl Mailbox {
    /// Creates a new mailbox name.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// The INBOX mailbox (case-insensitive per RFC).
    #[must_use]
    pub fn inbox() -> Self {
        Self("INBOX".to_string())
    }

    /// Returns the mailbox name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Mailbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Mailbox status information from SELECT/EXAMINE.
#[derive(Debug, Clone, Default)]
pub struct MailboxStatus {
    /// Number of messages in the mailbox.
    pub exists: u32,
    /// Number of recent messages.
    pub recent: u32,
    /// First unseen message sequence number.
    pub unseen: Option<SeqNum>,
    /// Next UID to be assigned.
    pub uid_next: Option<Uid>,
    /// UIDVALIDITY value.
    pub uid_validity: Option<UidValidity>,
    /// Flags defined for this mailbox.
    pub flags: Flags,
    /// Flags that can be permanently stored.
    pub permanent_flags: Flags,
    /// Whether mailbox is read-only.
    pub read_only: bool,
    /// Highest mod-sequence (if CONDSTORE enabled).
    pub highest_mod_seq: Option<u64>,
}

/// LIST response data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListResponse {
    /// Mailbox attributes.
    pub attributes: Vec<MailboxAttribute>,
    /// Hierarchy delimiter.
    pub delimiter: Option<char>,
    /// Mailbox name.
    pub mailbox: Mailbox,
}

/// Mailbox attributes from LIST response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MailboxAttribute {
    /// Mailbox cannot be selected.
    NoSelect,
    /// Mailbox has no children.
    HasNoChildren,
    /// Mailbox has children.
    HasChildren,
    /// Mailbox is marked for attention.
    Marked,
    /// Mailbox is not marked.
    Unmarked,
    // SPECIAL-USE mailbox attributes (RFC 6154)
    /// All messages (virtual mailbox).
    All,
    /// Mailbox is the archive folder.
    Archive,
    /// Mailbox is the drafts folder.
    Drafts,
    /// Flagged/starred messages (virtual mailbox).
    Flagged,
    /// Mailbox is the junk/spam folder.
    Junk,
    /// Mailbox is the sent folder.
    Sent,
    /// Mailbox is the trash folder.
    Trash,
    /// Important messages (RFC 8457).
    Important,
    // Other attributes
    /// Mailbox is subscribed.
    Subscribed,
    /// Unknown attribute.
    Unknown(String),
}

impl MailboxAttribute {
    /// Parses a mailbox attribute string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "\\NOSELECT" => Self::NoSelect,
            "\\HASNOCHILDREN" => Self::HasNoChildren,
            "\\HASCHILDREN" => Self::HasChildren,
            "\\MARKED" => Self::Marked,
            "\\UNMARKED" => Self::Unmarked,
            // RFC 6154 SPECIAL-USE
            "\\ALL" => Self::All,
            "\\ARCHIVE" => Self::Archive,
            "\\DRAFTS" => Self::Drafts,
            "\\FLAGGED" => Self::Flagged,
            "\\JUNK" | "\\SPAM" => Self::Junk,
            "\\SENT" => Self::Sent,
            "\\TRASH" => Self::Trash,
            // RFC 8457
            "\\IMPORTANT" => Self::Important,
            // Other
            "\\SUBSCRIBED" => Self::Subscribed,
            _ => Self::Unknown(s.to_string()),
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

    mod mailbox_tests {
        use super::*;

        #[test]
        fn new_from_string() {
            let mb = Mailbox::new("Archive".to_string());
            assert_eq!(mb.as_str(), "Archive");
        }

        #[test]
        fn new_from_str() {
            let mb = Mailbox::new("Drafts");
            assert_eq!(mb.as_str(), "Drafts");
        }

        #[test]
        fn inbox() {
            let inbox = Mailbox::inbox();
            assert_eq!(inbox.as_str(), "INBOX");
        }

        #[test]
        fn display() {
            let mb = Mailbox::new("Sent");
            assert_eq!(format!("{mb}"), "Sent");
        }

        #[test]
        fn equality() {
            let mb1 = Mailbox::new("INBOX");
            let mb2 = Mailbox::new("INBOX");
            let mb3 = Mailbox::new("Sent");
            assert_eq!(mb1, mb2);
            assert_ne!(mb1, mb3);
        }
    }

    mod mailbox_status_tests {
        use super::*;
        use crate::types::Flag;

        #[test]
        fn default() {
            let status = MailboxStatus::default();
            assert_eq!(status.exists, 0);
            assert_eq!(status.recent, 0);
            assert!(status.unseen.is_none());
            assert!(status.uid_next.is_none());
            assert!(status.uid_validity.is_none());
            assert!(status.flags.is_empty());
            assert!(status.permanent_flags.is_empty());
            assert!(!status.read_only);
            assert!(status.highest_mod_seq.is_none());
        }

        #[test]
        fn with_values() {
            let status = MailboxStatus {
                exists: 100,
                recent: 5,
                unseen: SeqNum::new(50),
                uid_next: Uid::new(101),
                uid_validity: UidValidity::new(123456),
                flags: Flags::from_vec(vec![Flag::Seen, Flag::Flagged]),
                permanent_flags: Flags::new(),
                read_only: false,
                highest_mod_seq: Some(999),
            };
            assert_eq!(status.exists, 100);
            assert_eq!(status.recent, 5);
            assert_eq!(status.unseen.unwrap().get(), 50);
            assert_eq!(status.highest_mod_seq, Some(999));
        }
    }

    mod list_response_tests {
        use super::*;

        #[test]
        fn with_attributes() {
            let resp = ListResponse {
                attributes: vec![MailboxAttribute::HasChildren, MailboxAttribute::Sent],
                delimiter: Some('/'),
                mailbox: Mailbox::new("Sent"),
            };
            assert_eq!(resp.attributes.len(), 2);
            assert_eq!(resp.delimiter, Some('/'));
            assert_eq!(resp.mailbox.as_str(), "Sent");
        }

        #[test]
        fn no_delimiter() {
            let resp = ListResponse {
                attributes: vec![],
                delimiter: None,
                mailbox: Mailbox::new("INBOX"),
            };
            assert!(resp.delimiter.is_none());
        }
    }

    mod mailbox_attribute_tests {
        use super::*;

        #[test]
        fn parse_noselect() {
            assert_eq!(
                MailboxAttribute::parse("\\NoSelect"),
                MailboxAttribute::NoSelect
            );
            assert_eq!(
                MailboxAttribute::parse("\\NOSELECT"),
                MailboxAttribute::NoSelect
            );
        }

        #[test]
        fn parse_has_no_children() {
            assert_eq!(
                MailboxAttribute::parse("\\HasNoChildren"),
                MailboxAttribute::HasNoChildren
            );
        }

        #[test]
        fn parse_has_children() {
            assert_eq!(
                MailboxAttribute::parse("\\HasChildren"),
                MailboxAttribute::HasChildren
            );
        }

        #[test]
        fn parse_marked() {
            assert_eq!(
                MailboxAttribute::parse("\\Marked"),
                MailboxAttribute::Marked
            );
        }

        #[test]
        fn parse_unmarked() {
            assert_eq!(
                MailboxAttribute::parse("\\Unmarked"),
                MailboxAttribute::Unmarked
            );
        }

        #[test]
        fn parse_all() {
            assert_eq!(MailboxAttribute::parse("\\All"), MailboxAttribute::All);
        }

        #[test]
        fn parse_archive() {
            assert_eq!(
                MailboxAttribute::parse("\\Archive"),
                MailboxAttribute::Archive
            );
        }

        #[test]
        fn parse_drafts() {
            assert_eq!(
                MailboxAttribute::parse("\\Drafts"),
                MailboxAttribute::Drafts
            );
        }

        #[test]
        fn parse_flagged() {
            assert_eq!(
                MailboxAttribute::parse("\\Flagged"),
                MailboxAttribute::Flagged
            );
        }

        #[test]
        fn parse_junk() {
            assert_eq!(MailboxAttribute::parse("\\Junk"), MailboxAttribute::Junk);
        }

        #[test]
        fn parse_spam() {
            assert_eq!(MailboxAttribute::parse("\\Spam"), MailboxAttribute::Junk);
        }

        #[test]
        fn parse_sent() {
            assert_eq!(MailboxAttribute::parse("\\Sent"), MailboxAttribute::Sent);
        }

        #[test]
        fn parse_trash() {
            assert_eq!(MailboxAttribute::parse("\\Trash"), MailboxAttribute::Trash);
        }

        #[test]
        fn parse_important() {
            assert_eq!(
                MailboxAttribute::parse("\\Important"),
                MailboxAttribute::Important
            );
        }

        #[test]
        fn parse_subscribed() {
            assert_eq!(
                MailboxAttribute::parse("\\Subscribed"),
                MailboxAttribute::Subscribed
            );
        }

        #[test]
        fn parse_unknown() {
            let attr = MailboxAttribute::parse("\\Custom");
            assert_eq!(attr, MailboxAttribute::Unknown("\\Custom".to_string()));
        }
    }
}
