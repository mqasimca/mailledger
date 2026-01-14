//! Core IMAP types.
//!
//! This module defines the fundamental types used throughout the IMAP library,
//! following RFC 9051 (`IMAP4rev2`) and RFC 3501 (`IMAP4rev1`).

#![allow(clippy::missing_const_for_fn)]

mod capability;
mod flags;
mod identifiers;
mod mailbox;
mod response_code;
mod sequence;

pub use capability::{Capability, Status};
pub use flags::{Flag, Flags};
pub use identifiers::{SeqNum, Tag, Uid, UidValidity};
pub use mailbox::{ListResponse, Mailbox, MailboxAttribute, MailboxStatus};
pub use response_code::ResponseCode;
pub use sequence::{SequenceSet, UidSet};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seq_num_new() {
        assert!(SeqNum::new(0).is_none());
        assert!(SeqNum::new(1).is_some());
        assert_eq!(SeqNum::new(42).unwrap().get(), 42);
    }

    #[test]
    fn test_uid_new() {
        assert!(Uid::new(0).is_none());
        assert!(Uid::new(1).is_some());
        assert_eq!(Uid::new(123).unwrap().get(), 123);
    }

    #[test]
    fn test_capability_parse() {
        assert_eq!(Capability::parse("IMAP4rev1"), Capability::Imap4Rev1);
        assert_eq!(Capability::parse("IMAP4REV2"), Capability::Imap4Rev2);
        assert_eq!(Capability::parse("idle"), Capability::Idle);
        assert_eq!(
            Capability::parse("AUTH=PLAIN"),
            Capability::Auth("PLAIN".to_string())
        );
        assert_eq!(
            Capability::parse("UNKNOWN"),
            Capability::Unknown("UNKNOWN".to_string())
        );
    }

    #[test]
    fn test_flag_parse() {
        assert_eq!(Flag::parse("\\Seen"), Flag::Seen);
        assert_eq!(Flag::parse("\\FLAGGED"), Flag::Flagged);
        assert_eq!(Flag::parse("custom"), Flag::Keyword("custom".to_string()));
    }

    #[test]
    fn test_flags_collection() {
        let mut flags = Flags::new();
        assert!(flags.is_empty());

        flags.insert(Flag::Seen);
        assert!(flags.is_seen());
        assert!(!flags.is_flagged());

        flags.insert(Flag::Flagged);
        assert!(flags.is_flagged());
        assert_eq!(flags.len(), 2);

        flags.remove(&Flag::Seen);
        assert!(!flags.is_seen());
    }

    #[test]
    fn test_sequence_set_display() {
        assert_eq!(SequenceSet::single(1).unwrap().to_string(), "1");
        assert_eq!(SequenceSet::range(1, 10).unwrap().to_string(), "1:10");
        assert_eq!(SequenceSet::All.to_string(), "*");
    }

    #[test]
    fn test_mailbox_attribute_parse() {
        assert_eq!(
            MailboxAttribute::parse("\\NoSelect"),
            MailboxAttribute::NoSelect
        );
        assert_eq!(
            MailboxAttribute::parse("\\HasChildren"),
            MailboxAttribute::HasChildren
        );
        assert_eq!(MailboxAttribute::parse("\\Trash"), MailboxAttribute::Trash);
    }
}
