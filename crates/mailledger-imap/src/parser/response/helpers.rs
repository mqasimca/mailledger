//! Parser helper functions.

use crate::parser::lexer::{Lexer, Token};
use crate::types::{
    Capability, Flag, Flags, ListResponse, Mailbox, MailboxAttribute, ResponseCode, SeqNum, Uid,
    UidValidity,
};
use crate::{Error, Result};

use super::types::StatusItem;

/// Parses a response code.
pub fn parse_response_code(lexer: &mut Lexer<'_>) -> Result<ResponseCode> {
    lexer.expect(Token::LBracket)?;

    let atom = lexer.read_atom_string()?;
    let upper = atom.to_uppercase();

    let code = match upper.as_str() {
        "ALERT" => ResponseCode::Alert,
        "PARSE" => ResponseCode::Parse,
        "READ-ONLY" => ResponseCode::ReadOnly,
        "READ-WRITE" => ResponseCode::ReadWrite,
        "TRYCREATE" => ResponseCode::TryCreate,
        "NOMODSEQ" => ResponseCode::NoModSeq,
        "UIDNEXT" => {
            lexer.expect_space()?;
            let n = lexer.read_number()?;
            let uid = Uid::new(n).ok_or_else(|| Error::Parse {
                position: lexer.position(),
                message: "Invalid UID 0".to_string(),
            })?;
            ResponseCode::UidNext(uid)
        }
        "UIDVALIDITY" => {
            lexer.expect_space()?;
            let n = lexer.read_number()?;
            let validity = UidValidity::new(n).ok_or_else(|| Error::Parse {
                position: lexer.position(),
                message: "Invalid UIDVALIDITY 0".to_string(),
            })?;
            ResponseCode::UidValidity(validity)
        }
        "UNSEEN" => {
            lexer.expect_space()?;
            let n = lexer.read_number()?;
            let seq = SeqNum::new(n).ok_or_else(|| Error::Parse {
                position: lexer.position(),
                message: "Invalid sequence number 0".to_string(),
            })?;
            ResponseCode::Unseen(seq)
        }
        "HIGHESTMODSEQ" => {
            lexer.expect_space()?;
            let n = u64::from(lexer.read_number()?);
            ResponseCode::HighestModSeq(n)
        }
        "CAPABILITY" => {
            let caps = parse_capability_data(lexer)?;
            ResponseCode::Capability(caps)
        }
        "PERMANENTFLAGS" => {
            lexer.expect_space()?;
            let flags = parse_flag_list(lexer)?;
            ResponseCode::PermanentFlags(flags.into_iter().collect())
        }
        _ => {
            // Skip until ]
            while lexer.peek() != Some(b']') && !lexer.is_eof() {
                lexer.advance();
            }
            ResponseCode::Unknown(atom.to_string())
        }
    };

    // Skip to closing bracket
    while lexer.peek() != Some(b']') && !lexer.is_eof() {
        lexer.advance();
    }
    lexer.expect(Token::RBracket)?;

    Ok(code)
}

/// Parses capability data.
pub fn parse_capability_data(lexer: &mut Lexer<'_>) -> Result<Vec<Capability>> {
    let mut caps = Vec::new();

    while lexer.peek() == Some(b' ') {
        lexer.advance();
        if let Token::Atom(s) = lexer.next_token()? {
            caps.push(Capability::parse(s));
        }
    }

    Ok(caps)
}

/// Parses a flag list.
pub fn parse_flag_list(lexer: &mut Lexer<'_>) -> Result<Flags> {
    lexer.expect(Token::LParen)?;

    let mut flags = Flags::new();

    loop {
        match lexer.next_token()? {
            Token::RParen => break,
            Token::Atom(s) => flags.insert(Flag::parse(s)),
            Token::Space => continue,
            token => {
                return Err(Error::Parse {
                    position: lexer.position(),
                    message: format!("Unexpected token in flag list: {token:?}"),
                });
            }
        }
    }

    Ok(flags)
}

/// Parses a LIST response.
pub fn parse_list_response(lexer: &mut Lexer<'_>) -> Result<ListResponse> {
    // Parse attributes
    lexer.expect(Token::LParen)?;
    let mut attributes = Vec::new();

    loop {
        match lexer.next_token()? {
            Token::RParen => break,
            Token::Atom(s) => attributes.push(MailboxAttribute::parse(s)),
            Token::Space => continue,
            token => {
                return Err(Error::Parse {
                    position: lexer.position(),
                    message: format!("Unexpected token in LIST attributes: {token:?}"),
                });
            }
        }
    }

    lexer.expect_space()?;

    // Parse delimiter
    let delimiter = match lexer.next_token()? {
        Token::Nil => None,
        Token::QuotedString(s) => s.chars().next(),
        token => {
            return Err(Error::Parse {
                position: lexer.position(),
                message: format!("Expected delimiter, got {token:?}"),
            });
        }
    };

    lexer.expect_space()?;

    // Parse mailbox name
    let mailbox_name = lexer.read_astring()?;

    Ok(ListResponse {
        attributes,
        delimiter,
        mailbox: Mailbox::new(mailbox_name),
    })
}

/// Parses a SEARCH response.
pub fn parse_search_response(lexer: &mut Lexer<'_>) -> Result<Vec<SeqNum>> {
    let mut nums = Vec::new();

    while lexer.peek() == Some(b' ') {
        lexer.advance();
        if let Token::Number(n) = lexer.next_token()?
            && let Some(seq) = SeqNum::new(n)
        {
            nums.push(seq);
        }
    }

    Ok(nums)
}

/// Parses a STATUS response.
pub fn parse_status_response(lexer: &mut Lexer<'_>) -> Result<(Mailbox, Vec<StatusItem>)> {
    let mailbox_name = lexer.read_astring()?;
    lexer.expect_space()?;
    lexer.expect(Token::LParen)?;

    let mut items = Vec::new();

    loop {
        match lexer.next_token()? {
            Token::RParen => break,
            Token::Space => continue,
            Token::Atom(name) => {
                lexer.expect_space()?;
                let value = lexer.read_number()?;

                let item = match name.to_uppercase().as_str() {
                    "MESSAGES" => StatusItem::Messages(value),
                    "RECENT" => StatusItem::Recent(value),
                    "UIDNEXT" => {
                        if let Some(uid) = Uid::new(value) {
                            StatusItem::UidNext(uid)
                        } else {
                            continue;
                        }
                    }
                    "UIDVALIDITY" => {
                        if let Some(v) = UidValidity::new(value) {
                            StatusItem::UidValidity(v)
                        } else {
                            continue;
                        }
                    }
                    "UNSEEN" => StatusItem::Unseen(value),
                    "HIGHESTMODSEQ" => StatusItem::HighestModSeq(u64::from(value)),
                    _ => continue,
                };
                items.push(item);
            }
            _ => continue,
        }
    }

    Ok((Mailbox::new(mailbox_name), items))
}

/// Reads text until CRLF.
pub fn read_text_until_crlf(lexer: &mut Lexer<'_>) -> String {
    let remaining = lexer.remaining();

    // Find CRLF
    let end = remaining
        .windows(2)
        .position(|w| w == b"\r\n")
        .unwrap_or(remaining.len());

    lexer.skip(end);

    // Skip CRLF if present
    if lexer.peek() == Some(b'\r') {
        lexer.skip(2);
    }

    String::from_utf8_lossy(&remaining[..end]).to_string()
}
