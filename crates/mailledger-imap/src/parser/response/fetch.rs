//! FETCH response parsing.

use crate::parser::lexer::{Lexer, Token};
use crate::types::Uid;
use crate::{Error, Result};

use super::parse_flag_list;
use super::types::{Address, Envelope, FetchItem};

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
                        if let Some(uid) = Uid::new(n) {
                            items.push(FetchItem::Uid(uid));
                        }
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
                    "BODY" | "BODY[]" => {
                        // Simple body fetch
                        let section = if let Some(start) = name.find('[') {
                            let end = name.find(']').unwrap_or(name.len());
                            Some(name[start + 1..end].to_string())
                        } else {
                            None
                        };

                        lexer.expect_space()?;
                        let data = match lexer.next_token()? {
                            Token::Literal(d) => Some(d),
                            Token::Nil => None,
                            _ => None,
                        };

                        items.push(FetchItem::Body {
                            section,
                            origin: None,
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
