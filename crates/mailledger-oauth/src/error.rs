//! Error types for `OAuth2` operations.

use std::io;

/// Result type alias for `OAuth2` operations.
pub type Result<T> = std::result::Result<T, Error>;

/// `OAuth2` error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// `OAuth2` error from server.
    #[error("OAuth2 error: {error} - {description}")]
    OAuth {
        /// Error code (e.g., `invalid_grant`).
        error: String,
        /// Human-readable description.
        description: String,
    },

    /// Token expired.
    #[error("Token expired")]
    TokenExpired,

    /// No refresh token available.
    #[error("No refresh token available")]
    NoRefreshToken,

    /// Invalid token response.
    #[error("Invalid token response: {0}")]
    InvalidResponse(String),

    /// Authorization timeout.
    #[error("Authorization timed out after {0} seconds")]
    Timeout(u64),

    /// User denied authorization.
    #[error("User denied authorization")]
    AccessDenied,

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// URL parsing error.
    #[error("URL error: {0}")]
    UrlError(#[from] url::ParseError),
}

impl Error {
    /// Creates an OAuth error from error code and description.
    #[must_use]
    pub fn oauth_error(error: impl Into<String>, description: impl Into<String>) -> Self {
        Self::OAuth {
            error: error.into(),
            description: description.into(),
        }
    }
}
