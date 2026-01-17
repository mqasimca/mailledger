//! MIME encoding and decoding utilities.
//!
//! Supports Base64, Quoted-Printable, and RFC 2047 header encoding.

use crate::error::{Error, Result};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use std::fmt::Write as _;

/// Encodes data as Base64.
#[must_use]
pub fn encode_base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Decodes Base64 data.
///
/// # Errors
///
/// Returns an error if the input is not valid Base64.
pub fn decode_base64(data: &str) -> Result<Vec<u8>> {
    STANDARD.decode(data).map_err(Into::into)
}

/// Maximum line length for Quoted-Printable encoding.
const MAX_LINE_LENGTH: usize = 76;

/// Encodes text using Quoted-Printable encoding (RFC 2045).
///
/// Encodes bytes that are not printable ASCII or would interfere
/// with email transmission.
#[must_use]
pub fn encode_quoted_printable(text: &str) -> String {
    let mut result = String::new();
    let mut line_length = 0;

    for byte in text.as_bytes() {
        // Check if we need soft line break
        if line_length >= MAX_LINE_LENGTH - 3 {
            result.push_str("=\r\n");
            line_length = 0;
        }

        match byte {
            // Printable ASCII except '=' and space (handle separately)
            b'!'..=b'<' | b'>'..=b'~' => {
                result.push(*byte as char);
                line_length += 1;
            }
            // Space needs special handling (encode at line end)
            b' ' => {
                if line_length >= MAX_LINE_LENGTH - 1 {
                    result.push_str("=20");
                    line_length += 3;
                } else {
                    result.push(' ');
                    line_length += 1;
                }
            }
            // Everything else gets encoded
            _ => {
                result.push('=');
                let _ = write!(result, "{byte:02X}");
                line_length += 3;
            }
        }
    }

    result
}

/// Decodes Quoted-Printable text (RFC 2045).
///
/// # Errors
///
/// Returns an error if the input contains invalid escape sequences.
pub fn decode_quoted_printable(text: &str) -> Result<String> {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '=' {
            // Soft line break
            if chars.peek() == Some(&'\r') {
                chars.next(); // consume \r
                if chars.peek() == Some(&'\n') {
                    chars.next(); // consume \n
                    continue;
                }
            } else if chars.peek() == Some(&'\n') {
                chars.next(); // consume \n
                continue;
            }

            // Hex encoded byte
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                let byte = u8::from_str_radix(&hex, 16)
                    .map_err(|e| Error::InvalidEncoding(format!("Invalid hex: {e}")))?;
                result.push(byte);
            } else {
                return Err(Error::InvalidEncoding(
                    "Incomplete escape sequence".to_string(),
                ));
            }
        } else {
            result.push(ch as u8);
        }
    }

    String::from_utf8(result).map_err(Into::into)
}

/// Encodes a header value using RFC 2047 encoding.
///
/// Format: `=?charset?encoding?encoded-text?=`
///
/// # Arguments
///
/// * `text` - Text to encode
/// * `charset` - Character set (e.g., "utf-8")
///
/// # Errors
///
/// Returns an error if encoding fails.
pub fn encode_rfc2047(text: &str, charset: &str) -> Result<String> {
    // Only encode if necessary (contains non-ASCII)
    if text.chars().all(|c| c.is_ascii() && c != '=' && c != '?') {
        return Ok(text.to_string());
    }

    // Use Base64 encoding (Q encoding is more complex)
    let encoded = encode_base64(text.as_bytes());
    Ok(format!("=?{charset}?B?{encoded}?="))
}

/// Decodes RFC 2047 encoded header value.
///
/// Format: `=?charset?encoding?encoded-text?=`
///
/// # Errors
///
/// Returns an error if the input is not valid RFC 2047 format.
pub fn decode_rfc2047(text: &str) -> Result<String> {
    // Check for RFC 2047 format
    if !text.starts_with("=?") || !text.ends_with("?=") {
        return Ok(text.to_string());
    }

    let inner = &text[2..text.len() - 2];
    let parts: Vec<&str> = inner.split('?').collect();

    if parts.len() != 3 {
        return Err(Error::InvalidEncoding(
            "Invalid RFC 2047 format".to_string(),
        ));
    }

    let encoding = parts[1].to_uppercase();
    let encoded_text = parts[2];

    match encoding.as_str() {
        "B" => {
            // Base64
            let decoded = decode_base64(encoded_text)?;
            String::from_utf8(decoded).map_err(Into::into)
        }
        "Q" => {
            // Quoted-Printable (with underscore for space)
            let text_with_spaces = encoded_text.replace('_', " ");
            decode_quoted_printable(&text_with_spaces)
        }
        _ => Err(Error::InvalidEncoding(format!(
            "Unknown encoding: {encoding}"
        ))),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_decode() {
        let data = b"Hello, World!";
        let encoded = encode_base64(data);
        assert_eq!(encoded, "SGVsbG8sIFdvcmxkIQ==");

        let decoded = decode_base64(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_quoted_printable_encode() {
        let text = "Hello, World!";
        let encoded = encode_quoted_printable(text);
        assert_eq!(encoded, "Hello, World!");

        let text = "Héllo, Wørld!";
        let encoded = encode_quoted_printable(text);
        assert!(encoded.contains("=C3"));
    }

    #[test]
    fn test_quoted_printable_decode() {
        let encoded = "Hello, World!";
        let decoded = decode_quoted_printable(encoded).unwrap();
        assert_eq!(decoded, "Hello, World!");

        let encoded = "H=C3=A9llo";
        let decoded = decode_quoted_printable(encoded).unwrap();
        assert_eq!(decoded, "Héllo");
    }

    #[test]
    fn test_quoted_printable_soft_line_break() {
        let encoded = "Hello=\r\nWorld";
        let decoded = decode_quoted_printable(encoded).unwrap();
        assert_eq!(decoded, "HelloWorld");
    }

    #[test]
    fn test_rfc2047_encode() {
        let text = "Hello";
        let encoded = encode_rfc2047(text, "utf-8").unwrap();
        assert_eq!(encoded, "Hello"); // No encoding needed

        let text = "Héllo";
        let encoded = encode_rfc2047(text, "utf-8").unwrap();
        assert!(encoded.starts_with("=?utf-8?B?"));
        assert!(encoded.ends_with("?="));
    }

    #[test]
    fn test_rfc2047_decode() {
        let encoded = "Hello";
        let decoded = decode_rfc2047(encoded).unwrap();
        assert_eq!(decoded, "Hello");

        let encoded = "=?utf-8?B?SMOpbGxv?=";
        let decoded = decode_rfc2047(encoded).unwrap();
        assert_eq!(decoded, "Héllo");
    }

    #[test]
    fn test_rfc2047_quoted_printable() {
        let encoded = "=?utf-8?Q?H=C3=A9llo?=";
        let decoded = decode_rfc2047(encoded).unwrap();
        assert_eq!(decoded, "Héllo");
    }
}
