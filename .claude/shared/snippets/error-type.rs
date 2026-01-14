//! Error types for this crate.

use thiserror::Error;

/// Errors that can occur in this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // Add more variants as needed...
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;
