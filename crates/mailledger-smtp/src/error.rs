//! Error types for SMTP operations.

use std::io;

/// Result type alias for SMTP operations.
pub type Result<T> = std::result::Result<T, Error>;

/// SMTP error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// TLS error.
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Server returned error response.
    #[error("SMTP error {code}: {message}")]
    SmtpError {
        /// Reply code (e.g., 550).
        code: u16,
        /// Error message from server.
        message: String,
    },

    /// Protocol error (unexpected response).
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Invalid email address.
    #[error("Invalid email address: {0}")]
    InvalidAddress(String),

    /// Connection already authenticated.
    #[error("Connection already authenticated")]
    AlreadyAuthenticated,

    /// Authentication required.
    #[error("Authentication required")]
    AuthRequired,

    /// Message too large.
    #[error("Message exceeds size limit: {0} bytes")]
    MessageTooLarge(usize),

    /// Feature not supported by server.
    #[error("Server does not support {0}")]
    NotSupported(String),

    /// Invalid state for operation.
    #[error("Invalid state for operation: {0}")]
    InvalidState(String),
}

impl Error {
    /// Creates an SMTP error from a reply code and message.
    #[must_use]
    pub fn smtp_error(code: u16, message: impl Into<String>) -> Self {
        Self::SmtpError {
            code,
            message: message.into(),
        }
    }

    /// Returns true if this is a permanent error (5xx).
    #[must_use]
    pub const fn is_permanent(&self) -> bool {
        matches!(self, Self::SmtpError { code, .. } if *code >= 500 && *code < 600)
    }

    /// Returns true if this is a transient error (4xx).
    #[must_use]
    pub const fn is_transient(&self) -> bool {
        matches!(self, Self::SmtpError { code, .. } if *code >= 400 && *code < 500)
    }
}
