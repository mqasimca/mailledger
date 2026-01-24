//! FETCH response parsing.

use crate::parser::lexer::{Lexer, Token};
use crate::types::Uid;
use crate::{Error, Result};

use super::parse_flag_list;
use super::types::{Address, BodyStructure, Envelope, FetchItem};

/// Parses a FETCH response.
pub fn parse_fetch_response(lexer: &mut Lexer<'_>) -> Result<Vec<FetchItem>> {
    lexer.expect(Token::LParen)?;

    let mut items = Vec::new();

    loop {
        match lexer.next_token()? {
            Token::RParen => break,
            Token::Space => continue,
            Token::Atom(name) => {
                let upper = name.to_uppercase();
                match upper.as_str() {
                    "FLAGS" => {
                        lexer.expect_space()?;
                        let flags = parse_flag_list(lexer)?;
                        items.push(FetchItem::Flags(flags));
                    }
                    "UID" => {
                        lexer.expect_space()?;
                        let n = lexer.read_number()?;
                        let uid = Uid::new(n).ok_or_else(|| Error::Parse {
                            position: lexer.position(),
                            message: format!("invalid UID value: {n} (UID cannot be 0)"),
                        })?;
                        items.push(FetchItem::Uid(uid));
                    }
                    "RFC822.SIZE" => {
                        lexer.expect_space()?;
                        let size = lexer.read_number()?;
                        items.push(FetchItem::Rfc822Size(size));
                    }
                    "INTERNALDATE" => {
                        lexer.expect_space()?;
                        if let Token::QuotedString(date) = lexer.next_token()? {
                            items.push(FetchItem::InternalDate(date));
                        }
                    }
                    "ENVELOPE" => {
                        lexer.expect_space()?;
                        let envelope = parse_envelope(lexer)?;
                        items.push(FetchItem::Envelope(Box::new(envelope)));
                    }
                    "BODYSTRUCTURE" => {
                        // BODYSTRUCTURE returns a parenthesized structure, not a literal
                        lexer.expect_space()?;
                        let body_structure = parse_body_structure(lexer)?;
                        items.push(FetchItem::BodyStructure(body_structure));
                    }
                    "BODY" | "RFC822" | "RFC822.HEADER" | "RFC822.TEXT" => {
                        // Parse BODY[section]<origin> or BODY.PEEK[section]<origin> format
                        // The lexer tokenizes [ ] < > separately, so we need to consume them
                        let (section, origin) = parse_body_section_and_origin(lexer)?;

                        lexer.expect_space()?;
                        let token = lexer.next_token()?;
                        let data = match token {
                            Token::Literal(d) => Some(d),
                            Token::Nil => None,
                            _ => None,
                        };

                        items.push(FetchItem::Body {
                            section,
                            origin,
                            data,
                        });
                    }
                    "MODSEQ" => {
                        lexer.expect_space()?;
                        lexer.expect(Token::LParen)?;
                        let n = u64::from(lexer.read_number()?);
                        lexer.expect(Token::RParen)?;
                        items.push(FetchItem::ModSeq(n));
                    }
                    _ => {
                        // Skip unknown fetch items
                        skip_fetch_item(lexer)?;
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(items)
}

/// Parses optional [section] and <origin> from a BODY fetch response.
///
/// In IMAP FETCH responses, BODY can be followed by:
/// - [section] like [TEXT], [HEADER], [1], [1.MIME], etc.
/// - <origin> like <0> for partial fetches
fn parse_body_section_and_origin(lexer: &mut Lexer<'_>) -> Result<(Option<String>, Option<u32>)> {
    let mut section = None;
    let mut origin = None;

    // Check for [section]
    if lexer.peek() == Some(b'[') {
        lexer.advance(); // consume '['

        // Read section content until ]
        let mut section_buf = String::new();
        loop {
            match lexer.peek() {
                Some(b']') => {
                    lexer.advance();
                    break;
                }
                Some(b) => {
                    section_buf.push(b as char);
                    lexer.advance();
                }
                None => break,
            }
        }

        if !section_buf.is_empty() {
            section = Some(section_buf);
        }
    }

    // Check for <origin>
    if lexer.peek() == Some(b'<') {
        lexer.advance(); // consume '<'

        // Read origin number until >
        let mut origin_buf = String::new();
        loop {
            match lexer.peek() {
                Some(b'>') => {
                    lexer.advance();
                    break;
                }
                Some(b) if b.is_ascii_digit() => {
                    origin_buf.push(b as char);
                    lexer.advance();
                }
                _ => break,
            }
        }

        if !origin_buf.is_empty() {
            origin = origin_buf.parse().ok();
        }
    }

    Ok((section, origin))
}

/// Parses an envelope structure.
pub fn parse_envelope(lexer: &mut Lexer<'_>) -> Result<Envelope> {
    lexer.expect(Token::LParen)?;

    let date = lexer.read_nstring()?;
    lexer.expect_space()?;

    let subject = lexer.read_nstring()?;
    lexer.expect_space()?;

    let from = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let sender = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let reply_to = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let to = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let cc = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let bcc = parse_address_list(lexer)?;
    lexer.expect_space()?;

    let in_reply_to = lexer.read_nstring()?;
    lexer.expect_space()?;

    let message_id = lexer.read_nstring()?;

    lexer.expect(Token::RParen)?;

    Ok(Envelope {
        date,
        subject,
        from,
        sender,
        reply_to,
        to,
        cc,
        bcc,
        in_reply_to,
        message_id,
    })
}

/// Parses an address list.
pub fn parse_address_list(lexer: &mut Lexer<'_>) -> Result<Vec<Address>> {
    match lexer.next_token()? {
        Token::Nil => Ok(Vec::new()),
        Token::LParen => {
            let mut addresses = Vec::new();

            loop {
                match lexer.peek() {
                    Some(b')') => {
                        lexer.advance();
                        break;
                    }
                    Some(b'(') => {
                        addresses.push(parse_address(lexer)?);
                    }
                    Some(b' ') => {
                        lexer.advance();
                    }
                    _ => break,
                }
            }

            Ok(addresses)
        }
        token => Err(Error::Parse {
            position: lexer.position(),
            message: format!("Expected address list, got {token:?}"),
        }),
    }
}

/// Parses a single address.
pub fn parse_address(lexer: &mut Lexer<'_>) -> Result<Address> {
    lexer.expect(Token::LParen)?;

    let name = lexer.read_nstring()?;
    lexer.expect_space()?;

    let adl = lexer.read_nstring()?;
    lexer.expect_space()?;

    let mailbox = lexer.read_nstring()?;
    lexer.expect_space()?;

    let host = lexer.read_nstring()?;

    lexer.expect(Token::RParen)?;

    Ok(Address {
        name,
        adl,
        mailbox,
        host,
    })
}

/// Parses a BODYSTRUCTURE response.
///
/// BODYSTRUCTURE is a complex nested structure. This parser handles:
/// - Single-part bodies: ("TYPE" "SUBTYPE" params id desc enc size ...)
/// - Multipart bodies: ((part1) (part2) ... "SUBTYPE" ...)
pub fn parse_body_structure(lexer: &mut Lexer<'_>) -> Result<BodyStructure> {
    lexer.expect(Token::LParen)?;

    // Check if this is a multipart (starts with another paren) or single-part (starts with string)
    if lexer.peek() == Some(b'(') {
        // Multipart - collect parts
        let mut parts = Vec::new();
        while lexer.peek() == Some(b'(') {
            parts.push(parse_body_structure(lexer)?);
            // Skip optional space between parts
            if lexer.peek() == Some(b' ') {
                lexer.advance();
            }
        }

        // Subtype follows the parts
        let subtype = lexer.read_nstring()?.unwrap_or_default().to_uppercase();

        // Skip remaining optional parameters
        skip_to_close_paren(lexer)?;

        Ok(BodyStructure::Multipart {
            bodies: parts,
            subtype,
        })
    } else {
        // Single-part body
        let media_type = lexer.read_nstring()?.unwrap_or_default().to_uppercase();
        lexer.expect_space()?;

        let media_subtype = lexer.read_nstring()?.unwrap_or_default().to_uppercase();
        lexer.expect_space()?;

        // Parameters (NIL or parenthesized list)
        let params = parse_body_params(lexer)?;
        lexer.expect_space()?;

        // Content-ID (nstring)
        let id = lexer.read_nstring()?;
        lexer.expect_space()?;

        // Content-Description (nstring)
        let description = lexer.read_nstring()?;
        lexer.expect_space()?;

        // Content-Transfer-Encoding (string)
        let encoding = lexer.read_nstring()?.unwrap_or_default();
        lexer.expect_space()?;

        // Size in octets (number)
        let size = lexer.read_number()?;

        // For TEXT types, there's a line count
        let lines = if media_type == "TEXT" {
            if lexer.peek() == Some(b' ') {
                lexer.advance();
                Some(lexer.read_number()?)
            } else {
                None
            }
        } else {
            None
        };

        // Skip remaining optional parameters (MD5, disposition, language, location)
        skip_to_close_paren(lexer)?;

        if media_type == "TEXT" {
            Ok(BodyStructure::Text {
                subtype: media_subtype,
                params,
                id,
                description,
                encoding,
                size,
                lines: lines.unwrap_or(0),
            })
        } else {
            Ok(BodyStructure::Basic {
                media_type,
                media_subtype,
                params,
                id,
                description,
                encoding,
                size,
            })
        }
    }
}

/// Parses body parameters (NIL or (key value key value ...)).
fn parse_body_params(lexer: &mut Lexer<'_>) -> Result<Vec<(String, String)>> {
    match lexer.next_token()? {
        Token::Nil => Ok(Vec::new()),
        Token::LParen => {
            let mut params = Vec::new();
            loop {
                match lexer.peek() {
                    Some(b')') => {
                        lexer.advance();
                        break;
                    }
                    Some(b' ') => {
                        lexer.advance();
                    }
                    _ => {
                        let key = lexer.read_nstring()?.unwrap_or_default();
                        if lexer.peek() == Some(b' ') {
                            lexer.advance();
                        }
                        let value = lexer.read_nstring()?.unwrap_or_default();
                        params.push((key, value));
                    }
                }
            }
            Ok(params)
        }
        _ => Ok(Vec::new()),
    }
}

/// Skips to the closing parenthesis at the current nesting level.
fn skip_to_close_paren(lexer: &mut Lexer<'_>) -> Result<()> {
    let mut depth = 1;
    while depth > 0 {
        match lexer.peek() {
            Some(b'(') => {
                depth += 1;
                lexer.advance();
            }
            Some(b')') => {
                depth -= 1;
                lexer.advance();
            }
            Some(b'{') => {
                // Skip literal
                lexer.advance();
                let _ = lexer.next_token()?;
            }
            Some(_) => {
                lexer.advance();
            }
            None => break,
        }
    }
    Ok(())
}

/// Skips an unknown fetch item value.
pub fn skip_fetch_item(lexer: &mut Lexer<'_>) -> Result<()> {
    // Skip space if present
    if lexer.peek() == Some(b' ') {
        lexer.advance();
    }

    // Skip the value (could be atom, string, list, or literal)
    let mut paren_depth = 0;

    loop {
        match lexer.peek() {
            Some(b'(') => {
                paren_depth += 1;
                lexer.advance();
            }
            Some(b')') => {
                if paren_depth == 0 {
                    break;
                }
                paren_depth -= 1;
                lexer.advance();
            }
            Some(b' ') if paren_depth == 0 => break,
            Some(_) => {
                lexer.advance();
            }
            None => break,
        }
    }

    Ok(())
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
    fn test_parse_fetch_uid_valid() {
        let data = b"(UID 123 FLAGS (\\Seen))";
        let mut lexer = Lexer::new(data);
        let items = parse_fetch_response(&mut lexer).unwrap();

        assert_eq!(items.len(), 2);
        assert!(matches!(items[0], FetchItem::Uid(_)));
    }

    #[test]
    fn test_parse_fetch_uid_zero_rejected() {
        let data = b"(UID 0)";
        let mut lexer = Lexer::new(data);
        let result = parse_fetch_response(&mut lexer);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UID"));
    }

    #[test]
    fn test_parse_fetch_flags() {
        let data = b"(FLAGS (\\Seen \\Flagged))";
        let mut lexer = Lexer::new(data);
        let items = parse_fetch_response(&mut lexer).unwrap();

        assert_eq!(items.len(), 1);
        assert!(matches!(items[0], FetchItem::Flags(_)));
    }

    #[test]
    fn test_parse_fetch_rfc822_size() {
        let data = b"(RFC822.SIZE 1234)";
        let mut lexer = Lexer::new(data);
        let items = parse_fetch_response(&mut lexer).unwrap();

        assert_eq!(items.len(), 1);
        if let FetchItem::Rfc822Size(size) = items[0] {
            assert_eq!(size, 1234);
        } else {
            panic!("Expected Rfc822Size");
        }
    }

    #[test]
    fn test_parse_fetch_modseq() {
        let data = b"(MODSEQ (12345))";
        let mut lexer = Lexer::new(data);
        let items = parse_fetch_response(&mut lexer).unwrap();

        assert_eq!(items.len(), 1);
        if let FetchItem::ModSeq(modseq) = items[0] {
            assert_eq!(modseq, 12345);
        } else {
            panic!("Expected ModSeq");
        }
    }

    #[test]
    fn test_parse_body_section_and_origin() {
        let data = b"[TEXT]<100>";
        let mut lexer = Lexer::new(data);
        let (section, origin) = parse_body_section_and_origin(&mut lexer).unwrap();

        assert_eq!(section, Some("TEXT".to_string()));
        assert_eq!(origin, Some(100));
    }

    #[test]
    fn test_parse_envelope() {
        let data = b"(\"date\" \"subject\" NIL NIL NIL NIL NIL NIL \"in-reply-to\" \"message-id\")";
        let mut lexer = Lexer::new(data);
        let envelope = parse_envelope(&mut lexer).unwrap();

        assert_eq!(envelope.date, Some("date".to_string()));
        assert_eq!(envelope.subject, Some("subject".to_string()));
        assert_eq!(envelope.in_reply_to, Some("in-reply-to".to_string()));
        assert_eq!(envelope.message_id, Some("message-id".to_string()));
    }
}
