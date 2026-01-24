//! QRESYNC and CONDSTORE support (RFC 7162).
//!
// Allow missing_const_for_fn since many can't be const due to Vec operations.
#![allow(clippy::missing_const_for_fn)]
//!
//! This module provides types and utilities for efficient mailbox synchronization
//! using the QRESYNC and CONDSTORE extensions.
//!
//! ## CONDSTORE (RFC 7162 Section 3)
//!
//! CONDSTORE provides modification sequence numbers (`MODSEQ`) that track
//! changes to messages. Each message has a `MODSEQ` value that increases
//! whenever the message's flags or other metadata change.
//!
//! ## QRESYNC (RFC 7162 Section 4)
//!
//! QRESYNC builds on CONDSTORE to enable efficient resynchronization after
//! reconnection. Instead of fetching all message flags, a client can:
//!
//! 1. Store the `UIDVALIDITY`, `HIGHESTMODSEQ`, and known UIDs
//! 2. On reconnect, use QRESYNC SELECT to get only changed/expunged messages
//!
//! # Example
//!
//! ```ignore
//! use mailledger_imap::qresync::{SyncState, QresyncParams};
//!
//! // Save state after initial sync
//! let state = SyncState {
//!     uidvalidity: mailbox.uidvalidity,
//!     highestmodseq: mailbox.highestmodseq.unwrap(),
//!     known_uids: mailbox.uids.clone(),
//! };
//! state.save("inbox_sync.json")?;
//!
//! // On reconnect, use saved state for QRESYNC
//! let loaded = SyncState::load("inbox_sync.json")?;
//! let params = QresyncParams::from_state(&loaded);
//! client.select_qresync("INBOX", params).await?;
//! ```

use std::num::NonZeroU64;

use crate::types::{SeqNum, Uid, UidSet, UidValidity};

/// Modification sequence number (MODSEQ).
///
/// Each message has a MODSEQ value that increases whenever the message's
/// metadata (flags, annotations, etc.) changes. The server also maintains
/// a HIGHESTMODSEQ for each mailbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModSeq(NonZeroU64);

impl ModSeq {
    /// Creates a new `ModSeq` from a non-zero u64.
    #[must_use]
    pub fn new(value: NonZeroU64) -> Self {
        Self(value)
    }

    /// Creates a new `ModSeq` from a u64, returning `None` if zero.
    #[must_use]
    pub fn from_u64(value: u64) -> Option<Self> {
        NonZeroU64::new(value).map(Self)
    }

    /// Returns the raw value.
    #[must_use]
    pub fn get(self) -> u64 {
        self.0.get()
    }
}

impl std::fmt::Display for ModSeq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u64> for ModSeq {
    type Error = &'static str;

    fn try_from(value: u64) -> std::result::Result<Self, Self::Error> {
        Self::from_u64(value).ok_or("ModSeq cannot be zero")
    }
}

/// State saved for QRESYNC resynchronization.
///
/// This structure contains all the information needed to efficiently
/// resynchronize a mailbox after reconnection.
#[derive(Debug, Clone)]
pub struct SyncState {
    /// The UIDVALIDITY value when state was captured.
    ///
    /// If this changes, the mailbox has been rebuilt and all cached
    /// data must be discarded.
    pub uidvalidity: UidValidity,

    /// The HIGHESTMODSEQ when state was captured.
    ///
    /// Used to request only changes since this point.
    pub highestmodseq: ModSeq,

    /// Known UIDs at the time state was captured.
    ///
    /// Optional but recommended - allows server to send VANISHED
    /// responses for expunged messages.
    pub known_uids: Option<UidSet>,
}

impl SyncState {
    /// Creates a new sync state.
    #[must_use]
    pub fn new(uidvalidity: UidValidity, highestmodseq: ModSeq) -> Self {
        Self {
            uidvalidity,
            highestmodseq,
            known_uids: None,
        }
    }

    /// Sets the known UIDs.
    #[must_use]
    pub fn with_known_uids(mut self, uids: UidSet) -> Self {
        self.known_uids = Some(uids);
        self
    }

    /// Creates QRESYNC parameters from this state.
    #[must_use]
    pub fn to_params(&self) -> QresyncParams {
        QresyncParams {
            uidvalidity: self.uidvalidity,
            modseq: self.highestmodseq,
            known_uids: self.known_uids.clone(),
            seq_match: None,
        }
    }
}

/// Parameters for QRESYNC SELECT/EXAMINE command.
///
/// These parameters are sent with SELECT or EXAMINE to enable
/// efficient resynchronization.
#[derive(Debug, Clone)]
pub struct QresyncParams {
    /// The last known UIDVALIDITY.
    pub uidvalidity: UidValidity,

    /// The last known MODSEQ (HIGHESTMODSEQ from previous session).
    pub modseq: ModSeq,

    /// Optional set of known UIDs.
    ///
    /// If provided, the server can send VANISHED responses for UIDs
    /// in this set that have been expunged.
    pub known_uids: Option<UidSet>,

    /// Optional sequence-to-UID mapping for known messages.
    ///
    /// This helps the server verify message positions haven't changed
    /// significantly since the last session.
    pub seq_match: Option<SeqUidMatch>,
}

impl QresyncParams {
    /// Creates minimal QRESYNC parameters.
    #[must_use]
    pub fn new(uidvalidity: UidValidity, modseq: ModSeq) -> Self {
        Self {
            uidvalidity,
            modseq,
            known_uids: None,
            seq_match: None,
        }
    }

    /// Adds known UIDs to the parameters.
    #[must_use]
    pub fn with_known_uids(mut self, uids: UidSet) -> Self {
        self.known_uids = Some(uids);
        self
    }

    /// Adds sequence-UID mapping to the parameters.
    #[must_use]
    pub fn with_seq_match(mut self, seq_match: SeqUidMatch) -> Self {
        self.seq_match = Some(seq_match);
        self
    }

    /// Serializes the QRESYNC parameters for the SELECT command.
    ///
    /// Returns a string like: `(QRESYNC (123456789 0 1:100))`
    #[must_use]
    pub fn serialize(&self) -> String {
        let mut parts = vec![
            format!("{}", self.uidvalidity.get()),
            format!("{}", self.modseq.get()),
        ];

        if let Some(ref uids) = self.known_uids {
            parts.push(uids.to_string());
        }

        if let Some(ref seq_match) = self.seq_match {
            parts.push(format!("({})", seq_match.serialize()));
        }

        format!("(QRESYNC ({}))", parts.join(" "))
    }
}

/// Sequence-to-UID mapping for QRESYNC.
///
/// This optional mapping helps the server verify that messages
/// haven't been renumbered significantly since the last session.
#[derive(Debug, Clone)]
pub struct SeqUidMatch {
    /// Sequence number ranges.
    pub seq_set: Vec<(SeqNum, SeqNum)>,
    /// Corresponding UID ranges.
    pub uid_set: Vec<(Uid, Uid)>,
}

impl SeqUidMatch {
    /// Creates a new sequence-UID mapping.
    #[must_use]
    pub fn new() -> Self {
        Self {
            seq_set: Vec::new(),
            uid_set: Vec::new(),
        }
    }

    /// Adds a mapping from sequence range to UID range.
    pub fn add(&mut self, seq_range: (SeqNum, SeqNum), uid_range: (Uid, Uid)) {
        self.seq_set.push(seq_range);
        self.uid_set.push(uid_range);
    }

    /// Serializes the mapping.
    #[must_use]
    pub fn serialize(&self) -> String {
        let seqs: Vec<String> = self
            .seq_set
            .iter()
            .map(|(start, end)| {
                if start == end {
                    format!("{}", start.get())
                } else {
                    format!("{}:{}", start.get(), end.get())
                }
            })
            .collect();

        let uids: Vec<String> = self
            .uid_set
            .iter()
            .map(|(start, end)| {
                if start == end {
                    format!("{}", start.get())
                } else {
                    format!("{}:{}", start.get(), end.get())
                }
            })
            .collect();

        format!("{} {}", seqs.join(","), uids.join(","))
    }
}

impl Default for SeqUidMatch {
    fn default() -> Self {
        Self::new()
    }
}

/// A VANISHED response from the server.
///
/// VANISHED responses are sent instead of individual EXPUNGE responses
/// when QRESYNC is enabled, listing all UIDs that have been expunged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VanishedResponse {
    /// Whether this is an EARLIER response (for historical expunges).
    pub earlier: bool,
    /// UIDs that have been expunged.
    pub uids: UidSet,
}

impl VanishedResponse {
    /// Creates a new VANISHED response.
    #[must_use]
    pub fn new(uids: UidSet, earlier: bool) -> Self {
        Self { earlier, uids }
    }
}

/// Changes detected during QRESYNC.
///
/// This structure collects all changes reported by the server during
/// a QRESYNC SELECT operation.
#[derive(Debug, Clone, Default)]
pub struct SyncChanges {
    /// Messages that have been expunged (VANISHED responses).
    pub vanished: Vec<Uid>,

    /// Messages with changed flags (FETCH responses).
    pub changed: Vec<ChangedMessage>,

    /// The new HIGHESTMODSEQ value.
    pub new_highestmodseq: Option<ModSeq>,
}

impl SyncChanges {
    /// Creates an empty changes structure.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if there are any changes.
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.vanished.is_empty() || !self.changed.is_empty()
    }

    /// Returns the number of total changes.
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.vanished.len() + self.changed.len()
    }
}

/// A message with changed metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedMessage {
    /// The message UID.
    pub uid: Uid,
    /// The new MODSEQ value.
    pub modseq: ModSeq,
    /// The current flags (if included in response).
    pub flags: Option<crate::types::Flags>,
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

    fn test_uidvalidity() -> UidValidity {
        UidValidity::new(123_456_789).unwrap()
    }

    fn test_modseq() -> ModSeq {
        ModSeq::from_u64(987_654_321).unwrap()
    }

    #[test]
    fn test_modseq_creation() {
        let modseq = ModSeq::from_u64(100);
        assert!(modseq.is_some());
        assert_eq!(modseq.unwrap().get(), 100);

        let zero = ModSeq::from_u64(0);
        assert!(zero.is_none());
    }

    #[test]
    fn test_modseq_try_from() {
        let result: std::result::Result<ModSeq, _> = 100u64.try_into();
        assert!(result.is_ok());

        let result: std::result::Result<ModSeq, _> = 0u64.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_state() {
        let state = SyncState::new(test_uidvalidity(), test_modseq());
        assert!(state.known_uids.is_none());

        let with_uids = state.clone().with_known_uids(UidSet::All);
        assert!(with_uids.known_uids.is_some());
    }

    #[test]
    fn test_qresync_params_serialize() {
        let params = QresyncParams::new(test_uidvalidity(), test_modseq());
        let serialized = params.serialize();
        assert!(serialized.contains("QRESYNC"));
        assert!(serialized.contains("123456789"));
        assert!(serialized.contains("987654321"));
    }

    #[test]
    fn test_sync_changes() {
        let mut changes = SyncChanges::new();
        assert!(!changes.has_changes());
        assert_eq!(changes.change_count(), 0);

        changes.vanished.push(Uid::new(1).unwrap());
        assert!(changes.has_changes());
        assert_eq!(changes.change_count(), 1);
    }

    #[test]
    fn test_vanished_response() {
        let response = VanishedResponse::new(UidSet::All, false);
        assert!(!response.earlier);

        let earlier = VanishedResponse::new(UidSet::All, true);
        assert!(earlier.earlier);
    }
}
