//! SASL authentication mechanisms.
//!
//! Implements:
//! - PLAIN (RFC 4616) - Basic username/password authentication
//! - OAUTHBEARER (RFC 7628) - Standard `OAuth2` authentication
//! - XOAUTH2 (Google/Microsoft proprietary) - Legacy `OAuth2` authentication

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// Generates PLAIN initial response (RFC 4616).
///
/// Format: `\0<username>\0<password>` (base64 encoded)
///
/// The PLAIN mechanism sends credentials as: authorization-id (empty),
/// authentication-id (username), and password, separated by NUL bytes.
///
/// # Arguments
///
/// * `username` - Authentication username
/// * `password` - Authentication password
///
/// # Example
///
/// ```
/// use mailledger_oauth::sasl::plain_response;
///
/// let response = plain_response("user@example.com", "password123");
/// // Can be used with IMAP AUTHENTICATE PLAIN or SMTP AUTH PLAIN
/// ```
#[must_use]
pub fn plain_response(username: &str, password: &str) -> String {
    // Format: \0username\0password
    // The first NUL is for the authorization identity (empty = same as auth identity)
    let auth_string = format!("\0{username}\0{password}");
    STANDARD.encode(auth_string.as_bytes())
}

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
#[allow(
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_string_new,
    clippy::needless_collect,
    clippy::unreadable_literal,
    clippy::used_underscore_items,
    clippy::similar_names
)]
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

    #[test]
    fn test_plain_response() {
        let response = plain_response("user@example.com", "password123");
        let decoded = STANDARD.decode(&response).unwrap();

        // Format should be \0username\0password
        assert_eq!(decoded[0], 0); // First NUL (empty authz-id)
        assert!(decoded.contains(&b'@'));
        // Find second NUL
        let second_nul = decoded.iter().skip(1).position(|&b| b == 0).unwrap() + 1;
        assert!(second_nul > 1);
    }

    #[test]
    fn test_plain_response_format() {
        let response = plain_response("test", "pass");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        // Check exact format per RFC 4616
        assert_eq!(decoded_str, "\0test\0pass");
    }

    #[test]
    fn test_plain_response_special_chars() {
        // Password with special characters
        let response = plain_response("user", "pass@word!");
        let decoded = STANDARD.decode(&response).unwrap();
        let decoded_str = String::from_utf8(decoded).unwrap();

        assert_eq!(decoded_str, "\0user\0pass@word!");
    }
}
