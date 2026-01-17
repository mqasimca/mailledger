//! `OAuth2` token types and management.

use crate::error::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// `OAuth2` access token with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Access token string.
    pub access_token: String,
    /// Token type (usually "Bearer").
    pub token_type: String,
    /// Expiration time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Refresh token for obtaining new access tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Scope granted by authorization server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl Token {
    /// Creates a new token.
    #[must_use]
    pub fn new(access_token: impl Into<String>, token_type: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            token_type: token_type.into(),
            expires_at: None,
            refresh_token: None,
            scope: None,
        }
    }

    /// Creates a token from token response.
    ///
    /// # Errors
    ///
    /// Returns an error if the response is invalid.
    pub fn from_response(response: TokenResponse) -> Result<Self> {
        let expires_at = response
            .expires_in
            .map(|secs| Utc::now() + Duration::seconds(i64::from(secs)));

        Ok(Self {
            access_token: response.access_token,
            token_type: response.token_type,
            expires_at,
            refresh_token: response.refresh_token,
            scope: response.scope,
        })
    }

    /// Checks if the token is expired (with 60 second buffer).
    #[must_use]
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|exp| Utc::now() + Duration::seconds(60) >= exp)
    }

    /// Returns true if the token is valid (not expired).
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Sets the refresh token.
    #[must_use]
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    /// Sets the expiration time.
    #[must_use]
    pub const fn with_expires_at(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Sets the scope.
    #[must_use]
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Returns the refresh token if available.
    ///
    /// # Errors
    ///
    /// Returns an error if no refresh token is available.
    pub fn refresh_token(&self) -> Result<&str> {
        self.refresh_token.as_deref().ok_or(Error::NoRefreshToken)
    }
}

/// Token response from `OAuth2` server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenResponse {
    /// Access token.
    pub access_token: String,
    /// Token type.
    pub token_type: String,
    /// Expires in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u32>,
    /// Refresh token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Error response from `OAuth2` server.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    /// Error code.
    pub error: String,
    /// Error description.
    #[serde(default)]
    pub error_description: String,
}

impl ErrorResponse {
    /// Converts to an Error.
    #[must_use]
    pub fn into_error(self) -> Error {
        Error::oauth_error(self.error, self.error_description)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let token = Token::new("access123", "Bearer");
        assert_eq!(token.access_token, "access123");
        assert_eq!(token.token_type, "Bearer");
        assert!(token.expires_at.is_none());
        assert!(token.refresh_token.is_none());
    }

    #[test]
    fn test_token_with_refresh() {
        let token = Token::new("access123", "Bearer").with_refresh_token("refresh456");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh456"));
    }

    #[test]
    fn test_token_expiration() {
        let expired =
            Token::new("access123", "Bearer").with_expires_at(Utc::now() - Duration::seconds(120));
        assert!(expired.is_expired());
        assert!(!expired.is_valid());

        let valid =
            Token::new("access123", "Bearer").with_expires_at(Utc::now() + Duration::seconds(3600));
        assert!(!valid.is_expired());
        assert!(valid.is_valid());
    }

    #[test]
    fn test_token_from_response() {
        let response = TokenResponse {
            access_token: "test_token".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: Some(3600),
            refresh_token: Some("refresh".to_string()),
            scope: Some("email".to_string()),
        };

        let token = Token::from_response(response).unwrap();
        assert_eq!(token.access_token, "test_token");
        assert!(token.expires_at.is_some());
        assert!(token.is_valid());
    }
}
