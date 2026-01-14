//! Error types for the IMAP library.

use std::time::Duration;

use thiserror::Error;

/// Errors that can occur during IMAP operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error during network operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TLS handshake or encryption error.
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Invalid DNS name for TLS.
    #[error("Invalid DNS name: {0}")]
    InvalidDnsName(#[from] rustls::pki_types::InvalidDnsNameError),

    /// Protocol parsing error.
    #[error("Protocol error at position {position}: {message}")]
    Parse {
        /// Byte position where the error occurred.
        position: usize,
        /// Description of what went wrong.
        message: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Server returned NO response.
    #[error("Server returned NO: {0}")]
    No(String),

    /// Server returned BAD response.
    #[error("Server returned BAD: {0}")]
    Bad(String),

    /// Server sent BYE (disconnecting).
    #[error("Server sent BYE: {0}")]
    Bye(String),

    /// Operation timed out.
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    /// Invalid state for the requested operation.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Protocol violation or unexpected data.
    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;
