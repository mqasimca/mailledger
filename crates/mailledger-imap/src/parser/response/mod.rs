//! IMAP response parser.
//!
//! Parses server responses according to RFC 9051 grammar.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::elidable_lifetime_names)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::needless_continue)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::option_if_let_else)]

mod fetch;
mod helpers;
mod types;

pub use types::{Address, BodyStructure, Envelope, FetchItem, StatusItem, UntaggedResponse};

use crate::parser::lexer::{Lexer, Token};
use crate::types::{ResponseCode, SeqNum, Status, Tag};
use crate::{Error, Result};

use helpers::{
    parse_capability_data, parse_list_response, parse_response_code, parse_search_response,
    parse_status_response, read_text_until_crlf,
};

/// A parsed IMAP response.
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Tagged response (command completion).
    Tagged {
        /// The command tag.
        tag: Tag,
        /// Response status.
        status: Status,
        /// Optional response code.
        code: Option<ResponseCode>,
        /// Human-readable text.
        text: String,
    },
    /// Untagged response (server data).
    Untagged(UntaggedResponse),
    /// Continuation request.
    Continuation {
        /// Optional text/data.
        text: Option<String>,
    },
}

/// Response parser.
pub struct ResponseParser;

impl ResponseParser {
    /// Parses a complete response line.
    pub fn parse(input: &[u8]) -> Result<Response> {
        let mut lexer = Lexer::new(input);

        match lexer.next_token()? {
            Token::Asterisk => Self::parse_untagged(&mut lexer),
            Token::Plus => Self::parse_continuation(&mut lexer),
            Token::Atom(tag) => Self::parse_tagged(&mut lexer, tag),
            token => Err(Error::Parse {
                position: 0,
                message: format!("Expected *, +, or tag, got {token:?}"),
            }),
        }
    }

    /// Parses a tagged response.
    fn parse_tagged(lexer: &mut Lexer<'_>, tag_str: &str) -> Result<Response> {
        lexer.expect_space()?;

        let status = Self::parse_status(lexer)?;
        lexer.expect_space()?;

        let (code, text) = Self::parse_resp_text(lexer)?;

        Ok(Response::Tagged {
            tag: Tag::new(tag_str),
            status,
            code,
            text,
        })
    }

    /// Parses an untagged response.
    fn parse_untagged(lexer: &mut Lexer<'_>) -> Result<Response> {
        lexer.expect_space()?;

        let token = lexer.next_token()?;

        match token {
            Token::Atom(s) => {
                let upper = s.to_uppercase();
                match upper.as_str() {
                    "OK" => {
                        lexer.expect_space()?;
                        let (code, text) = Self::parse_resp_text(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Ok { code, text }))
                    }
                    "NO" => {
                        lexer.expect_space()?;
                        let (code, text) = Self::parse_resp_text(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::No { code, text }))
                    }
                    "BAD" => {
                        lexer.expect_space()?;
                        let (code, text) = Self::parse_resp_text(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Bad { code, text }))
                    }
                    "PREAUTH" => {
                        lexer.expect_space()?;
                        let (code, text) = Self::parse_resp_text(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::PreAuth { code, text }))
                    }
                    "BYE" => {
                        lexer.expect_space()?;
                        let (code, text) = Self::parse_resp_text(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Bye { code, text }))
                    }
                    "CAPABILITY" => {
                        let caps = parse_capability_data(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Capability(caps)))
                    }
                    "FLAGS" => {
                        lexer.expect_space()?;
                        let flags = parse_flag_list(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Flags(flags)))
                    }
                    "LIST" => {
                        lexer.expect_space()?;
                        let list = parse_list_response(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::List(list)))
                    }
                    "SEARCH" => {
                        let nums = parse_search_response(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Search(nums)))
                    }
                    "STATUS" => {
                        lexer.expect_space()?;
                        let (mailbox, items) = parse_status_response(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Status {
                            mailbox,
                            items,
                        }))
                    }
                    _ => Err(Error::Parse {
                        position: lexer.position(),
                        message: format!("Unknown untagged response: {s}"),
                    }),
                }
            }
            Token::Number(n) => {
                lexer.expect_space()?;
                let keyword = lexer.read_atom_string()?;
                let upper = keyword.to_uppercase();

                match upper.as_str() {
                    "EXISTS" => Ok(Response::Untagged(UntaggedResponse::Exists(n))),
                    "RECENT" => Ok(Response::Untagged(UntaggedResponse::Recent(n))),
                    "EXPUNGE" => {
                        let seq = SeqNum::new(n).ok_or_else(|| Error::Parse {
                            position: lexer.position(),
                            message: "Invalid sequence number 0".to_string(),
                        })?;
                        Ok(Response::Untagged(UntaggedResponse::Expunge(seq)))
                    }
                    "FETCH" => {
                        let seq = SeqNum::new(n).ok_or_else(|| Error::Parse {
                            position: lexer.position(),
                            message: "Invalid sequence number 0".to_string(),
                        })?;
                        lexer.expect_space()?;
                        let items = fetch::parse_fetch_response(lexer)?;
                        Ok(Response::Untagged(UntaggedResponse::Fetch { seq, items }))
                    }
                    _ => Err(Error::Parse {
                        position: lexer.position(),
                        message: format!("Unknown message data: {keyword}"),
                    }),
                }
            }
            _ => Err(Error::Parse {
                position: lexer.position(),
                message: format!("Unexpected token in untagged response: {token:?}"),
            }),
        }
    }

    /// Parses a continuation response.
    fn parse_continuation(lexer: &mut Lexer<'_>) -> Result<Response> {
        // Skip optional space
        if lexer.peek() == Some(b' ') {
            lexer.advance();
        }

        // Read remaining text until CRLF
        let text = read_text_until_crlf(lexer);

        Ok(Response::Continuation {
            text: if text.is_empty() { None } else { Some(text) },
        })
    }

    /// Parses a status keyword.
    fn parse_status(lexer: &mut Lexer<'_>) -> Result<Status> {
        let s = lexer.read_atom_string()?;
        match s.to_uppercase().as_str() {
            "OK" => Ok(Status::Ok),
            "NO" => Ok(Status::No),
            "BAD" => Ok(Status::Bad),
            "PREAUTH" => Ok(Status::PreAuth),
            "BYE" => Ok(Status::Bye),
            _ => Err(Error::Parse {
                position: lexer.position(),
                message: format!("Invalid status: {s}"),
            }),
        }
    }

    /// Parses response text with optional response code.
    fn parse_resp_text(lexer: &mut Lexer<'_>) -> Result<(Option<ResponseCode>, String)> {
        let code = if lexer.peek() == Some(b'[') {
            Some(parse_response_code(lexer)?)
        } else {
            None
        };

        // Skip space after code if present
        if lexer.peek() == Some(b' ') {
            lexer.advance();
        }

        let text = read_text_until_crlf(lexer);

        Ok((code, text))
    }
}

// Re-export parse_flag_list for fetch module
pub(crate) use helpers::parse_flag_list;

#[cfg(test)]
mod tests {
    use crate::types::{Capability, Flag, MailboxAttribute, ResponseCode};

    use super::*;

    #[test]
    fn test_parse_ok_response() {
        let input = b"* OK IMAP4rev2 server ready\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Ok { code, text }) => {
                assert!(code.is_none());
                assert_eq!(text, "IMAP4rev2 server ready");
            }
            _ => panic!("Expected untagged OK"),
        }
    }

    #[test]
    fn test_parse_tagged_ok() {
        let input = b"A001 OK LOGIN completed\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Tagged {
                tag,
                status,
                code,
                text,
            } => {
                assert_eq!(tag.as_str(), "A001");
                assert_eq!(status, Status::Ok);
                assert!(code.is_none());
                assert_eq!(text, "LOGIN completed");
            }
            _ => panic!("Expected tagged response"),
        }
    }

    #[test]
    fn test_parse_capability() {
        let input = b"* CAPABILITY IMAP4rev1 IDLE NAMESPACE\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Capability(caps)) => {
                assert!(caps.contains(&Capability::Imap4Rev1));
                assert!(caps.contains(&Capability::Idle));
                assert!(caps.contains(&Capability::Namespace));
            }
            _ => panic!("Expected capability response"),
        }
    }

    #[test]
    fn test_parse_exists() {
        let input = b"* 23 EXISTS\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Exists(n)) => {
                assert_eq!(n, 23);
            }
            _ => panic!("Expected EXISTS"),
        }
    }

    #[test]
    fn test_parse_flags() {
        let input = b"* FLAGS (\\Seen \\Answered \\Flagged \\Deleted \\Draft)\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Flags(flags)) => {
                assert!(flags.contains(&Flag::Seen));
                assert!(flags.contains(&Flag::Answered));
                assert!(flags.contains(&Flag::Flagged));
            }
            _ => panic!("Expected FLAGS"),
        }
    }

    #[test]
    fn test_parse_list() {
        let input = b"* LIST (\\HasChildren) \"/\" \"INBOX\"\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::List(list)) => {
                assert!(list.attributes.contains(&MailboxAttribute::HasChildren));
                assert_eq!(list.delimiter, Some('/'));
                assert_eq!(list.mailbox.as_str(), "INBOX");
            }
            _ => panic!("Expected LIST"),
        }
    }

    #[test]
    fn test_parse_continuation() {
        let input = b"+ Ready for literal\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Continuation { text } => {
                assert_eq!(text, Some("Ready for literal".to_string()));
            }
            _ => panic!("Expected continuation"),
        }
    }

    #[test]
    fn test_parse_response_code() {
        let input = b"* OK [UIDVALIDITY 1234567890] UIDs valid\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Ok { code, text }) => {
                match code {
                    Some(ResponseCode::UidValidity(v)) => {
                        assert_eq!(v.get(), 1_234_567_890);
                    }
                    _ => panic!("Expected UIDVALIDITY code"),
                }
                assert_eq!(text, "UIDs valid");
            }
            _ => panic!("Expected untagged OK"),
        }
    }

    #[test]
    fn test_parse_fetch() {
        let input = b"* 1 FETCH (FLAGS (\\Seen) UID 12345)\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Fetch { seq, items }) => {
                assert_eq!(seq.get(), 1);
                assert!(
                    items
                        .iter()
                        .any(|i| matches!(i, FetchItem::Uid(uid) if uid.get() == 12345))
                );
                assert!(
                    items
                        .iter()
                        .any(|i| matches!(i, FetchItem::Flags(f) if f.is_seen()))
                );
            }
            _ => panic!("Expected FETCH"),
        }
    }

    #[test]
    fn test_parse_search() {
        let input = b"* SEARCH 1 2 3 5 8 13\r\n";
        let response = ResponseParser::parse(input).unwrap();

        match response {
            Response::Untagged(UntaggedResponse::Search(nums)) => {
                let values: Vec<u32> = nums.iter().map(|s| s.get()).collect();
                assert_eq!(values, vec![1, 2, 3, 5, 8, 13]);
            }
            _ => panic!("Expected SEARCH"),
        }
    }
}
