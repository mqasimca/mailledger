//! Core IMAP identifiers.
//!
//! Types for tags, sequence numbers, UIDs, and UIDVALIDITY.

use std::num::NonZeroU32;

/// IMAP command tag.
///
/// Tags are alphanumeric prefixes that identify commands and their responses.
/// Each command sent by the client has a unique tag, and the server's response
/// includes the same tag to correlate request and response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag(pub String);

impl Tag {
    /// Creates a new tag from a string.
    #[must_use]
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    /// Returns the tag as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message sequence number.
///
/// Sequence numbers are assigned to messages in a mailbox starting from 1.
/// They are ephemeral and change when messages are expunged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SeqNum(pub NonZeroU32);

impl SeqNum {
    /// Creates a new sequence number.
    ///
    /// Returns `None` if the value is 0.
    #[must_use]
    pub fn new(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(Self)
    }

    /// Returns the underlying value.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl std::fmt::Display for SeqNum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a message.
///
/// UIDs are persistent identifiers that don't change when messages are expunged.
/// Combined with `UIDVALIDITY`, they uniquely identify a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Uid(pub NonZeroU32);

impl Uid {
    /// Creates a new UID.
    ///
    /// Returns `None` if the value is 0.
    #[must_use]
    pub fn new(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(Self)
    }

    /// Returns the underlying value.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
    }
}

impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// UIDVALIDITY value for a mailbox.
///
/// If this value changes, all cached UIDs are invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UidValidity(pub NonZeroU32);

impl UidValidity {
    /// Creates a new UIDVALIDITY.
    #[must_use]
    pub fn new(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(Self)
    }

    /// Returns the underlying value.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0.get()
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

    mod tag_tests {
        use super::*;

        #[test]
        fn new_from_string() {
            let tag = Tag::new("A001".to_string());
            assert_eq!(tag.as_str(), "A001");
        }

        #[test]
        fn new_from_str() {
            let tag = Tag::new("B002");
            assert_eq!(tag.as_str(), "B002");
        }

        #[test]
        fn display() {
            let tag = Tag::new("CMD123");
            assert_eq!(format!("{tag}"), "CMD123");
        }

        #[test]
        fn equality() {
            let tag1 = Tag::new("A001");
            let tag2 = Tag::new("A001");
            let tag3 = Tag::new("A002");
            assert_eq!(tag1, tag2);
            assert_ne!(tag1, tag3);
        }
    }

    mod seq_num_tests {
        use super::*;

        #[test]
        fn new_valid() {
            let seq = SeqNum::new(1);
            assert!(seq.is_some());
            assert_eq!(seq.unwrap().get(), 1);
        }

        #[test]
        fn new_zero_returns_none() {
            let seq = SeqNum::new(0);
            assert!(seq.is_none());
        }

        #[test]
        fn new_large_value() {
            let seq = SeqNum::new(u32::MAX);
            assert!(seq.is_some());
            assert_eq!(seq.unwrap().get(), u32::MAX);
        }

        #[test]
        fn display() {
            let seq = SeqNum::new(42).unwrap();
            assert_eq!(format!("{seq}"), "42");
        }

        #[test]
        fn ordering() {
            let seq1 = SeqNum::new(1).unwrap();
            let seq2 = SeqNum::new(2).unwrap();
            assert!(seq1 < seq2);
        }
    }

    mod uid_tests {
        use super::*;

        #[test]
        fn new_valid() {
            let uid = Uid::new(100);
            assert!(uid.is_some());
            assert_eq!(uid.unwrap().get(), 100);
        }

        #[test]
        fn new_zero_returns_none() {
            let uid = Uid::new(0);
            assert!(uid.is_none());
        }

        #[test]
        fn display() {
            let uid = Uid::new(12345).unwrap();
            assert_eq!(format!("{uid}"), "12345");
        }

        #[test]
        fn ordering() {
            let uid1 = Uid::new(100).unwrap();
            let uid2 = Uid::new(200).unwrap();
            assert!(uid1 < uid2);
        }
    }

    mod uid_validity_tests {
        use super::*;

        #[test]
        fn new_valid() {
            let uv = UidValidity::new(987654321);
            assert!(uv.is_some());
            assert_eq!(uv.unwrap().get(), 987654321);
        }

        #[test]
        fn new_zero_returns_none() {
            let uv = UidValidity::new(0);
            assert!(uv.is_none());
        }

        #[test]
        fn equality() {
            let uv1 = UidValidity::new(123).unwrap();
            let uv2 = UidValidity::new(123).unwrap();
            let uv3 = UidValidity::new(456).unwrap();
            assert_eq!(uv1, uv2);
            assert_ne!(uv1, uv3);
        }
    }
}
