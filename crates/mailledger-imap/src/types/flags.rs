//! Message flags.

/// Message flags.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Flag {
    /// Message has been read.
    Seen,
    /// Message has been answered.
    Answered,
    /// Message is flagged for special attention.
    Flagged,
    /// Message is marked for deletion.
    Deleted,
    /// Message is a draft.
    Draft,
    /// Message is recent (first session to see it).
    Recent,
    /// Custom keyword flag.
    Keyword(String),
}

impl Flag {
    /// Parses a flag string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "\\SEEN" => Self::Seen,
            "\\ANSWERED" => Self::Answered,
            "\\FLAGGED" => Self::Flagged,
            "\\DELETED" => Self::Deleted,
            "\\DRAFT" => Self::Draft,
            "\\RECENT" => Self::Recent,
            _ => Self::Keyword(s.to_string()),
        }
    }

    /// Returns the flag as an IMAP string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Seen => "\\Seen",
            Self::Answered => "\\Answered",
            Self::Flagged => "\\Flagged",
            Self::Deleted => "\\Deleted",
            Self::Draft => "\\Draft",
            Self::Recent => "\\Recent",
            Self::Keyword(s) => s,
        }
    }
}

impl std::fmt::Display for Flag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Collection of message flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Flags {
    flags: Vec<Flag>,
}

impl Flags {
    /// Creates an empty flags collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates flags from a vector.
    #[must_use]
    pub fn from_vec(flags: Vec<Flag>) -> Self {
        Self { flags }
    }

    /// Adds a flag.
    pub fn insert(&mut self, flag: Flag) {
        if !self.flags.contains(&flag) {
            self.flags.push(flag);
        }
    }

    /// Removes a flag.
    pub fn remove(&mut self, flag: &Flag) {
        self.flags.retain(|f| f != flag);
    }

    /// Returns true if the flag is present.
    #[must_use]
    pub fn contains(&self, flag: &Flag) -> bool {
        self.flags.contains(flag)
    }

    /// Returns true if the message has been seen.
    #[must_use]
    pub fn is_seen(&self) -> bool {
        self.contains(&Flag::Seen)
    }

    /// Returns true if the message has been answered.
    #[must_use]
    pub fn is_answered(&self) -> bool {
        self.contains(&Flag::Answered)
    }

    /// Returns true if the message is flagged.
    #[must_use]
    pub fn is_flagged(&self) -> bool {
        self.contains(&Flag::Flagged)
    }

    /// Returns true if the message is marked for deletion.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.contains(&Flag::Deleted)
    }

    /// Returns true if the message is a draft.
    #[must_use]
    pub fn is_draft(&self) -> bool {
        self.contains(&Flag::Draft)
    }

    /// Returns an iterator over the flags.
    pub fn iter(&self) -> impl Iterator<Item = &Flag> {
        self.flags.iter()
    }

    /// Returns the number of flags.
    #[must_use]
    pub fn len(&self) -> usize {
        self.flags.len()
    }

    /// Returns true if there are no flags.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }
}

impl IntoIterator for Flags {
    type Item = Flag;
    type IntoIter = std::vec::IntoIter<Flag>;

    fn into_iter(self) -> Self::IntoIter {
        self.flags.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod flag_tests {
        use super::*;

        #[test]
        fn parse_seen() {
            assert_eq!(Flag::parse("\\Seen"), Flag::Seen);
            assert_eq!(Flag::parse("\\SEEN"), Flag::Seen);
            assert_eq!(Flag::parse("\\seen"), Flag::Seen);
        }

        #[test]
        fn parse_answered() {
            assert_eq!(Flag::parse("\\Answered"), Flag::Answered);
        }

        #[test]
        fn parse_flagged() {
            assert_eq!(Flag::parse("\\Flagged"), Flag::Flagged);
        }

        #[test]
        fn parse_deleted() {
            assert_eq!(Flag::parse("\\Deleted"), Flag::Deleted);
        }

        #[test]
        fn parse_draft() {
            assert_eq!(Flag::parse("\\Draft"), Flag::Draft);
        }

        #[test]
        fn parse_recent() {
            assert_eq!(Flag::parse("\\Recent"), Flag::Recent);
        }

        #[test]
        fn parse_keyword() {
            let flag = Flag::parse("$Important");
            assert_eq!(flag, Flag::Keyword("$Important".to_string()));
        }

        #[test]
        fn as_str() {
            assert_eq!(Flag::Seen.as_str(), "\\Seen");
            assert_eq!(Flag::Answered.as_str(), "\\Answered");
            assert_eq!(Flag::Flagged.as_str(), "\\Flagged");
            assert_eq!(Flag::Deleted.as_str(), "\\Deleted");
            assert_eq!(Flag::Draft.as_str(), "\\Draft");
            assert_eq!(Flag::Recent.as_str(), "\\Recent");
            assert_eq!(Flag::Keyword("Custom".to_string()).as_str(), "Custom");
        }

        #[test]
        fn display() {
            assert_eq!(format!("{}", Flag::Seen), "\\Seen");
            assert_eq!(format!("{}", Flag::Keyword("Test".to_string())), "Test");
        }
    }

    mod flags_tests {
        use super::*;

        #[test]
        fn new_creates_empty() {
            let flags = Flags::new();
            assert!(flags.is_empty());
            assert_eq!(flags.len(), 0);
        }

        #[test]
        fn from_vec() {
            let flags = Flags::from_vec(vec![Flag::Seen, Flag::Answered]);
            assert_eq!(flags.len(), 2);
            assert!(flags.contains(&Flag::Seen));
            assert!(flags.contains(&Flag::Answered));
        }

        #[test]
        fn insert_unique() {
            let mut flags = Flags::new();
            flags.insert(Flag::Seen);
            flags.insert(Flag::Seen); // duplicate
            assert_eq!(flags.len(), 1);
        }

        #[test]
        fn remove() {
            let mut flags = Flags::from_vec(vec![Flag::Seen, Flag::Answered]);
            flags.remove(&Flag::Seen);
            assert!(!flags.contains(&Flag::Seen));
            assert!(flags.contains(&Flag::Answered));
        }

        #[test]
        fn is_seen() {
            let flags = Flags::from_vec(vec![Flag::Seen]);
            assert!(flags.is_seen());
            assert!(!flags.is_answered());
        }

        #[test]
        fn is_answered() {
            let flags = Flags::from_vec(vec![Flag::Answered]);
            assert!(flags.is_answered());
        }

        #[test]
        fn is_flagged() {
            let flags = Flags::from_vec(vec![Flag::Flagged]);
            assert!(flags.is_flagged());
        }

        #[test]
        fn is_deleted() {
            let flags = Flags::from_vec(vec![Flag::Deleted]);
            assert!(flags.is_deleted());
        }

        #[test]
        fn is_draft() {
            let flags = Flags::from_vec(vec![Flag::Draft]);
            assert!(flags.is_draft());
        }

        #[test]
        fn iter() {
            let flags = Flags::from_vec(vec![Flag::Seen, Flag::Answered]);
            let collected: Vec<_> = flags.iter().collect();
            assert_eq!(collected.len(), 2);
        }

        #[test]
        fn into_iter() {
            let flags = Flags::from_vec(vec![Flag::Seen, Flag::Flagged]);
            let collected: Vec<_> = flags.into_iter().collect();
            assert_eq!(collected.len(), 2);
        }

        #[test]
        fn default() {
            let flags = Flags::default();
            assert!(flags.is_empty());
        }
    }
}
