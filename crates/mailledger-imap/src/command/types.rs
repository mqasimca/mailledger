//! Command-related type definitions.

use crate::types::{Flag, SequenceSet, UidSet};

/// STATUS attributes to request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusAttribute {
    /// Number of messages.
    Messages,
    /// Number of recent messages.
    Recent,
    /// Next UID.
    UidNext,
    /// UIDVALIDITY.
    UidValidity,
    /// Number of unseen messages.
    Unseen,
    /// Highest mod-sequence.
    HighestModSeq,
}

impl StatusAttribute {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Messages => "MESSAGES",
            Self::Recent => "RECENT",
            Self::UidNext => "UIDNEXT",
            Self::UidValidity => "UIDVALIDITY",
            Self::Unseen => "UNSEEN",
            Self::HighestModSeq => "HIGHESTMODSEQ",
        }
    }
}

/// FETCH items to request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchItems {
    /// Fetch all (equivalent to FLAGS INTERNALDATE RFC822.SIZE ENVELOPE).
    All,
    /// Fetch full (equivalent to FLAGS INTERNALDATE RFC822.SIZE ENVELOPE BODY).
    Full,
    /// Fetch fast (equivalent to FLAGS INTERNALDATE RFC822.SIZE).
    Fast,
    /// Custom list of items.
    Items(Vec<FetchAttribute>),
}

/// Individual FETCH attribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchAttribute {
    /// Message flags.
    Flags,
    /// Internal date.
    InternalDate,
    /// RFC822 size.
    Rfc822Size,
    /// Envelope structure.
    Envelope,
    /// Body structure.
    BodyStructure,
    /// UID.
    Uid,
    /// Body section.
    Body {
        /// Section specifier.
        section: Option<String>,
        /// Peek (don't set \Seen).
        peek: bool,
        /// Partial fetch range.
        partial: Option<(u32, u32)>,
    },
    /// RFC822 (full message).
    Rfc822,
    /// RFC822.HEADER.
    Rfc822Header,
    /// RFC822.TEXT.
    Rfc822Text,
    /// MODSEQ.
    ModSeq,
}

/// STORE action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreAction {
    /// Replace flags.
    SetFlags(Vec<Flag>),
    /// Add flags.
    AddFlags(Vec<Flag>),
    /// Remove flags.
    RemoveFlags(Vec<Flag>),
    /// Replace flags only if unchanged since mod-sequence (CONDSTORE).
    SetFlagsUnchangedSince {
        /// Flags to set.
        flags: Vec<Flag>,
        /// Mod-sequence value.
        modseq: u64,
    },
    /// Add flags only if unchanged since mod-sequence (CONDSTORE).
    AddFlagsUnchangedSince {
        /// Flags to add.
        flags: Vec<Flag>,
        /// Mod-sequence value.
        modseq: u64,
    },
    /// Remove flags only if unchanged since mod-sequence (CONDSTORE).
    RemoveFlagsUnchangedSince {
        /// Flags to remove.
        flags: Vec<Flag>,
        /// Mod-sequence value.
        modseq: u64,
    },
}

/// SEARCH criteria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchCriteria {
    /// All messages.
    All,
    /// Messages with \Answered flag.
    Answered,
    /// Messages with \Deleted flag.
    Deleted,
    /// Messages with \Draft flag.
    Draft,
    /// Messages with \Flagged flag.
    Flagged,
    /// Messages without \Seen flag.
    New,
    /// Messages without \Deleted flag.
    Undeleted,
    /// Messages without \Seen flag.
    Unseen,
    /// Messages with \Seen flag.
    Seen,
    /// Sequence number set.
    SequenceSet(SequenceSet),
    /// UID set.
    UidSet(UidSet),
    /// Subject contains text.
    Subject(String),
    /// From contains text.
    From(String),
    /// To contains text.
    To(String),
    /// Body contains text.
    Body(String),
    /// Text in header or body.
    Text(String),
    /// Messages since date.
    Since(String),
    /// Messages before date.
    Before(String),
    /// Messages on date.
    On(String),
    /// Larger than size.
    Larger(u32),
    /// Smaller than size.
    Smaller(u32),
    /// Header field contains value.
    Header(String, String),
    /// Messages with mod-sequence greater than value (CONDSTORE).
    ModSeq(u64),
    /// AND of criteria.
    And(Vec<Self>),
    /// OR of criteria.
    Or(Box<Self>, Box<Self>),
    /// NOT of criteria.
    Not(Box<Self>),
}
