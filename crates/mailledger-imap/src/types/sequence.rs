//! Sequence sets for message ranges.

use super::{SeqNum, Uid};

/// Sequence set for specifying message ranges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SequenceSet {
    /// Single sequence number.
    Single(SeqNum),
    /// Range of sequence numbers (inclusive).
    Range(SeqNum, SeqNum),
    /// Range from start to end of mailbox.
    RangeFrom(SeqNum),
    /// All messages (*).
    All,
    /// Multiple sequence specifications.
    Set(Vec<Self>),
}

impl SequenceSet {
    /// Creates a sequence set from a single number.
    #[must_use]
    pub fn single(n: u32) -> Option<Self> {
        SeqNum::new(n).map(Self::Single)
    }

    /// Creates a range sequence set.
    #[must_use]
    pub fn range(start: u32, end: u32) -> Option<Self> {
        Some(Self::Range(SeqNum::new(start)?, SeqNum::new(end)?))
    }
}

impl std::fmt::Display for SequenceSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(n) => write!(f, "{n}"),
            Self::Range(start, end) => write!(f, "{start}:{end}"),
            Self::RangeFrom(start) => write!(f, "{start}:*"),
            Self::All => write!(f, "*"),
            Self::Set(items) => {
                let s: Vec<_> = items.iter().map(ToString::to_string).collect();
                write!(f, "{}", s.join(","))
            }
        }
    }
}

/// UID-based sequence set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UidSet {
    /// Single UID.
    Single(Uid),
    /// Range of UIDs (inclusive).
    Range(Uid, Uid),
    /// Range from start to highest UID.
    RangeFrom(Uid),
    /// All messages.
    All,
    /// Multiple UID specifications.
    Set(Vec<Self>),
}

impl UidSet {
    /// Creates a UID set from a single UID.
    #[must_use]
    pub fn single(uid: Uid) -> Self {
        Self::Single(uid)
    }

    /// Creates a UID set from a range.
    #[must_use]
    pub fn range(start: Uid, end: Uid) -> Self {
        Self::Range(start, end)
    }

    /// Converts this UID set to a sequence set for use in UID commands.
    ///
    /// This is used internally for UID FETCH, UID STORE, etc. commands
    /// where the command serialization expects a `SequenceSet` but the
    /// actual values are UIDs.
    #[must_use]
    pub fn as_sequence_set(&self) -> SequenceSet {
        match self {
            // Both Uid and SeqNum wrap NonZeroU32, so this conversion is infallible
            Self::Single(uid) => SequenceSet::Single(SeqNum(uid.0)),
            Self::Range(start, end) => SequenceSet::Range(SeqNum(start.0), SeqNum(end.0)),
            Self::RangeFrom(start) => SequenceSet::RangeFrom(SeqNum(start.0)),
            Self::All => SequenceSet::All,
            Self::Set(items) => SequenceSet::Set(items.iter().map(Self::as_sequence_set).collect()),
        }
    }
}

impl std::fmt::Display for UidSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(n) => write!(f, "{n}"),
            Self::Range(start, end) => write!(f, "{start}:{end}"),
            Self::RangeFrom(start) => write!(f, "{start}:*"),
            Self::All => write!(f, "*"),
            Self::Set(items) => {
                let s: Vec<_> = items.iter().map(ToString::to_string).collect();
                write!(f, "{}", s.join(","))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sequence_set_tests {
        use super::*;

        #[test]
        fn single_valid() {
            let seq = SequenceSet::single(5);
            assert!(seq.is_some());
            if let SequenceSet::Single(n) = seq.unwrap() {
                assert_eq!(n.get(), 5);
            } else {
                panic!("Expected Single variant");
            }
        }

        #[test]
        fn single_zero_returns_none() {
            let seq = SequenceSet::single(0);
            assert!(seq.is_none());
        }

        #[test]
        fn range_valid() {
            let seq = SequenceSet::range(1, 10);
            assert!(seq.is_some());
            if let SequenceSet::Range(start, end) = seq.unwrap() {
                assert_eq!(start.get(), 1);
                assert_eq!(end.get(), 10);
            } else {
                panic!("Expected Range variant");
            }
        }

        #[test]
        fn range_zero_start_returns_none() {
            let seq = SequenceSet::range(0, 10);
            assert!(seq.is_none());
        }

        #[test]
        fn range_zero_end_returns_none() {
            let seq = SequenceSet::range(1, 0);
            assert!(seq.is_none());
        }

        #[test]
        fn display_single() {
            let seq = SequenceSet::single(42).unwrap();
            assert_eq!(format!("{seq}"), "42");
        }

        #[test]
        fn display_range() {
            let seq = SequenceSet::range(1, 100).unwrap();
            assert_eq!(format!("{seq}"), "1:100");
        }

        #[test]
        fn display_range_from() {
            let start = SeqNum::new(50).unwrap();
            let seq = SequenceSet::RangeFrom(start);
            assert_eq!(format!("{seq}"), "50:*");
        }

        #[test]
        fn display_all() {
            let seq = SequenceSet::All;
            assert_eq!(format!("{seq}"), "*");
        }

        #[test]
        fn display_set() {
            let seq = SequenceSet::Set(vec![
                SequenceSet::single(1).unwrap(),
                SequenceSet::range(5, 10).unwrap(),
            ]);
            assert_eq!(format!("{seq}"), "1,5:10");
        }
    }

    mod uid_set_tests {
        use super::*;

        #[test]
        fn single() {
            let uid = Uid::new(100).unwrap();
            let set = UidSet::single(uid);
            if let UidSet::Single(u) = set {
                assert_eq!(u.get(), 100);
            } else {
                panic!("Expected Single variant");
            }
        }

        #[test]
        fn range() {
            let start = Uid::new(1).unwrap();
            let end = Uid::new(50).unwrap();
            let set = UidSet::range(start, end);
            if let UidSet::Range(s, e) = set {
                assert_eq!(s.get(), 1);
                assert_eq!(e.get(), 50);
            } else {
                panic!("Expected Range variant");
            }
        }

        #[test]
        fn display_single() {
            let uid = Uid::new(123).unwrap();
            let set = UidSet::single(uid);
            assert_eq!(format!("{set}"), "123");
        }

        #[test]
        fn display_range() {
            let start = Uid::new(1).unwrap();
            let end = Uid::new(999).unwrap();
            let set = UidSet::range(start, end);
            assert_eq!(format!("{set}"), "1:999");
        }

        #[test]
        fn display_range_from() {
            let start = Uid::new(100).unwrap();
            let set = UidSet::RangeFrom(start);
            assert_eq!(format!("{set}"), "100:*");
        }

        #[test]
        fn display_all() {
            let set = UidSet::All;
            assert_eq!(format!("{set}"), "*");
        }

        #[test]
        fn as_sequence_set_single() {
            let uid = Uid::new(42).unwrap();
            let set = UidSet::single(uid);
            let seq = set.as_sequence_set();
            if let SequenceSet::Single(n) = seq {
                assert_eq!(n.get(), 42);
            } else {
                panic!("Expected Single variant");
            }
        }

        #[test]
        fn as_sequence_set_all() {
            let set = UidSet::All;
            let seq = set.as_sequence_set();
            assert!(matches!(seq, SequenceSet::All));
        }
    }
}
