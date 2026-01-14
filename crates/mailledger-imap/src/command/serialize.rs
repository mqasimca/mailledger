//! Command serialization helpers.

use crate::types::Mailbox;

use super::types::{FetchAttribute, FetchItems, SearchCriteria, StoreAction};

/// Writes an astring (atom or quoted string).
pub fn write_astring(buf: &mut Vec<u8>, s: &str) {
    if s.is_empty() || s.bytes().any(needs_quoting) {
        buf.push(b'"');
        for b in s.bytes() {
            if b == b'"' || b == b'\\' {
                buf.push(b'\\');
            }
            buf.push(b);
        }
        buf.push(b'"');
    } else {
        buf.extend_from_slice(s.as_bytes());
    }
}

/// Writes a mailbox name.
pub fn write_mailbox(buf: &mut Vec<u8>, mailbox: &Mailbox) {
    write_astring(buf, mailbox.as_str());
}

/// Returns true if the byte needs quoting.
const fn needs_quoting(b: u8) -> bool {
    matches!(b, b' ' | b'"' | b'\\' | b'(' | b')' | b'{' | b'%' | b'*') || b < 0x20 || b == 0x7F
}

/// Writes FETCH items.
pub fn write_fetch_items(buf: &mut Vec<u8>, items: &FetchItems) {
    match items {
        FetchItems::All => buf.extend_from_slice(b"ALL"),
        FetchItems::Full => buf.extend_from_slice(b"FULL"),
        FetchItems::Fast => buf.extend_from_slice(b"FAST"),
        FetchItems::Items(attrs) => {
            if attrs.len() == 1 {
                write_fetch_attribute(buf, &attrs[0]);
            } else {
                buf.push(b'(');
                for (i, attr) in attrs.iter().enumerate() {
                    if i > 0 {
                        buf.push(b' ');
                    }
                    write_fetch_attribute(buf, attr);
                }
                buf.push(b')');
            }
        }
    }
}

/// Writes a single FETCH attribute.
pub fn write_fetch_attribute(buf: &mut Vec<u8>, attr: &FetchAttribute) {
    match attr {
        FetchAttribute::Flags => buf.extend_from_slice(b"FLAGS"),
        FetchAttribute::InternalDate => buf.extend_from_slice(b"INTERNALDATE"),
        FetchAttribute::Rfc822Size => buf.extend_from_slice(b"RFC822.SIZE"),
        FetchAttribute::Envelope => buf.extend_from_slice(b"ENVELOPE"),
        FetchAttribute::BodyStructure => buf.extend_from_slice(b"BODYSTRUCTURE"),
        FetchAttribute::Uid => buf.extend_from_slice(b"UID"),
        FetchAttribute::Rfc822 => buf.extend_from_slice(b"RFC822"),
        FetchAttribute::Rfc822Header => buf.extend_from_slice(b"RFC822.HEADER"),
        FetchAttribute::Rfc822Text => buf.extend_from_slice(b"RFC822.TEXT"),
        FetchAttribute::ModSeq => buf.extend_from_slice(b"MODSEQ"),
        FetchAttribute::Body {
            section,
            peek,
            partial,
        } => {
            if *peek {
                buf.extend_from_slice(b"BODY.PEEK[");
            } else {
                buf.extend_from_slice(b"BODY[");
            }
            if let Some(s) = section {
                buf.extend_from_slice(s.as_bytes());
            }
            buf.push(b']');
            if let Some((start, len)) = partial {
                buf.extend_from_slice(format!("<{start}.{len}>").as_bytes());
            }
        }
    }
}

/// Writes STORE action.
pub fn write_store_action(buf: &mut Vec<u8>, action: &StoreAction, silent: bool) {
    match action {
        StoreAction::SetFlags(f) | StoreAction::AddFlags(f) | StoreAction::RemoveFlags(f) => {
            let prefix = match action {
                StoreAction::SetFlags(_) => "FLAGS",
                StoreAction::AddFlags(_) => "+FLAGS",
                StoreAction::RemoveFlags(_) => "-FLAGS",
                _ => unreachable!(),
            };
            buf.extend_from_slice(prefix.as_bytes());
            if silent {
                buf.extend_from_slice(b".SILENT");
            }
            buf.extend_from_slice(b" (");
            for (i, flag) in f.iter().enumerate() {
                if i > 0 {
                    buf.push(b' ');
                }
                buf.extend_from_slice(flag.as_str().as_bytes());
            }
            buf.push(b')');
        }
        StoreAction::SetFlagsUnchangedSince { flags, modseq }
        | StoreAction::AddFlagsUnchangedSince { flags, modseq }
        | StoreAction::RemoveFlagsUnchangedSince { flags, modseq } => {
            let prefix = match action {
                StoreAction::SetFlagsUnchangedSince { .. } => "FLAGS",
                StoreAction::AddFlagsUnchangedSince { .. } => "+FLAGS",
                StoreAction::RemoveFlagsUnchangedSince { .. } => "-FLAGS",
                _ => unreachable!(),
            };
            buf.extend_from_slice(prefix.as_bytes());
            if silent {
                buf.extend_from_slice(b".SILENT");
            }
            buf.extend_from_slice(format!(" (UNCHANGEDSINCE {modseq}) (").as_bytes());
            for (i, flag) in flags.iter().enumerate() {
                if i > 0 {
                    buf.push(b' ');
                }
                buf.extend_from_slice(flag.as_str().as_bytes());
            }
            buf.push(b')');
        }
    }
}

/// Writes SEARCH criteria.
pub fn write_search_criteria(buf: &mut Vec<u8>, criteria: &SearchCriteria) {
    match criteria {
        SearchCriteria::All => buf.extend_from_slice(b"ALL"),
        SearchCriteria::Answered => buf.extend_from_slice(b"ANSWERED"),
        SearchCriteria::Deleted => buf.extend_from_slice(b"DELETED"),
        SearchCriteria::Draft => buf.extend_from_slice(b"DRAFT"),
        SearchCriteria::Flagged => buf.extend_from_slice(b"FLAGGED"),
        SearchCriteria::New => buf.extend_from_slice(b"NEW"),
        SearchCriteria::Undeleted => buf.extend_from_slice(b"UNDELETED"),
        SearchCriteria::Unseen => buf.extend_from_slice(b"UNSEEN"),
        SearchCriteria::Seen => buf.extend_from_slice(b"SEEN"),
        SearchCriteria::SequenceSet(set) => {
            buf.extend_from_slice(set.to_string().as_bytes());
        }
        SearchCriteria::UidSet(set) => {
            buf.extend_from_slice(b"UID ");
            buf.extend_from_slice(set.to_string().as_bytes());
        }
        SearchCriteria::Subject(s) => {
            buf.extend_from_slice(b"SUBJECT ");
            write_astring(buf, s);
        }
        SearchCriteria::From(s) => {
            buf.extend_from_slice(b"FROM ");
            write_astring(buf, s);
        }
        SearchCriteria::To(s) => {
            buf.extend_from_slice(b"TO ");
            write_astring(buf, s);
        }
        SearchCriteria::Body(s) => {
            buf.extend_from_slice(b"BODY ");
            write_astring(buf, s);
        }
        SearchCriteria::Text(s) => {
            buf.extend_from_slice(b"TEXT ");
            write_astring(buf, s);
        }
        SearchCriteria::Since(date) => {
            buf.extend_from_slice(b"SINCE ");
            buf.extend_from_slice(date.as_bytes());
        }
        SearchCriteria::Before(date) => {
            buf.extend_from_slice(b"BEFORE ");
            buf.extend_from_slice(date.as_bytes());
        }
        SearchCriteria::On(date) => {
            buf.extend_from_slice(b"ON ");
            buf.extend_from_slice(date.as_bytes());
        }
        SearchCriteria::Larger(size) => {
            buf.extend_from_slice(format!("LARGER {size}").as_bytes());
        }
        SearchCriteria::Smaller(size) => {
            buf.extend_from_slice(format!("SMALLER {size}").as_bytes());
        }
        SearchCriteria::Header(name, value) => {
            buf.extend_from_slice(b"HEADER ");
            write_astring(buf, name);
            buf.push(b' ');
            write_astring(buf, value);
        }
        SearchCriteria::ModSeq(modseq) => {
            buf.extend_from_slice(format!("MODSEQ {modseq}").as_bytes());
        }
        SearchCriteria::And(criteria) => {
            for (i, c) in criteria.iter().enumerate() {
                if i > 0 {
                    buf.push(b' ');
                }
                write_search_criteria(buf, c);
            }
        }
        SearchCriteria::Or(a, b) => {
            buf.extend_from_slice(b"OR ");
            write_search_criteria(buf, a);
            buf.push(b' ');
            write_search_criteria(buf, b);
        }
        SearchCriteria::Not(c) => {
            buf.extend_from_slice(b"NOT ");
            write_search_criteria(buf, c);
        }
    }
}
