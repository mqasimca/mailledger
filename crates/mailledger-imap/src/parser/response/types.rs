//! Response data types.

use crate::types::{Flags, Mailbox, SeqNum, Uid, UidValidity};

/// FETCH response item.
#[derive(Debug, Clone, PartialEq)]
pub enum FetchItem {
    /// Message flags.
    Flags(Flags),
    /// Internal date.
    InternalDate(String),
    /// RFC822 size.
    Rfc822Size(u32),
    /// Envelope.
    Envelope(Box<Envelope>),
    /// UID.
    Uid(Uid),
    /// BODY section.
    Body {
        /// Section specifier.
        section: Option<String>,
        /// Origin offset.
        origin: Option<u32>,
        /// Body data.
        data: Option<Vec<u8>>,
    },
    /// BODYSTRUCTURE.
    BodyStructure(BodyStructure),
    /// MODSEQ (CONDSTORE).
    ModSeq(u64),
}

/// Message envelope.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Envelope {
    /// Date header.
    pub date: Option<String>,
    /// Subject header.
    pub subject: Option<String>,
    /// From addresses.
    pub from: Vec<Address>,
    /// Sender addresses.
    pub sender: Vec<Address>,
    /// Reply-To addresses.
    pub reply_to: Vec<Address>,
    /// To addresses.
    pub to: Vec<Address>,
    /// Cc addresses.
    pub cc: Vec<Address>,
    /// Bcc addresses.
    pub bcc: Vec<Address>,
    /// In-Reply-To header.
    pub in_reply_to: Option<String>,
    /// Message-ID header.
    pub message_id: Option<String>,
}

/// Email address from envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    /// Display name.
    pub name: Option<String>,
    /// Source route (obsolete).
    pub adl: Option<String>,
    /// Mailbox name (local part).
    pub mailbox: Option<String>,
    /// Host name (domain part).
    pub host: Option<String>,
}

impl Address {
    /// Returns the full email address.
    #[must_use]
    pub fn email(&self) -> Option<String> {
        match (&self.mailbox, &self.host) {
            (Some(m), Some(h)) => Some(format!("{m}@{h}")),
            _ => None,
        }
    }
}

/// Body structure (simplified).
#[derive(Debug, Clone, PartialEq)]
pub enum BodyStructure {
    /// Single-part body.
    Basic {
        /// MIME type.
        media_type: String,
        /// MIME subtype.
        media_subtype: String,
        /// Body parameters.
        params: Vec<(String, String)>,
        /// Content-ID.
        id: Option<String>,
        /// Content-Description.
        description: Option<String>,
        /// Content-Transfer-Encoding.
        encoding: String,
        /// Body size in bytes.
        size: u32,
    },
    /// Message/RFC822 body.
    Message {
        /// Envelope of nested message.
        envelope: Box<Envelope>,
        /// Body structure of nested message.
        body: Box<Self>,
        /// Size in lines.
        lines: u32,
    },
    /// Text body.
    Text {
        /// Text subtype.
        subtype: String,
        /// Body parameters.
        params: Vec<(String, String)>,
        /// Content-ID.
        id: Option<String>,
        /// Content-Description.
        description: Option<String>,
        /// Content-Transfer-Encoding.
        encoding: String,
        /// Body size in bytes.
        size: u32,
        /// Size in lines.
        lines: u32,
    },
    /// Multipart body.
    Multipart {
        /// Child body parts.
        bodies: Vec<Self>,
        /// Multipart subtype.
        subtype: String,
    },
}

/// STATUS response item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusItem {
    /// Number of messages.
    Messages(u32),
    /// Number of recent messages.
    Recent(u32),
    /// Next UID.
    UidNext(Uid),
    /// UIDVALIDITY value.
    UidValidity(UidValidity),
    /// Number of unseen messages.
    Unseen(u32),
    /// Highest mod-sequence.
    HighestModSeq(u64),
}

/// Untagged response data.
#[derive(Debug, Clone, PartialEq)]
pub enum UntaggedResponse {
    /// OK response with optional code.
    Ok {
        /// Optional response code.
        code: Option<crate::types::ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// NO response.
    No {
        /// Optional response code.
        code: Option<crate::types::ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// BAD response.
    Bad {
        /// Optional response code.
        code: Option<crate::types::ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// PREAUTH response.
    PreAuth {
        /// Optional response code.
        code: Option<crate::types::ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// BYE response.
    Bye {
        /// Optional response code.
        code: Option<crate::types::ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// CAPABILITY response.
    Capability(Vec<crate::types::Capability>),
    /// LIST response.
    List(crate::types::ListResponse),
    /// FLAGS response.
    Flags(Flags),
    /// EXISTS response (message count).
    Exists(u32),
    /// RECENT response.
    Recent(u32),
    /// EXPUNGE response (message removed).
    Expunge(SeqNum),
    /// FETCH response.
    Fetch {
        /// Message sequence number.
        seq: SeqNum,
        /// Fetch data items.
        items: Vec<FetchItem>,
    },
    /// SEARCH response.
    Search(Vec<SeqNum>),
    /// STATUS response.
    Status {
        /// Mailbox name.
        mailbox: Mailbox,
        /// Status items.
        items: Vec<StatusItem>,
    },
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    mod address_tests {
        use super::*;

        #[test]
        fn email_with_both_parts() {
            let addr = Address {
                name: Some("John Doe".to_string()),
                adl: None,
                mailbox: Some("john".to_string()),
                host: Some("example.com".to_string()),
            };
            assert_eq!(addr.email(), Some("john@example.com".to_string()));
        }

        #[test]
        fn email_without_mailbox() {
            let addr = Address {
                name: Some("John Doe".to_string()),
                adl: None,
                mailbox: None,
                host: Some("example.com".to_string()),
            };
            assert_eq!(addr.email(), None);
        }

        #[test]
        fn email_without_host() {
            let addr = Address {
                name: None,
                adl: None,
                mailbox: Some("john".to_string()),
                host: None,
            };
            assert_eq!(addr.email(), None);
        }

        #[test]
        fn email_with_neither() {
            let addr = Address {
                name: Some("John Doe".to_string()),
                adl: Some("route".to_string()),
                mailbox: None,
                host: None,
            };
            assert_eq!(addr.email(), None);
        }
    }

    mod envelope_tests {
        use super::*;

        #[test]
        fn default_envelope_is_empty() {
            let env = Envelope::default();
            assert!(env.date.is_none());
            assert!(env.subject.is_none());
            assert!(env.from.is_empty());
            assert!(env.to.is_empty());
            assert!(env.cc.is_empty());
            assert!(env.bcc.is_empty());
            assert!(env.message_id.is_none());
        }

        #[test]
        fn envelope_with_addresses() {
            let from = Address {
                name: Some("Sender".to_string()),
                adl: None,
                mailbox: Some("sender".to_string()),
                host: Some("example.com".to_string()),
            };
            let to = Address {
                name: Some("Recipient".to_string()),
                adl: None,
                mailbox: Some("recipient".to_string()),
                host: Some("example.org".to_string()),
            };
            let env = Envelope {
                date: Some("Mon, 1 Jan 2024 12:00:00 +0000".to_string()),
                subject: Some("Test Subject".to_string()),
                from: vec![from.clone()],
                sender: vec![from],
                reply_to: vec![],
                to: vec![to],
                cc: vec![],
                bcc: vec![],
                in_reply_to: None,
                message_id: Some("<msg@example.com>".to_string()),
            };
            assert_eq!(env.from.len(), 1);
            assert_eq!(env.to.len(), 1);
            assert_eq!(env.from[0].email(), Some("sender@example.com".to_string()));
        }
    }

    mod fetch_item_tests {
        use super::*;
        use crate::types::Flag;

        #[test]
        fn fetch_item_flags() {
            let flags = Flags::from_vec(vec![Flag::Seen, Flag::Answered]);
            let item = FetchItem::Flags(flags.clone());
            if let FetchItem::Flags(f) = item {
                assert!(f.contains(&Flag::Seen));
                assert!(f.contains(&Flag::Answered));
            } else {
                panic!("Expected FetchItem::Flags");
            }
        }

        #[test]
        fn fetch_item_rfc822_size() {
            let item = FetchItem::Rfc822Size(12345);
            if let FetchItem::Rfc822Size(size) = item {
                assert_eq!(size, 12345);
            } else {
                panic!("Expected FetchItem::Rfc822Size");
            }
        }

        #[test]
        fn fetch_item_body_with_data() {
            let item = FetchItem::Body {
                section: Some("1".to_string()),
                origin: Some(0),
                data: Some(b"Hello, World!".to_vec()),
            };
            if let FetchItem::Body {
                section,
                origin,
                data,
            } = item
            {
                assert_eq!(section, Some("1".to_string()));
                assert_eq!(origin, Some(0));
                assert_eq!(data, Some(b"Hello, World!".to_vec()));
            } else {
                panic!("Expected FetchItem::Body");
            }
        }

        #[test]
        fn fetch_item_modseq() {
            let item = FetchItem::ModSeq(98765);
            if let FetchItem::ModSeq(seq) = item {
                assert_eq!(seq, 98765);
            } else {
                panic!("Expected FetchItem::ModSeq");
            }
        }
    }

    mod body_structure_tests {
        use super::*;

        #[test]
        fn basic_body_structure() {
            let body = BodyStructure::Basic {
                media_type: "application".to_string(),
                media_subtype: "pdf".to_string(),
                params: vec![("name".to_string(), "document.pdf".to_string())],
                id: Some("<part1>".to_string()),
                description: Some("PDF attachment".to_string()),
                encoding: "base64".to_string(),
                size: 102400,
            };
            if let BodyStructure::Basic {
                media_type,
                media_subtype,
                size,
                ..
            } = body
            {
                assert_eq!(media_type, "application");
                assert_eq!(media_subtype, "pdf");
                assert_eq!(size, 102400);
            } else {
                panic!("Expected BodyStructure::Basic");
            }
        }

        #[test]
        fn text_body_structure() {
            let body = BodyStructure::Text {
                subtype: "plain".to_string(),
                params: vec![("charset".to_string(), "utf-8".to_string())],
                id: None,
                description: None,
                encoding: "7bit".to_string(),
                size: 500,
                lines: 25,
            };
            if let BodyStructure::Text { subtype, lines, .. } = body {
                assert_eq!(subtype, "plain");
                assert_eq!(lines, 25);
            } else {
                panic!("Expected BodyStructure::Text");
            }
        }

        #[test]
        fn multipart_body_structure() {
            let part1 = BodyStructure::Text {
                subtype: "plain".to_string(),
                params: vec![],
                id: None,
                description: None,
                encoding: "7bit".to_string(),
                size: 100,
                lines: 5,
            };
            let part2 = BodyStructure::Text {
                subtype: "html".to_string(),
                params: vec![],
                id: None,
                description: None,
                encoding: "quoted-printable".to_string(),
                size: 500,
                lines: 20,
            };
            let body = BodyStructure::Multipart {
                bodies: vec![part1, part2],
                subtype: "alternative".to_string(),
            };
            if let BodyStructure::Multipart { bodies, subtype } = body {
                assert_eq!(subtype, "alternative");
                assert_eq!(bodies.len(), 2);
            } else {
                panic!("Expected BodyStructure::Multipart");
            }
        }
    }

    mod status_item_tests {
        use super::*;

        #[test]
        fn status_messages() {
            let item = StatusItem::Messages(42);
            if let StatusItem::Messages(count) = item {
                assert_eq!(count, 42);
            } else {
                panic!("Expected StatusItem::Messages");
            }
        }

        #[test]
        fn status_unseen() {
            let item = StatusItem::Unseen(5);
            if let StatusItem::Unseen(count) = item {
                assert_eq!(count, 5);
            } else {
                panic!("Expected StatusItem::Unseen");
            }
        }

        #[test]
        fn status_highest_modseq() {
            let item = StatusItem::HighestModSeq(123456789);
            if let StatusItem::HighestModSeq(seq) = item {
                assert_eq!(seq, 123456789);
            } else {
                panic!("Expected StatusItem::HighestModSeq");
            }
        }
    }

    mod untagged_response_tests {
        use super::*;

        #[test]
        fn exists_response() {
            let resp = UntaggedResponse::Exists(150);
            if let UntaggedResponse::Exists(count) = resp {
                assert_eq!(count, 150);
            } else {
                panic!("Expected UntaggedResponse::Exists");
            }
        }

        #[test]
        fn recent_response() {
            let resp = UntaggedResponse::Recent(3);
            if let UntaggedResponse::Recent(count) = resp {
                assert_eq!(count, 3);
            } else {
                panic!("Expected UntaggedResponse::Recent");
            }
        }

        #[test]
        fn search_response() {
            let seq_nums = vec![
                SeqNum::new(1).unwrap(),
                SeqNum::new(5).unwrap(),
                SeqNum::new(10).unwrap(),
            ];
            let resp = UntaggedResponse::Search(seq_nums.clone());
            if let UntaggedResponse::Search(results) = resp {
                assert_eq!(results.len(), 3);
            } else {
                panic!("Expected UntaggedResponse::Search");
            }
        }

        #[test]
        fn fetch_response() {
            let items = vec![FetchItem::Rfc822Size(1000), FetchItem::ModSeq(12345)];
            let resp = UntaggedResponse::Fetch {
                seq: SeqNum::new(1).unwrap(),
                items,
            };
            if let UntaggedResponse::Fetch { seq, items } = resp {
                assert_eq!(seq.get(), 1);
                assert_eq!(items.len(), 2);
            } else {
                panic!("Expected UntaggedResponse::Fetch");
            }
        }
    }
}
