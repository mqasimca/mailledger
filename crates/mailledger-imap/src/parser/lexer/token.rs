//! IMAP token types.

/// Token types produced by the lexer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'a> {
    /// Atom (unquoted string without special characters).
    Atom(&'a str),
    /// Quoted string.
    QuotedString(String),
    /// Literal string with size prefix {n}.
    Literal(Vec<u8>),
    /// Number.
    Number(u32),
    /// Opening parenthesis.
    LParen,
    /// Closing parenthesis.
    RParen,
    /// Opening bracket.
    LBracket,
    /// Closing bracket.
    RBracket,
    /// Space character.
    Space,
    /// Asterisk (untagged response prefix).
    Asterisk,
    /// Plus (continuation response prefix).
    Plus,
    /// NIL literal.
    Nil,
    /// CRLF line ending.
    Crlf,
    /// End of input.
    Eof,
}
