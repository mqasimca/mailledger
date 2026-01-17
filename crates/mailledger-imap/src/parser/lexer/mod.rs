//! IMAP lexer for tokenizing server responses.
//!
//! This module implements a lexer for the IMAP protocol grammar defined in RFC 9051.
//! It breaks raw bytes into tokens that the parser can process.

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

mod token;

pub use token::Token;

use crate::{Error, Result};

/// IMAP lexer state.
pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input.
    #[must_use]
    pub const fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    /// Returns the current position in the input.
    #[must_use]
    pub const fn position(&self) -> usize {
        self.pos
    }

    /// Returns the remaining input.
    #[must_use]
    pub fn remaining(&self) -> &'a [u8] {
        &self.input[self.pos..]
    }

    /// Returns true if at end of input.
    #[must_use]
    pub const fn is_eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Peeks at the current byte without consuming it.
    #[must_use]
    pub fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Peeks at the byte at offset from current position.
    #[must_use]
    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        self.input.get(self.pos + offset).copied()
    }

    /// Advances by one byte and returns it.
    pub fn advance(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    /// Skips n bytes.
    pub fn skip(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.input.len());
    }

    /// Reads the next token.
    pub fn next_token(&mut self) -> Result<Token<'a>> {
        let Some(byte) = self.peek() else {
            return Ok(Token::Eof);
        };

        match byte {
            // CRLF
            b'\r' => {
                if self.peek_at(1) == Some(b'\n') {
                    self.skip(2);
                    Ok(Token::Crlf)
                } else {
                    Err(self.error("Expected LF after CR"))
                }
            }

            // Space
            b' ' => {
                self.advance();
                Ok(Token::Space)
            }

            // Special characters
            b'(' => {
                self.advance();
                Ok(Token::LParen)
            }
            b')' => {
                self.advance();
                Ok(Token::RParen)
            }
            b'[' => {
                self.advance();
                Ok(Token::LBracket)
            }
            b']' => {
                self.advance();
                Ok(Token::RBracket)
            }
            b'*' => {
                self.advance();
                Ok(Token::Asterisk)
            }
            b'+' => {
                self.advance();
                Ok(Token::Plus)
            }

            // Quoted string
            b'"' => self.read_quoted_string(),

            // Literal
            b'{' => self.read_literal_prefix(),

            // Number or atom starting with digit
            b'0'..=b'9' => self.read_number_or_atom(),

            // Atom (including NIL)
            _ if is_atom_char(byte) => self.read_atom(),

            // Invalid character
            _ => Err(self.error(&format!("Unexpected character: {byte:#04x}"))),
        }
    }

    /// Reads a quoted string token.
    fn read_quoted_string(&mut self) -> Result<Token<'a>> {
        self.advance(); // Skip opening quote

        let mut result = Vec::new();

        loop {
            match self.advance() {
                Some(b'"') => break,
                Some(b'\\') => {
                    // Escaped character
                    match self.advance() {
                        Some(b'"') => result.push(b'"'),
                        Some(b'\\') => result.push(b'\\'),
                        Some(c) => {
                            // In IMAP, only " and \ can be escaped
                            return Err(self.error(&format!("Invalid escape: \\{c}")));
                        }
                        None => return Err(self.error("Unexpected EOF in quoted string")),
                    }
                }
                Some(c) => result.push(c),
                None => return Err(self.error("Unexpected EOF in quoted string")),
            }
        }

        // Convert to string
        let s =
            String::from_utf8(result).map_err(|_| self.error("Invalid UTF-8 in quoted string"))?;

        Ok(Token::QuotedString(s))
    }

    /// Reads a literal size prefix {n}.
    fn read_literal_prefix(&mut self) -> Result<Token<'a>> {
        self.advance(); // Skip {

        let start = self.pos;

        // Check for + (LITERAL+)
        let mut literal_plus = false;

        while let Some(b) = self.peek() {
            match b {
                b'0'..=b'9' => {
                    self.advance();
                }
                b'+' => {
                    literal_plus = true;
                    self.advance();
                }
                b'}' => {
                    break;
                }
                _ => return Err(self.error("Invalid character in literal size")),
            }
        }

        let size_str = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| self.error("Invalid literal size"))?;

        let size_str = size_str.trim_end_matches('+');

        let size: usize = size_str
            .parse()
            .map_err(|_| self.error("Invalid literal size number"))?;

        // Skip }
        if self.advance() != Some(b'}') {
            return Err(self.error("Expected } after literal size"));
        }

        // For non-synchronizing literals (LITERAL+), we don't need CRLF
        // But for regular literals, we expect CRLF
        if literal_plus {
            // LITERAL+ may or may not have CRLF
            if self.peek() == Some(b'\r') && self.peek_at(1) == Some(b'\n') {
                self.skip(2);
            }
        } else if self.advance() != Some(b'\r') || self.advance() != Some(b'\n') {
            return Err(self.error("Expected CRLF after literal size"));
        }

        // Read literal data
        if self.pos + size > self.input.len() {
            return Err(self.error("Incomplete literal data"));
        }

        let data = self.input[self.pos..self.pos + size].to_vec();
        self.skip(size);

        Ok(Token::Literal(data))
    }

    /// Reads a number or atom starting with a digit.
    fn read_number_or_atom(&mut self) -> Result<Token<'a>> {
        let start = self.pos;

        // Check if it's all digits
        let mut all_digits = true;

        while let Some(b) = self.peek() {
            if is_atom_char(b) {
                if !b.is_ascii_digit() {
                    all_digits = false;
                }
                self.advance();
            } else {
                break;
            }
        }

        let s = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| self.error("Invalid UTF-8 in atom"))?;

        if all_digits {
            let n: u32 = s.parse().map_err(|_| self.error("Number too large"))?;
            Ok(Token::Number(n))
        } else {
            Ok(Token::Atom(s))
        }
    }

    /// Reads an atom token.
    fn read_atom(&mut self) -> Result<Token<'a>> {
        let start = self.pos;

        while let Some(b) = self.peek() {
            if is_atom_char(b) {
                self.advance();
            } else {
                break;
            }
        }

        let s = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| self.error("Invalid UTF-8 in atom"))?;

        // Check for NIL
        if s.eq_ignore_ascii_case("NIL") {
            Ok(Token::Nil)
        } else {
            Ok(Token::Atom(s))
        }
    }

    /// Creates a parse error at the current position.
    fn error(&self, message: &str) -> Error {
        Error::Parse {
            position: self.pos,
            message: message.to_string(),
        }
    }

    /// Expects and consumes a specific token.
    #[allow(clippy::needless_pass_by_value)]
    pub fn expect(&mut self, expected: Token<'_>) -> Result<()> {
        let token = self.next_token()?;
        if std::mem::discriminant(&token) == std::mem::discriminant(&expected) {
            Ok(())
        } else {
            Err(self.error(&format!("Expected {expected:?}, got {token:?}")))
        }
    }

    /// Expects and consumes a space.
    pub fn expect_space(&mut self) -> Result<()> {
        self.expect(Token::Space)
    }

    /// Expects and consumes CRLF.
    pub fn expect_crlf(&mut self) -> Result<()> {
        self.expect(Token::Crlf)
    }

    /// Reads an astring (atom or string).
    pub fn read_astring(&mut self) -> Result<String> {
        match self.next_token()? {
            Token::Atom(s) => Ok(s.to_string()),
            Token::QuotedString(s) => Ok(s),
            Token::Literal(data) => {
                String::from_utf8(data).map_err(|_| self.error("Invalid UTF-8 in literal"))
            }
            token => Err(self.error(&format!("Expected astring, got {token:?}"))),
        }
    }

    /// Reads a nstring (NIL or string).
    pub fn read_nstring(&mut self) -> Result<Option<String>> {
        match self.next_token()? {
            Token::Nil => Ok(None),
            Token::QuotedString(s) => Ok(Some(s)),
            Token::Literal(data) => {
                let s =
                    String::from_utf8(data).map_err(|_| self.error("Invalid UTF-8 in literal"))?;
                Ok(Some(s))
            }
            token => Err(self.error(&format!("Expected nstring, got {token:?}"))),
        }
    }

    /// Reads a number.
    pub fn read_number(&mut self) -> Result<u32> {
        match self.next_token()? {
            Token::Number(n) => Ok(n),
            token => Err(self.error(&format!("Expected number, got {token:?}"))),
        }
    }

    /// Reads an atom.
    pub fn read_atom_string(&mut self) -> Result<&'a str> {
        match self.next_token()? {
            Token::Atom(s) => Ok(s),
            token => Err(self.error(&format!("Expected atom, got {token:?}"))),
        }
    }

    /// Skips optional spaces.
    pub fn skip_spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.advance();
        }
    }
}

/// Returns true if the byte is a valid atom character.
///
/// Note: This includes `\` to handle flags like `\Seen` as single tokens,
/// even though RFC 9051 technically defines `\` as a quoted-special.
#[must_use]
pub const fn is_atom_char(b: u8) -> bool {
    // IMAP atom chars are any CHAR except atom-specials
    // atom-specials = "(" / ")" / "{" / SP / CTL / list-wildcards / quoted-specials / resp-specials
    // list-wildcards = "%" / "*"
    // quoted-specials = DQUOTE / "\"
    // resp-specials = "]"
    //
    // Note: We include "\" to handle flags like \Seen as single tokens

    matches!(b,
        0x21..=0x27 |  // ! " # $ % & '  (but not " which is 0x22)
        0x2B..=0x5A |  // + , - . / 0-9 : ; < = > ? @ A-Z
        0x5C |         // \ (for flags like \Seen)
        0x5E..=0x7A |  // ^ _ ` a-z
        0x7C |         // |
        0x7E           // ~
    ) && b != b'"'
        && b != b'%'
}

/// Returns true if the byte is an atom special character.
#[must_use]
pub const fn is_atom_special(b: u8) -> bool {
    matches!(
        b,
        b'(' | b')' | b'{' | b' ' | b'%' | b'*' | b'"' | b'\\' | b']'
    ) || b < 0x20
        || b == 0x7F
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new(b"* OK");

        assert_eq!(lexer.next_token().unwrap(), Token::Asterisk);
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("OK"));
        assert_eq!(lexer.next_token().unwrap(), Token::Eof);
    }

    #[test]
    fn test_tagged_response() {
        let mut lexer = Lexer::new(b"A001 OK LOGIN completed\r\n");

        assert_eq!(lexer.next_token().unwrap(), Token::Atom("A001"));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("OK"));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("LOGIN"));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("completed"));
        assert_eq!(lexer.next_token().unwrap(), Token::Crlf);
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new(b"123 456");

        assert_eq!(lexer.next_token().unwrap(), Token::Number(123));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(456));
    }

    #[test]
    fn test_quoted_string() {
        let mut lexer = Lexer::new(b"\"hello world\"");

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("hello world".to_string())
        );
    }

    #[test]
    fn test_quoted_string_escaped() {
        let mut lexer = Lexer::new(b"\"hello \\\"world\\\"\"");

        assert_eq!(
            lexer.next_token().unwrap(),
            Token::QuotedString("hello \"world\"".to_string())
        );
    }

    #[test]
    fn test_nil() {
        let mut lexer = Lexer::new(b"NIL nil Nil");

        assert_eq!(lexer.next_token().unwrap(), Token::Nil);
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Nil);
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Nil);
    }

    #[test]
    fn test_parentheses() {
        let mut lexer = Lexer::new(b"(\\Seen \\Flagged)");

        assert_eq!(lexer.next_token().unwrap(), Token::LParen);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("\\Seen"));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("\\Flagged"));
        assert_eq!(lexer.next_token().unwrap(), Token::RParen);
    }

    #[test]
    fn test_brackets() {
        let mut lexer = Lexer::new(b"[UIDNEXT 100]");

        assert_eq!(lexer.next_token().unwrap(), Token::LBracket);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("UIDNEXT"));
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Number(100));
        assert_eq!(lexer.next_token().unwrap(), Token::RBracket);
    }

    #[test]
    fn test_literal() {
        let mut lexer = Lexer::new(b"{5}\r\nhello");

        match lexer.next_token().unwrap() {
            Token::Literal(data) => assert_eq!(data, b"hello"),
            other => panic!("Expected literal, got {other:?}"),
        }
    }

    #[test]
    fn test_continuation() {
        let mut lexer = Lexer::new(b"+ Ready\r\n");

        assert_eq!(lexer.next_token().unwrap(), Token::Plus);
        assert_eq!(lexer.next_token().unwrap(), Token::Space);
        assert_eq!(lexer.next_token().unwrap(), Token::Atom("Ready"));
        assert_eq!(lexer.next_token().unwrap(), Token::Crlf);
    }

    #[test]
    fn test_is_atom_char() {
        assert!(is_atom_char(b'A'));
        assert!(is_atom_char(b'z'));
        assert!(is_atom_char(b'0'));
        assert!(is_atom_char(b':'));
        assert!(is_atom_char(b'\\'));
        assert!(!is_atom_char(b' '));
        assert!(!is_atom_char(b'('));
        assert!(!is_atom_char(b')'));
        assert!(!is_atom_char(b'{'));
    }
}
