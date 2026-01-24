//! Response codes.

use super::{Capability, Flag, SeqNum, Uid, UidValidity};

/// Response code from a tagged response.
///
/// These provide additional information about command completion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseCode {
    /// ALERT: Human-readable message that MUST be shown to user.
    Alert,
    /// CAPABILITY response.
    Capability(Vec<Capability>),
    /// PARSE: Error parsing message.
    Parse,
    /// PERMANENTFLAGS: Flags that can be changed permanently.
    PermanentFlags(Vec<Flag>),
    /// READ-ONLY: Mailbox selected as read-only.
    ReadOnly,
    /// READ-WRITE: Mailbox selected as read-write.
    ReadWrite,
    /// TRYCREATE: Mailbox doesn't exist, but can be created.
    TryCreate,
    /// UIDNEXT: Next UID to be assigned.
    UidNext(Uid),
    /// UIDVALIDITY: Unique identifier validity value.
    UidValidity(UidValidity),
    /// UNSEEN: First unseen message sequence number.
    Unseen(SeqNum),
    /// APPENDUID: UID assigned to appended message.
    AppendUid {
        /// UIDVALIDITY of the mailbox.
        uidvalidity: UidValidity,
        /// UID of the appended message.
        uid: Uid,
    },
    /// COPYUID: UIDs of copied messages.
    CopyUid {
        /// UIDVALIDITY of the destination mailbox.
        uidvalidity: UidValidity,
        /// Source UIDs.
        source_uids: Vec<Uid>,
        /// Destination UIDs.
        dest_uids: Vec<Uid>,
    },
    /// HIGHESTMODSEQ: Highest mod-sequence value (CONDSTORE).
    HighestModSeq(u64),
    /// NOMODSEQ: Server doesn't support mod-sequences for this mailbox.
    NoModSeq,
    /// Unknown response code.
    Unknown(String),
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
    fn alert() {
        let code = ResponseCode::Alert;
        assert!(matches!(code, ResponseCode::Alert));
    }

    #[test]
    fn capability_with_list() {
        let caps = vec![Capability::Imap4Rev1, Capability::Idle];
        let code = ResponseCode::Capability(caps.clone());
        if let ResponseCode::Capability(c) = code {
            assert_eq!(c.len(), 2);
        } else {
            panic!("Expected Capability variant");
        }
    }

    #[test]
    fn parse() {
        let code = ResponseCode::Parse;
        assert!(matches!(code, ResponseCode::Parse));
    }

    #[test]
    fn permanent_flags() {
        let flags = vec![Flag::Seen, Flag::Answered, Flag::Deleted];
        let code = ResponseCode::PermanentFlags(flags.clone());
        if let ResponseCode::PermanentFlags(f) = code {
            assert_eq!(f.len(), 3);
        } else {
            panic!("Expected PermanentFlags variant");
        }
    }

    #[test]
    fn read_only() {
        let code = ResponseCode::ReadOnly;
        assert!(matches!(code, ResponseCode::ReadOnly));
    }

    #[test]
    fn read_write() {
        let code = ResponseCode::ReadWrite;
        assert!(matches!(code, ResponseCode::ReadWrite));
    }

    #[test]
    fn try_create() {
        let code = ResponseCode::TryCreate;
        assert!(matches!(code, ResponseCode::TryCreate));
    }

    #[test]
    fn uid_next() {
        let uid = Uid::new(100).unwrap();
        let code = ResponseCode::UidNext(uid);
        if let ResponseCode::UidNext(u) = code {
            assert_eq!(u.get(), 100);
        } else {
            panic!("Expected UidNext variant");
        }
    }

    #[test]
    fn uid_validity() {
        let uv = UidValidity::new(123456).unwrap();
        let code = ResponseCode::UidValidity(uv);
        if let ResponseCode::UidValidity(v) = code {
            assert_eq!(v.get(), 123456);
        } else {
            panic!("Expected UidValidity variant");
        }
    }

    #[test]
    fn unseen() {
        let seq = SeqNum::new(42).unwrap();
        let code = ResponseCode::Unseen(seq);
        if let ResponseCode::Unseen(s) = code {
            assert_eq!(s.get(), 42);
        } else {
            panic!("Expected Unseen variant");
        }
    }

    #[test]
    fn append_uid() {
        let uv = UidValidity::new(999).unwrap();
        let uid = Uid::new(50).unwrap();
        let code = ResponseCode::AppendUid {
            uidvalidity: uv,
            uid,
        };
        if let ResponseCode::AppendUid { uidvalidity, uid } = code {
            assert_eq!(uidvalidity.get(), 999);
            assert_eq!(uid.get(), 50);
        } else {
            panic!("Expected AppendUid variant");
        }
    }

    #[test]
    fn copy_uid() {
        let uv = UidValidity::new(888).unwrap();
        let src = vec![Uid::new(1).unwrap(), Uid::new(2).unwrap()];
        let dst = vec![Uid::new(101).unwrap(), Uid::new(102).unwrap()];
        let code = ResponseCode::CopyUid {
            uidvalidity: uv,
            source_uids: src,
            dest_uids: dst,
        };
        if let ResponseCode::CopyUid {
            uidvalidity,
            source_uids,
            dest_uids,
        } = code
        {
            assert_eq!(uidvalidity.get(), 888);
            assert_eq!(source_uids.len(), 2);
            assert_eq!(dest_uids.len(), 2);
        } else {
            panic!("Expected CopyUid variant");
        }
    }

    #[test]
    fn highest_mod_seq() {
        let code = ResponseCode::HighestModSeq(987654321);
        if let ResponseCode::HighestModSeq(seq) = code {
            assert_eq!(seq, 987654321);
        } else {
            panic!("Expected HighestModSeq variant");
        }
    }

    #[test]
    fn no_mod_seq() {
        let code = ResponseCode::NoModSeq;
        assert!(matches!(code, ResponseCode::NoModSeq));
    }

    #[test]
    fn unknown() {
        let code = ResponseCode::Unknown("CUSTOM-CODE".to_string());
        if let ResponseCode::Unknown(s) = code {
            assert_eq!(s, "CUSTOM-CODE");
        } else {
            panic!("Expected Unknown variant");
        }
    }
}
