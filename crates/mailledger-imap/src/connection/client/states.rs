//! Type-state markers for IMAP client connection states.
//!
//! These types are used with the type-state pattern to enforce valid IMAP
//! state transitions at compile time. Unlike simple marker types, `Selected`
//! carries runtime state about the currently selected mailbox.

use std::sync::Arc;

use crate::types::MailboxStatus;

/// Marker type for the not-authenticated state.
///
/// In this state, only authentication commands (LOGIN, AUTHENTICATE) are valid.
#[derive(Debug, Clone, Copy, Default)]
pub struct NotAuthenticated;

/// Marker type for the authenticated state.
///
/// In this state, mailbox operations (SELECT, EXAMINE, LIST, CREATE, etc.) are valid.
#[derive(Debug, Clone, Copy, Default)]
pub struct Authenticated;

/// State for a selected mailbox.
///
/// Unlike the marker types, this carries runtime information about the
/// currently selected mailbox. This design follows the `imap-next` pattern
/// of storing relevant state while maintaining type-safety.
#[derive(Debug, Clone)]
pub struct Selected {
    /// The selected mailbox name.
    pub(crate) mailbox: Arc<str>,
    /// Whether the mailbox was opened read-only (via EXAMINE).
    pub(crate) read_only: bool,
    /// Cached mailbox status from SELECT/EXAMINE response.
    pub(crate) status: MailboxStatus,
}

impl Selected {
    /// Creates a new Selected state.
    #[must_use]
    pub fn new(mailbox: impl Into<Arc<str>>, read_only: bool, status: MailboxStatus) -> Self {
        Self {
            mailbox: mailbox.into(),
            read_only,
            status,
        }
    }

    /// Returns the name of the selected mailbox.
    #[must_use]
    pub fn mailbox(&self) -> &str {
        &self.mailbox
    }

    /// Returns true if the mailbox was opened read-only (via EXAMINE).
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Returns the mailbox status snapshot from SELECT/EXAMINE.
    #[must_use]
    pub const fn status(&self) -> &MailboxStatus {
        &self.status
    }

    /// Returns the number of messages in the mailbox.
    #[must_use]
    pub const fn exists(&self) -> u32 {
        self.status.exists
    }

    /// Returns the number of recent messages.
    #[must_use]
    pub const fn recent(&self) -> u32 {
        self.status.recent
    }

    /// Returns the UID validity value.
    #[must_use]
    pub fn uid_validity(&self) -> Option<u32> {
        self.status.uid_validity.map(crate::types::UidValidity::get)
    }

    /// Returns the next UID value.
    #[must_use]
    pub fn uid_next(&self) -> Option<u32> {
        self.status.uid_next.map(crate::types::Uid::get)
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

    fn _assert_send<T: Send>() {}
    fn _assert_sync<T: Sync>() {}

    #[test]
    fn test_state_markers_are_send_sync() {
        _assert_send::<NotAuthenticated>();
        _assert_sync::<NotAuthenticated>();
        _assert_send::<Authenticated>();
        _assert_sync::<Authenticated>();
        _assert_send::<Selected>();
        _assert_sync::<Selected>();
    }

    #[test]
    fn test_selected_state_accessors() {
        use crate::types::{Uid, UidValidity};

        let status = MailboxStatus {
            exists: 100,
            recent: 5,
            uid_validity: UidValidity::new(12345),
            uid_next: Uid::new(200),
            ..Default::default()
        };
        let selected = Selected::new("INBOX", false, status);

        assert_eq!(selected.mailbox(), "INBOX");
        assert!(!selected.is_read_only());
        assert_eq!(selected.exists(), 100);
        assert_eq!(selected.recent(), 5);
        assert_eq!(selected.uid_validity(), Some(12345));
        assert_eq!(selected.uid_next(), Some(200));
    }

    #[test]
    fn test_selected_read_only() {
        let selected = Selected::new("Archive", true, MailboxStatus::default());
        assert!(selected.is_read_only());
    }
}
