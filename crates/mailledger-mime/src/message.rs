//! MIME message structure and handling.

use crate::content_type::ContentType;
use crate::encoding::{decode_base64, decode_quoted_printable};
use crate::error::{Error, Result};
use crate::header::Headers;
use std::fmt;

/// Transfer encoding types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferEncoding {
    /// 7-bit ASCII.
    SevenBit,
    /// 8-bit binary.
    EightBit,
    /// Base64 encoding.
    Base64,
    /// Quoted-Printable encoding.
    QuotedPrintable,
    /// Binary (no encoding).
    Binary,
}

impl TransferEncoding {
    /// Parses transfer encoding from string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "8bit" => Self::EightBit,
            "base64" => Self::Base64,
            "quoted-printable" => Self::QuotedPrintable,
            "binary" => Self::Binary,
            _ => Self::SevenBit, // Default (includes "7bit")
        }
    }
}

impl fmt::Display for TransferEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SevenBit => write!(f, "7bit"),
            Self::EightBit => write!(f, "8bit"),
            Self::Base64 => write!(f, "base64"),
            Self::QuotedPrintable => write!(f, "quoted-printable"),
            Self::Binary => write!(f, "binary"),
        }
    }
}

/// MIME message part.
#[derive(Debug, Clone)]
pub struct Part {
    /// Part headers.
    pub headers: Headers,
    /// Part body (raw bytes).
    pub body: Vec<u8>,
}

impl Part {
    /// Creates a new part.
    #[must_use]
    pub const fn new(headers: Headers, body: Vec<u8>) -> Self {
        Self { headers, body }
    }

    /// Gets the content type.
    ///
    /// # Errors
    ///
    /// Returns an error if content type header is invalid.
    pub fn content_type(&self) -> Result<ContentType> {
        self.headers
            .get("content-type")
            .map_or_else(|| Ok(ContentType::text_plain()), ContentType::parse)
    }

    /// Gets the transfer encoding.
    #[must_use]
    pub fn transfer_encoding(&self) -> TransferEncoding {
        self.headers
            .get("content-transfer-encoding")
            .map_or(TransferEncoding::SevenBit, TransferEncoding::parse)
    }

    /// Decodes the body according to the transfer encoding.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails.
    pub fn decode_body(&self) -> Result<Vec<u8>> {
        match self.transfer_encoding() {
            TransferEncoding::Base64 => {
                let body_str = String::from_utf8_lossy(&self.body);
                // Remove whitespace for lenient parsing
                let cleaned: String = body_str.chars().filter(|c| !c.is_whitespace()).collect();
                decode_base64(&cleaned)
            }
            TransferEncoding::QuotedPrintable => {
                let body_str = String::from_utf8_lossy(&self.body);
                let decoded = decode_quoted_printable(&body_str)?;
                Ok(decoded.into_bytes())
            }
            _ => Ok(self.body.clone()),
        }
    }

    /// Gets the decoded body as a string.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding or UTF-8 conversion fails.
    pub fn body_text(&self) -> Result<String> {
        let decoded = self.decode_body()?;
        String::from_utf8(decoded).map_err(Into::into)
    }
}

/// MIME message.
#[derive(Debug, Clone)]
pub struct Message {
    /// Message headers.
    pub headers: Headers,
    /// Message parts (empty for single-part messages).
    pub parts: Vec<Part>,
    /// Body for single-part messages.
    pub body: Option<Vec<u8>>,
}

impl Message {
    /// Creates a new message.
    #[must_use]
    pub const fn new(headers: Headers) -> Self {
        Self {
            headers,
            parts: Vec::new(),
            body: None,
        }
    }

    /// Creates a single-part message.
    #[must_use]
    pub const fn single_part(headers: Headers, body: Vec<u8>) -> Self {
        Self {
            headers,
            parts: Vec::new(),
            body: Some(body),
        }
    }

    /// Creates a multipart message.
    #[must_use]
    pub const fn multipart(headers: Headers, parts: Vec<Part>) -> Self {
        Self {
            headers,
            parts,
            body: None,
        }
    }

    /// Gets the content type.
    ///
    /// # Errors
    ///
    /// Returns an error if content type header is invalid.
    pub fn content_type(&self) -> Result<ContentType> {
        self.headers
            .get("content-type")
            .map_or_else(|| Ok(ContentType::text_plain()), ContentType::parse)
    }

    /// Checks if this is a multipart message.
    ///
    /// # Errors
    ///
    /// Returns an error if content type cannot be determined.
    pub fn is_multipart(&self) -> Result<bool> {
        Ok(self.content_type()?.is_multipart())
    }

    /// Gets the From header.
    #[must_use]
    pub fn from(&self) -> Option<&str> {
        self.headers.get("from")
    }

    /// Gets the To header.
    #[must_use]
    pub fn to(&self) -> Option<&str> {
        self.headers.get("to")
    }

    /// Gets the Subject header.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.headers.get("subject")
    }

    /// Gets the Date header.
    #[must_use]
    pub fn date(&self) -> Option<&str> {
        self.headers.get("date")
    }

    /// Gets the Message-ID header.
    #[must_use]
    pub fn message_id(&self) -> Option<&str> {
        self.headers.get("message-id")
    }

    /// Gets the body as text for single-part messages.
    ///
    /// # Errors
    ///
    /// Returns an error if this is a multipart message or decoding fails.
    pub fn body_text(&self) -> Result<String> {
        if !self.parts.is_empty() {
            return Err(Error::InvalidMultipart(
                "Use parts for multipart messages".to_string(),
            ));
        }

        let body = self
            .body
            .as_ref()
            .ok_or_else(|| Error::Parse("No body".to_string()))?;

        // Decode based on transfer encoding
        let transfer_encoding = self
            .headers
            .get("content-transfer-encoding")
            .map_or(TransferEncoding::SevenBit, TransferEncoding::parse);

        let decoded = match transfer_encoding {
            TransferEncoding::Base64 => {
                let body_str = String::from_utf8_lossy(body);
                let cleaned: String = body_str.chars().filter(|c| !c.is_whitespace()).collect();
                decode_base64(&cleaned)?
            }
            TransferEncoding::QuotedPrintable => {
                let body_str = String::from_utf8_lossy(body);
                let decoded = decode_quoted_printable(&body_str)?;
                decoded.into_bytes()
            }
            _ => body.clone(),
        };

        String::from_utf8(decoded).map_err(Into::into)
    }

    /// Finds the first text/plain part in a multipart message.
    ///
    /// # Errors
    ///
    /// Returns an error if no text part is found or decoding fails.
    pub fn text_part(&self) -> Result<String> {
        for part in &self.parts {
            let ct = part.content_type()?;
            if ct.main_type == "text" && ct.sub_type == "plain" {
                return part.body_text();
            }
        }

        Err(Error::Parse("No text/plain part found".to_string()))
    }

    /// Finds the first text/html part in a multipart message.
    ///
    /// # Errors
    ///
    /// Returns an error if no HTML part is found or decoding fails.
    pub fn html_part(&self) -> Result<String> {
        for part in &self.parts {
            let ct = part.content_type()?;
            if ct.main_type == "text" && ct.sub_type == "html" {
                return part.body_text();
            }
        }

        Err(Error::Parse("No text/html part found".to_string()))
    }
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
    fn test_transfer_encoding_parse() {
        assert_eq!(TransferEncoding::parse("7bit"), TransferEncoding::SevenBit);
        assert_eq!(TransferEncoding::parse("base64"), TransferEncoding::Base64);
        assert_eq!(
            TransferEncoding::parse("quoted-printable"),
            TransferEncoding::QuotedPrintable
        );
    }

    #[test]
    fn test_part_new() {
        let headers = Headers::new();
        let body = b"Hello, World!".to_vec();
        let part = Part::new(headers, body);
        assert_eq!(part.body, b"Hello, World!");
    }

    #[test]
    fn test_part_body_text() {
        let mut headers = Headers::new();
        headers.add("content-type", "text/plain; charset=utf-8");
        let body = b"Hello, World!".to_vec();
        let part = Part::new(headers, body);

        let text = part.body_text().unwrap();
        assert_eq!(text, "Hello, World!");
    }

    #[test]
    fn test_message_single_part() {
        let mut headers = Headers::new();
        headers.add("from", "sender@example.com");
        headers.add("to", "recipient@example.com");
        headers.add("subject", "Test");

        let body = b"Hello, World!".to_vec();
        let message = Message::single_part(headers, body);

        assert_eq!(message.from(), Some("sender@example.com"));
        assert_eq!(message.to(), Some("recipient@example.com"));
        assert_eq!(message.subject(), Some("Test"));
        assert_eq!(message.body_text().unwrap(), "Hello, World!");
    }

    #[test]
    fn test_message_multipart() {
        let mut headers = Headers::new();
        headers.add("content-type", "multipart/mixed; boundary=abc123");

        let mut part1_headers = Headers::new();
        part1_headers.add("content-type", "text/plain");
        let part1 = Part::new(part1_headers, b"Part 1".to_vec());

        let mut part2_headers = Headers::new();
        part2_headers.add("content-type", "text/plain");
        let part2 = Part::new(part2_headers, b"Part 2".to_vec());

        let message = Message::multipart(headers, vec![part1, part2]);

        assert!(message.is_multipart().unwrap());
        assert_eq!(message.parts.len(), 2);
    }
}
