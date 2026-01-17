//! IMAP command builder.
//!
//! This module provides types and serialization for IMAP commands.

mod serialize;
mod tag_generator;
mod types;

use crate::types::{Flag, Mailbox, SequenceSet};

pub use tag_generator::TagGenerator;
pub use types::{FetchAttribute, FetchItems, SearchCriteria, StatusAttribute, StoreAction};

use serialize::{
    write_astring, write_fetch_items, write_mailbox, write_search_criteria, write_store_action,
};

/// IMAP command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    // Any State Commands
    /// CAPABILITY command.
    Capability,
    /// NOOP command.
    Noop,
    /// LOGOUT command.
    Logout,

    // Not Authenticated State Commands
    /// STARTTLS command.
    StartTls,
    /// LOGIN command.
    Login {
        /// Username.
        username: String,
        /// Password.
        password: String,
    },
    /// AUTHENTICATE command.
    Authenticate {
        /// Authentication mechanism.
        mechanism: String,
        /// Initial response (optional).
        initial_response: Option<String>,
    },

    // Authenticated State Commands
    /// ID command (RFC 2971) - client/server identification.
    Id {
        /// Client identification parameters (field-value pairs).
        /// None = ID NIL (no identification).
        parameters: Option<Vec<(String, String)>>,
    },
    /// ENABLE command.
    Enable {
        /// Capabilities to enable.
        capabilities: Vec<String>,
    },
    /// SELECT command.
    Select {
        /// Mailbox to select.
        mailbox: Mailbox,
        /// Enable CONDSTORE.
        condstore: bool,
    },
    /// EXAMINE command (read-only SELECT).
    Examine {
        /// Mailbox to examine.
        mailbox: Mailbox,
    },
    /// CREATE command.
    Create {
        /// Mailbox to create.
        mailbox: Mailbox,
    },
    /// DELETE command.
    Delete {
        /// Mailbox to delete.
        mailbox: Mailbox,
    },
    /// RENAME command.
    Rename {
        /// Current mailbox name.
        from: Mailbox,
        /// New mailbox name.
        to: Mailbox,
    },
    /// SUBSCRIBE command.
    Subscribe {
        /// Mailbox to subscribe.
        mailbox: Mailbox,
    },
    /// UNSUBSCRIBE command.
    Unsubscribe {
        /// Mailbox to unsubscribe.
        mailbox: Mailbox,
    },
    /// LIST command.
    List {
        /// Reference name.
        reference: String,
        /// Mailbox pattern.
        pattern: String,
    },
    /// NAMESPACE command.
    Namespace,
    /// STATUS command.
    Status {
        /// Mailbox name.
        mailbox: Mailbox,
        /// Status items to request.
        items: Vec<StatusAttribute>,
    },
    /// APPEND command.
    Append {
        /// Target mailbox.
        mailbox: Mailbox,
        /// Flags to set.
        flags: Option<Vec<Flag>>,
        /// Message data.
        message: Vec<u8>,
    },

    // Selected State Commands
    /// CLOSE command.
    Close,
    /// UNSELECT command.
    Unselect,
    /// EXPUNGE command.
    Expunge,
    /// UID EXPUNGE command (RFC 4315 UIDPLUS) - expunge specific UIDs.
    UidExpunge {
        /// UIDs to expunge.
        uids: SequenceSet,
    },
    /// SEARCH command.
    Search {
        /// Search criteria.
        criteria: SearchCriteria,
        /// Use UIDs.
        uid: bool,
    },
    /// FETCH command.
    Fetch {
        /// Sequence set.
        sequence: SequenceSet,
        /// Items to fetch.
        items: FetchItems,
        /// Use UIDs.
        uid: bool,
    },
    /// STORE command.
    Store {
        /// Sequence set.
        sequence: SequenceSet,
        /// Store action.
        action: StoreAction,
        /// Use UIDs.
        uid: bool,
        /// Silent mode (no FETCH response).
        silent: bool,
    },
    /// COPY command.
    Copy {
        /// Sequence set.
        sequence: SequenceSet,
        /// Target mailbox.
        mailbox: Mailbox,
        /// Use UIDs.
        uid: bool,
    },
    /// MOVE command.
    Move {
        /// Sequence set.
        sequence: SequenceSet,
        /// Target mailbox.
        mailbox: Mailbox,
        /// Use UIDs.
        uid: bool,
    },
    /// IDLE command.
    Idle,
    /// DONE (to end IDLE).
    Done,
}

impl Command {
    /// Serializes the command to bytes with the given tag.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn serialize(&self, tag: &str) -> Vec<u8> {
        let mut buf = Vec::new();

        // DONE doesn't get a tag (it's sent during IDLE)
        if !matches!(self, Self::Done) {
            buf.extend_from_slice(tag.as_bytes());
            buf.push(b' ');
        }

        match self {
            Self::Capability => buf.extend_from_slice(b"CAPABILITY"),
            Self::Noop => buf.extend_from_slice(b"NOOP"),
            Self::Logout => buf.extend_from_slice(b"LOGOUT"),
            Self::StartTls => buf.extend_from_slice(b"STARTTLS"),

            Self::Login { username, password } => {
                buf.extend_from_slice(b"LOGIN ");
                write_astring(&mut buf, username);
                buf.push(b' ');
                write_astring(&mut buf, password);
            }

            Self::Authenticate {
                mechanism,
                initial_response,
            } => {
                buf.extend_from_slice(b"AUTHENTICATE ");
                buf.extend_from_slice(mechanism.as_bytes());
                if let Some(resp) = initial_response {
                    buf.push(b' ');
                    buf.extend_from_slice(resp.as_bytes());
                }
            }

            Self::Id { parameters } => {
                buf.extend_from_slice(b"ID ");
                if let Some(params) = parameters {
                    buf.push(b'(');
                    for (i, (key, value)) in params.iter().enumerate() {
                        if i > 0 {
                            buf.push(b' ');
                        }
                        write_astring(&mut buf, key);
                        buf.push(b' ');
                        write_astring(&mut buf, value);
                    }
                    buf.push(b')');
                } else {
                    buf.extend_from_slice(b"NIL");
                }
            }

            Self::Enable { capabilities } => {
                buf.extend_from_slice(b"ENABLE");
                for cap in capabilities {
                    buf.push(b' ');
                    buf.extend_from_slice(cap.as_bytes());
                }
            }

            Self::Select { mailbox, condstore } => {
                buf.extend_from_slice(b"SELECT ");
                write_mailbox(&mut buf, mailbox);
                if *condstore {
                    buf.extend_from_slice(b" (CONDSTORE)");
                }
            }

            Self::Examine { mailbox } => {
                buf.extend_from_slice(b"EXAMINE ");
                write_mailbox(&mut buf, mailbox);
            }

            Self::Create { mailbox } => {
                buf.extend_from_slice(b"CREATE ");
                write_mailbox(&mut buf, mailbox);
            }

            Self::Delete { mailbox } => {
                buf.extend_from_slice(b"DELETE ");
                write_mailbox(&mut buf, mailbox);
            }

            Self::Rename { from, to } => {
                buf.extend_from_slice(b"RENAME ");
                write_mailbox(&mut buf, from);
                buf.push(b' ');
                write_mailbox(&mut buf, to);
            }

            Self::Subscribe { mailbox } => {
                buf.extend_from_slice(b"SUBSCRIBE ");
                write_mailbox(&mut buf, mailbox);
            }

            Self::Unsubscribe { mailbox } => {
                buf.extend_from_slice(b"UNSUBSCRIBE ");
                write_mailbox(&mut buf, mailbox);
            }

            Self::List { reference, pattern } => {
                buf.extend_from_slice(b"LIST ");
                write_astring(&mut buf, reference);
                buf.push(b' ');
                write_astring(&mut buf, pattern);
            }

            Self::Namespace => buf.extend_from_slice(b"NAMESPACE"),

            Self::Status { mailbox, items } => {
                buf.extend_from_slice(b"STATUS ");
                write_mailbox(&mut buf, mailbox);
                buf.extend_from_slice(b" (");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        buf.push(b' ');
                    }
                    buf.extend_from_slice(item.as_str().as_bytes());
                }
                buf.push(b')');
            }

            Self::Append {
                mailbox,
                flags,
                message,
            } => {
                buf.extend_from_slice(b"APPEND ");
                write_mailbox(&mut buf, mailbox);
                if let Some(flags) = flags {
                    buf.extend_from_slice(b" (");
                    for (i, flag) in flags.iter().enumerate() {
                        if i > 0 {
                            buf.push(b' ');
                        }
                        buf.extend_from_slice(flag.as_str().as_bytes());
                    }
                    buf.push(b')');
                }
                buf.extend_from_slice(format!(" {{{}}}", message.len()).as_bytes());
            }

            Self::Close => buf.extend_from_slice(b"CLOSE"),
            Self::Unselect => buf.extend_from_slice(b"UNSELECT"),
            Self::Expunge => buf.extend_from_slice(b"EXPUNGE"),

            Self::UidExpunge { uids } => {
                buf.extend_from_slice(b"UID EXPUNGE ");
                buf.extend_from_slice(uids.to_string().as_bytes());
            }

            Self::Search { criteria, uid } => {
                if *uid {
                    buf.extend_from_slice(b"UID ");
                }
                buf.extend_from_slice(b"SEARCH ");
                write_search_criteria(&mut buf, criteria);
            }

            Self::Fetch {
                sequence,
                items,
                uid,
            } => {
                if *uid {
                    buf.extend_from_slice(b"UID ");
                }
                buf.extend_from_slice(b"FETCH ");
                buf.extend_from_slice(sequence.to_string().as_bytes());
                buf.push(b' ');
                write_fetch_items(&mut buf, items);
            }

            Self::Store {
                sequence,
                action,
                uid,
                silent,
            } => {
                if *uid {
                    buf.extend_from_slice(b"UID ");
                }
                buf.extend_from_slice(b"STORE ");
                buf.extend_from_slice(sequence.to_string().as_bytes());
                buf.push(b' ');
                write_store_action(&mut buf, action, *silent);
            }

            Self::Copy {
                sequence,
                mailbox,
                uid,
            } => {
                if *uid {
                    buf.extend_from_slice(b"UID ");
                }
                buf.extend_from_slice(b"COPY ");
                buf.extend_from_slice(sequence.to_string().as_bytes());
                buf.push(b' ');
                write_mailbox(&mut buf, mailbox);
            }

            Self::Move {
                sequence,
                mailbox,
                uid,
            } => {
                if *uid {
                    buf.extend_from_slice(b"UID ");
                }
                buf.extend_from_slice(b"MOVE ");
                buf.extend_from_slice(sequence.to_string().as_bytes());
                buf.push(b' ');
                write_mailbox(&mut buf, mailbox);
            }

            Self::Idle => buf.extend_from_slice(b"IDLE"),
            Self::Done => buf.extend_from_slice(b"DONE"),
        }

        buf.extend_from_slice(b"\r\n");
        buf
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use crate::types::Flag;

    use super::*;

    #[test]
    fn test_capability_command() {
        let cmd = Command::Capability;
        assert_eq!(cmd.serialize("A001"), b"A001 CAPABILITY\r\n");
    }

    #[test]
    fn test_login_command() {
        let cmd = Command::Login {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert_eq!(cmd.serialize("A001"), b"A001 LOGIN user pass\r\n");
    }

    #[test]
    fn test_login_quoted() {
        let cmd = Command::Login {
            username: "user@example.com".to_string(),
            password: "pass word".to_string(),
        };
        assert_eq!(
            cmd.serialize("A001"),
            b"A001 LOGIN user@example.com \"pass word\"\r\n"
        );
    }

    #[test]
    fn test_select_command() {
        let cmd = Command::Select {
            mailbox: Mailbox::inbox(),
            condstore: false,
        };
        assert_eq!(cmd.serialize("A001"), b"A001 SELECT INBOX\r\n");
    }

    #[test]
    fn test_select_condstore() {
        let cmd = Command::Select {
            mailbox: Mailbox::inbox(),
            condstore: true,
        };
        assert_eq!(cmd.serialize("A001"), b"A001 SELECT INBOX (CONDSTORE)\r\n");
    }

    #[test]
    fn test_list_command() {
        let cmd = Command::List {
            reference: String::new(),
            pattern: "*".to_string(),
        };
        // Note: * is quoted since it's a list-wildcard; both quoted and unquoted are valid
        assert_eq!(cmd.serialize("A001"), b"A001 LIST \"\" \"*\"\r\n");
    }

    #[test]
    fn test_fetch_command() {
        let cmd = Command::Fetch {
            sequence: SequenceSet::range(1, 10).unwrap(),
            items: FetchItems::Items(vec![FetchAttribute::Flags, FetchAttribute::Uid]),
            uid: false,
        };
        assert_eq!(cmd.serialize("A001"), b"A001 FETCH 1:10 (FLAGS UID)\r\n");
    }

    #[test]
    fn test_uid_fetch_command() {
        let cmd = Command::Fetch {
            sequence: SequenceSet::All,
            items: FetchItems::All,
            uid: true,
        };
        assert_eq!(cmd.serialize("A001"), b"A001 UID FETCH * ALL\r\n");
    }

    #[test]
    fn test_store_command() {
        let cmd = Command::Store {
            sequence: SequenceSet::single(1).unwrap(),
            action: StoreAction::AddFlags(vec![Flag::Seen]),
            uid: false,
            silent: true,
        };
        assert_eq!(
            cmd.serialize("A001"),
            b"A001 STORE 1 +FLAGS.SILENT (\\Seen)\r\n"
        );
    }

    #[test]
    fn test_search_command() {
        let cmd = Command::Search {
            criteria: SearchCriteria::Unseen,
            uid: false,
        };
        assert_eq!(cmd.serialize("A001"), b"A001 SEARCH UNSEEN\r\n");
    }

    #[test]
    fn test_idle_command() {
        let cmd = Command::Idle;
        assert_eq!(cmd.serialize("A001"), b"A001 IDLE\r\n");
    }

    #[test]
    fn test_done_command() {
        let cmd = Command::Done;
        assert_eq!(cmd.serialize(""), b"DONE\r\n");
    }

    #[test]
    fn test_id_command_nil() {
        let cmd = Command::Id { parameters: None };
        assert_eq!(cmd.serialize("A001"), b"A001 ID NIL\r\n");
    }

    #[test]
    fn test_id_command_with_params() {
        let cmd = Command::Id {
            parameters: Some(vec![
                ("name".to_string(), "mailledger".to_string()),
                ("version".to_string(), "0.1.0".to_string()),
            ]),
        };
        assert_eq!(
            cmd.serialize("A001"),
            b"A001 ID (name mailledger version 0.1.0)\r\n"
        );
    }

    #[test]
    fn test_uid_expunge_command() {
        let cmd = Command::UidExpunge {
            uids: SequenceSet::range(100, 200).unwrap(),
        };
        assert_eq!(cmd.serialize("A001"), b"A001 UID EXPUNGE 100:200\r\n");
    }
}
