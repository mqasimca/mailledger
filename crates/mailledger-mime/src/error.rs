//! Error types for MIME operations.

use std::string::FromUtf8Error;

/// Result type alias for MIME operations.
pub type Result<T> = std::result::Result<T, Error>;

/// MIME error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid MIME header.
    #[error("Invalid MIME header: {0}")]
    InvalidHeader(String),

    /// Invalid content type.
    #[error("Invalid content type: {0}")]
    InvalidContentType(String),

    /// Invalid encoding.
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    /// Base64 decode error.
    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    /// UTF-8 decode error.
    #[error("UTF-8 decode error: {0}")]
    Utf8Decode(#[from] FromUtf8Error),

    /// Missing boundary in multipart message.
    #[error("Missing boundary in multipart message")]
    MissingBoundary,

    /// Invalid multipart structure.
    #[error("Invalid multipart structure: {0}")]
    InvalidMultipart(String),

    /// Missing required header.
    #[error("Missing required header: {0}")]
    MissingHeader(String),

    /// Parse error.
    #[error("Parse error: {0}")]
    Parse(String),
}
