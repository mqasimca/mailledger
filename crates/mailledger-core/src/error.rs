//! Error types for the core library.

use thiserror::Error;

/// Errors that can occur in core operations.
#[derive(Debug, Error)]
pub enum Error {
    /// IMAP operation failed.
    #[error("IMAP error: {0}")]
    Imap(#[from] mailledger_imap::Error),

    /// Database operation failed.
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Account not found.
    #[error("Account not found: {0}")]
    AccountNotFound(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Credential storage error.
    #[error("Credential error: {0}")]
    Credential(#[from] crate::account::credentials::CredentialError),
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;
