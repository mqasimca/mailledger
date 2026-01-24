//! MIME header handling.

use crate::encoding::{decode_rfc2047, encode_rfc2047};
use crate::error::Result;
use std::collections::HashMap;
use std::fmt;

/// Collection of email headers.
#[derive(Debug, Clone, Default)]
pub struct Headers {
    headers: HashMap<String, Vec<String>>,
}

impl Headers {
    /// Creates a new empty header collection.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a header value.
    pub fn add(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        let value = value.into();
        self.headers.entry(name).or_default().push(value);
    }

    /// Sets a header value, replacing any existing values.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_lowercase();
        let value = value.into();
        self.headers.insert(name, vec![value]);
    }

    /// Gets the first value for a header.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_lowercase())
            .and_then(|v| v.first().map(String::as_str))
    }

    /// Gets all values for a header.
    #[must_use]
    pub fn get_all(&self, name: &str) -> Vec<&str> {
        self.headers
            .get(&name.to_lowercase())
            .map(|v| v.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Removes all values for a header.
    pub fn remove(&mut self, name: &str) {
        self.headers.remove(&name.to_lowercase());
    }

    /// Returns an iterator over all headers.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers
            .iter()
            .flat_map(|(name, values)| values.iter().map(move |v| (name.as_str(), v.as_str())))
    }

    /// Parses headers from raw text.
    ///
    /// Headers are in the format:
    /// ```text
    /// Header-Name: value
    /// Continuation: line
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if header format is invalid.
    pub fn parse(text: &str) -> Result<Self> {
        let mut headers = Self::new();
        let mut current_name: Option<String> = None;
        let mut current_value = String::new();

        let lines: Vec<&str> = text.lines().collect();

        for line in &lines {
            if line.is_empty() {
                // Empty line signals end of headers, but save current header first
                if let Some(name) = current_name.take() {
                    headers.add(name, current_value.trim().to_string());
                }
                break;
            }

            // Check for continuation line (starts with space or tab)
            if line.starts_with(' ') || line.starts_with('\t') {
                if current_name.is_some() {
                    current_value.push(' ');
                    current_value.push_str(line.trim());
                }
            } else {
                // Save previous header if exists
                if let Some(name) = current_name.take() {
                    headers.add(name, current_value.trim().to_string());
                    current_value.clear();
                }

                // Parse new header
                if let Some((name, value)) = line.split_once(':') {
                    current_name = Some(name.trim().to_string());
                    current_value = value.trim().to_string();
                }
            }
        }

        // Save last header if we didn't hit an empty line
        if let Some(name) = current_name {
            headers.add(name, current_value.trim().to_string());
        }

        Ok(headers)
    }

    /// Encodes a header value using RFC 2047 if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails.
    pub fn encode_value(value: &str) -> Result<String> {
        encode_rfc2047(value, "utf-8")
    }

    /// Decodes a header value from RFC 2047 if encoded.
    ///
    /// # Errors
    ///
    /// Returns an error if decoding fails.
    pub fn decode_value(value: &str) -> Result<String> {
        decode_rfc2047(value)
    }
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut sorted_headers: Vec<_> = self.headers.iter().collect();
        sorted_headers.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (name, values) in sorted_headers {
            // Capitalize header name (e.g., "content-type" -> "Content-Type")
            let capitalized = name
                .split('-')
                .map(|part| {
                    let mut chars = part.chars();
                    chars.next().map_or_else(String::new, |first| {
                        first.to_uppercase().collect::<String>() + chars.as_str()
                    })
                })
                .collect::<Vec<_>>()
                .join("-");

            for value in values {
                writeln!(f, "{capitalized}: {value}")?;
            }
        }

        Ok(())
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
    fn test_headers_new() {
        let headers = Headers::new();
        assert!(headers.headers.is_empty());
    }

    #[test]
    fn test_headers_add_get() {
        let mut headers = Headers::new();
        headers.add("Content-Type", "text/plain");
        assert_eq!(headers.get("Content-Type"), Some("text/plain"));
        assert_eq!(headers.get("content-type"), Some("text/plain")); // Case insensitive
    }

    #[test]
    fn test_headers_set() {
        let mut headers = Headers::new();
        headers.add("To", "alice@example.com");
        headers.add("To", "bob@example.com");
        assert_eq!(headers.get_all("To").len(), 2);

        headers.set("To", "charlie@example.com");
        assert_eq!(headers.get_all("To").len(), 1);
        assert_eq!(headers.get("To"), Some("charlie@example.com"));
    }

    #[test]
    fn test_headers_remove() {
        let mut headers = Headers::new();
        headers.add("Subject", "Test");
        assert!(headers.get("Subject").is_some());

        headers.remove("Subject");
        assert!(headers.get("Subject").is_none());
    }

    #[test]
    fn test_headers_parse() {
        let text = concat!(
            "From: sender@example.com\r\n",
            "To: recipient@example.com\r\n",
            "Subject: Test Message\r\n",
            "Content-Type: text/plain;\r\n",
            " charset=utf-8\r\n",
            "\r\n"
        );

        let headers = Headers::parse(text).unwrap();
        assert_eq!(headers.get("From"), Some("sender@example.com"));
        assert_eq!(headers.get("To"), Some("recipient@example.com"));
        assert_eq!(headers.get("Subject"), Some("Test Message"));
        assert_eq!(
            headers.get("Content-Type"),
            Some("text/plain; charset=utf-8")
        );
    }

    #[test]
    fn test_headers_display() {
        let mut headers = Headers::new();
        headers.add("from", "sender@example.com");
        headers.add("to", "recipient@example.com");

        let s = headers.to_string();
        assert!(s.contains("From: sender@example.com"));
        assert!(s.contains("To: recipient@example.com"));
    }

    #[test]
    fn test_headers_iter() {
        let mut headers = Headers::new();
        headers.add("From", "sender@example.com");
        headers.add("To", "recipient@example.com");

        let mut count = 0;
        for (name, value) in headers.iter() {
            assert!(!name.is_empty());
            assert!(!value.is_empty());
            count += 1;
        }
        assert_eq!(count, 2);
    }
}
