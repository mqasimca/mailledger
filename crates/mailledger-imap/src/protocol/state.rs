//! Protocol state types.
//!
//! This module defines the states that the IMAP protocol can be in,
//! following RFC 9051 section 3.

/// Protocol state as defined by RFC 9051.
///
/// The IMAP protocol has four states:
/// - `NotAuthenticated`: Initial state, only authentication commands allowed
/// - `Authenticated`: User is authenticated, can select mailboxes
/// - `Selected`: A mailbox is selected, can manipulate messages
/// - `Logout`: Connection is being closed
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ProtocolState {
    /// Not authenticated - waiting for credentials.
    ///
    /// In this state, only these commands are valid:
    /// - CAPABILITY
    /// - NOOP
    /// - LOGOUT
    /// - STARTTLS (if available)
    /// - AUTHENTICATE
    /// - LOGIN
    #[default]
    NotAuthenticated,

    /// Authenticated - user has logged in.
    ///
    /// In this state, these additional commands are valid:
    /// - SELECT
    /// - EXAMINE
    /// - CREATE
    /// - DELETE
    /// - RENAME
    /// - SUBSCRIBE
    /// - UNSUBSCRIBE
    /// - LIST
    /// - LSUB
    /// - STATUS
    /// - APPEND
    Authenticated,

    /// Selected - a mailbox is currently open.
    ///
    /// In this state, all commands are valid plus:
    /// - CHECK
    /// - CLOSE
    /// - EXPUNGE
    /// - SEARCH
    /// - FETCH
    /// - STORE
    /// - COPY
    /// - MOVE
    /// - UID (prefix)
    Selected(SelectedState),

    /// Logout - connection is being closed.
    ///
    /// After receiving BYE, no more commands can be sent.
    Logout,
}

impl ProtocolState {
    /// Returns `true` if we're authenticated (authenticated or selected).
    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        matches!(self, Self::Authenticated | Self::Selected(_))
    }

    /// Returns `true` if a mailbox is selected.
    #[must_use]
    pub const fn is_selected(&self) -> bool {
        matches!(self, Self::Selected(_))
    }

    /// Returns the selected mailbox name, if any.
    #[must_use]
    pub fn selected_mailbox(&self) -> Option<&str> {
        match self {
            Self::Selected(state) => Some(&state.mailbox),
            _ => None,
        }
    }

    /// Returns `true` if the selected mailbox is read-only.
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        match self {
            Self::Selected(state) => state.read_only,
            _ => false,
        }
    }
}

/// State information when a mailbox is selected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedState {
    /// Name of the selected mailbox.
    pub mailbox: String,
    /// Whether the mailbox is read-only (EXAMINE vs SELECT).
    pub read_only: bool,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_state_default() {
        assert_eq!(ProtocolState::default(), ProtocolState::NotAuthenticated);
    }

    #[test]
    fn test_is_authenticated() {
        assert!(!ProtocolState::NotAuthenticated.is_authenticated());
        assert!(ProtocolState::Authenticated.is_authenticated());
        assert!(
            ProtocolState::Selected(SelectedState {
                mailbox: "INBOX".to_string(),
                read_only: false,
            })
            .is_authenticated()
        );
        assert!(!ProtocolState::Logout.is_authenticated());
    }

    #[test]
    fn test_is_selected() {
        assert!(!ProtocolState::NotAuthenticated.is_selected());
        assert!(!ProtocolState::Authenticated.is_selected());
        assert!(
            ProtocolState::Selected(SelectedState {
                mailbox: "INBOX".to_string(),
                read_only: false,
            })
            .is_selected()
        );
    }

    #[test]
    fn test_selected_mailbox() {
        assert_eq!(ProtocolState::NotAuthenticated.selected_mailbox(), None);
        assert_eq!(
            ProtocolState::Selected(SelectedState {
                mailbox: "Drafts".to_string(),
                read_only: true,
            })
            .selected_mailbox(),
            Some("Drafts")
        );
    }

    #[test]
    fn test_is_read_only() {
        assert!(!ProtocolState::Authenticated.is_read_only());
        assert!(
            !ProtocolState::Selected(SelectedState {
                mailbox: "INBOX".to_string(),
                read_only: false,
            })
            .is_read_only()
        );
        assert!(
            ProtocolState::Selected(SelectedState {
                mailbox: "INBOX".to_string(),
                read_only: true,
            })
            .is_read_only()
        );
    }
}
