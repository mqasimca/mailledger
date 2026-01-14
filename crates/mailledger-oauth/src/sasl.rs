//! SASL `OAuth2` authentication mechanisms.
//!
//! Implements OAUTHBEARER (RFC 7628) and XOAUTH2 (Google/Microsoft proprietary).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// Generates OAUTHBEARER initial response (RFC 7628).
///
/// Format: `n,a=<user>,\x01auth=Bearer <token>\x01\x01`
///
/// # Arguments
///
/// * `user` - User email address
/// * `token` - `OAuth2` access token
///
/// # Example
///
/// ```
/// use mailledger_oauth::sasl::oauthbearer_response;
///
/// let response = oauthbearer_response("user@example.com", "ya29.a0...");
/// // Can be used with IMAP AUTHENTICATE OAUTHBEARER or SMTP AUTH OAUTHBEARER
/// ```
#[must_use]
pub fn oauthbearer_response(user: &str, token: &str) -> String {
    let auth_string = format!("n,a={user},\x01auth=Bearer {token}\x01\x01");
    STANDARD.encode(auth_string.as_bytes())
}

/// Generates XOAUTH2 initial response (Google/Microsoft proprietary).
///
/// Format: `user=<user>\x01auth=Bearer <token>\x01\x01`
///
/// # Arguments
///
/// * `user` - User email address
/// * `token` - `OAuth2` access token
///
/// # Example
///
/// ```
/// use mailledger_oauth::sasl::xoauth2_response;
///
/// let response = xoauth2_response("user@example.com", "ya29.a0...");
/// // Can be used with IMAP AUTHENTICATE XOAUTH2 or SMTP AUTH XOAUTH2
/// ```
#[must_use]
pub fn xoauth2_response(user: &str, token: &str) -> String {
    let auth_string = format!("user={user}\x01auth=Bearer {token}\x01\x01");
    STANDARD.encode(auth_string.as_bytes())
}

/// Parses an `OAuth2` error response from the server.
///
/// `OAuth2` errors are JSON-encoded: `{"status":"401", "schemes":"bearer", "scope":"..."}`
///
/// # Errors
///
/// Returns an error if the response cannot be parsed.
pub fn parse_oauth_error(response: &str) -> Result<OAuthError, serde_json::Error> {
    serde_json::from_str(response)
}

/// `OAuth2` error response from server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OAuthError {
    /// HTTP status code.
    pub status: String,
    /// Authentication schemes supported.
    pub schemes: String,
    /// `OAuth2` scope required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauthbearer_response() {
        let response = oauthbearer_response("user@example.com", "token123");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        assert!(decoded_str.starts_with("n,a=user@example.com"));
        assert!(decoded_str.contains("auth=Bearer token123"));
        assert!(decoded_str.ends_with("\x01\x01"));
    }

    #[test]
    fn test_xoauth2_response() {
        let response = xoauth2_response("user@example.com", "token123");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        assert!(decoded_str.starts_with("user=user@example.com"));
        assert!(decoded_str.contains("auth=Bearer token123"));
        assert!(decoded_str.ends_with("\x01\x01"));
    }

    #[test]
    fn test_oauthbearer_format() {
        let response = oauthbearer_response("test@test.com", "abc");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        // Check exact format per RFC 7628
        assert_eq!(decoded_str, "n,a=test@test.com,\x01auth=Bearer abc\x01\x01");
    }

    #[test]
    fn test_xoauth2_format() {
        let response = xoauth2_response("test@test.com", "abc");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        // Check exact XOAUTH2 format
        assert_eq!(decoded_str, "user=test@test.com\x01auth=Bearer abc\x01\x01");
    }

    #[test]
    fn test_parse_oauth_error() {
        let json = r#"{"status":"401","schemes":"bearer","scope":"https://mail.google.com/"}"#;
        let error = parse_oauth_error(json).unwrap();

        assert_eq!(error.status, "401");
        assert_eq!(error.schemes, "bearer");
        assert_eq!(error.scope.as_deref(), Some("https://mail.google.com/"));
    }

    #[test]
    fn test_responses_are_base64() {
        let response = oauthbearer_response("user@example.com", "token");
        // Should not contain raw text, only base64 characters
        assert!(!response.contains("user@example.com"));
        assert!(!response.contains("token"));
        assert!(STANDARD.decode(&response).is_ok());
    }
}
