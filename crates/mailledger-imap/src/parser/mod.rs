//! IMAP protocol parser.
//!
//! This module provides a sans-I/O parser for IMAP server responses.
//! The parser is designed to be protocol-complete and handle all response
//! types defined in RFC 9051 (`IMAP4rev2`) and RFC 3501 (`IMAP4rev1`).
//!
//! # Architecture
//!
//! The parser is split into two main components:
//!
//! - **Lexer**: Tokenizes raw bytes into IMAP tokens (atoms, strings, numbers, etc.)
//! - **Response Parser**: Builds structured response objects from tokens
//!
//! # Example
//!
//! ```
//! use mailledger_imap::parser::{ResponseParser, Response, UntaggedResponse};
//!
//! let input = b"* OK IMAP4rev2 server ready\r\n";
//! let response = ResponseParser::parse(input).unwrap();
//!
//! match response {
//!     Response::Untagged(UntaggedResponse::Ok { text, .. }) => {
//!         assert!(text.contains("IMAP4rev2"));
//!     }
//!     _ => panic!("Expected untagged OK"),
//! }
//! ```

pub mod lexer;
pub mod response;

pub use lexer::{Lexer, Token};
pub use response::{
    Address, BodyStructure, Envelope, FetchItem, Response, ResponseParser, StatusItem,
    UntaggedResponse,
};
