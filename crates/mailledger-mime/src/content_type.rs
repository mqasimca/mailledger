//! MIME content type handling.

use crate::error::{Error, Result};
use std::collections::HashMap;
use std::fmt;

/// MIME content type with parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentType {
    /// Main type (e.g., "text", "image", "multipart").
    pub main_type: String,
    /// Subtype (e.g., "plain", "html", "jpeg").
    pub sub_type: String,
    /// Parameters (e.g., charset=utf-8, boundary=xxx).
    pub parameters: HashMap<String, String>,
}

impl ContentType {
    /// Creates a new content type.
    #[must_use]
    pub fn new(main_type: impl Into<String>, sub_type: impl Into<String>) -> Self {
        Self {
            main_type: main_type.into(),
            sub_type: sub_type.into(),
            parameters: HashMap::new(),
        }
    }

    /// Creates a text/plain content type.
    #[must_use]
    pub fn text_plain() -> Self {
        let mut ct = Self::new("text", "plain");
        ct.parameters
            .insert("charset".to_string(), "utf-8".to_string());
        ct
    }

    /// Creates a text/html content type.
    #[must_use]
    pub fn text_html() -> Self {
        let mut ct = Self::new("text", "html");
        ct.parameters
            .insert("charset".to_string(), "utf-8".to_string());
        ct
    }

    /// Creates a multipart/mixed content type with boundary.
    #[must_use]
    pub fn multipart_mixed(boundary: impl Into<String>) -> Self {
        let mut ct = Self::new("multipart", "mixed");
        ct.parameters
            .insert("boundary".to_string(), boundary.into());
        ct
    }

    /// Creates a multipart/alternative content type with boundary.
    #[must_use]
    pub fn multipart_alternative(boundary: impl Into<String>) -> Self {
        let mut ct = Self::new("multipart", "alternative");
        ct.parameters
            .insert("boundary".to_string(), boundary.into());
        ct
    }

    /// Creates a multipart/related content type with boundary.
    #[must_use]
    pub fn multipart_related(boundary: impl Into<String>) -> Self {
        let mut ct = Self::new("multipart", "related");
        ct.parameters
            .insert("boundary".to_string(), boundary.into());
        ct
    }

    /// Adds a parameter.
    #[must_use]
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }

    /// Returns the charset parameter if present.
    #[must_use]
    pub fn charset(&self) -> Option<&str> {
        self.parameters.get("charset").map(String::as_str)
    }

    /// Returns the boundary parameter if present.
    #[must_use]
    pub fn boundary(&self) -> Option<&str> {
        self.parameters.get("boundary").map(String::as_str)
    }

    /// Checks if this is a multipart content type.
    #[must_use]
    pub fn is_multipart(&self) -> bool {
        self.main_type.eq_ignore_ascii_case("multipart")
    }

    /// Checks if this is a text content type.
    #[must_use]
    pub fn is_text(&self) -> bool {
        self.main_type.eq_ignore_ascii_case("text")
    }

    /// Parses a content type string.
    ///
    /// Format: `type/subtype; param1=value1; param2=value2`
    ///
    /// # Errors
    ///
    /// Returns an error if the format is invalid.
    pub fn parse(s: &str) -> Result<Self> {
        let mut parts = s.split(';');

        // Parse type/subtype
        let type_str = parts
            .next()
            .ok_or_else(|| Error::InvalidContentType("Empty content type".to_string()))?
            .trim();

        let mut type_parts = type_str.split('/');
        let main_type = type_parts
            .next()
            .ok_or_else(|| Error::InvalidContentType("Missing main type".to_string()))?
            .trim()
            .to_lowercase();

        let sub_type = type_parts
            .next()
            .ok_or_else(|| Error::InvalidContentType("Missing subtype".to_string()))?
            .trim()
            .to_lowercase();

        let mut content_type = Self::new(main_type, sub_type);

        // Parse parameters
        for param in parts {
            let param = param.trim();
            if let Some((key, value)) = param.split_once('=') {
                let key = key.trim().to_lowercase();
                let value = value.trim().trim_matches('"').to_string();
                content_type.parameters.insert(key, value);
            }
        }

        Ok(content_type)
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let main = &self.main_type;
        let sub = &self.sub_type;
        write!(f, "{main}/{sub}")?;

        for (key, value) in &self.parameters {
            // Quote value if it contains special characters
            if value.contains(|c: char| c.is_whitespace() || "()<>@,;:\\\"/[]?=".contains(c)) {
                write!(f, "; {key}=\"{value}\"")?;
            } else {
                write!(f, "; {key}={value}")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_new() {
        let ct = ContentType::new("text", "plain");
        assert_eq!(ct.main_type, "text");
        assert_eq!(ct.sub_type, "plain");
        assert!(ct.parameters.is_empty());
    }

    #[test]
    fn test_text_plain() {
        let ct = ContentType::text_plain();
        assert_eq!(ct.main_type, "text");
        assert_eq!(ct.sub_type, "plain");
        assert_eq!(ct.charset(), Some("utf-8"));
    }

    #[test]
    fn test_multipart_mixed() {
        let ct = ContentType::multipart_mixed("boundary123");
        assert_eq!(ct.main_type, "multipart");
        assert_eq!(ct.sub_type, "mixed");
        assert_eq!(ct.boundary(), Some("boundary123"));
        assert!(ct.is_multipart());
    }

    #[test]
    fn test_content_type_parse() {
        let ct = ContentType::parse("text/plain; charset=utf-8").unwrap();
        assert_eq!(ct.main_type, "text");
        assert_eq!(ct.sub_type, "plain");
        assert_eq!(ct.charset(), Some("utf-8"));
    }

    #[test]
    fn test_content_type_parse_quoted() {
        let ct = ContentType::parse("multipart/mixed; boundary=\"----=_Part_123\"").unwrap();
        assert_eq!(ct.main_type, "multipart");
        assert_eq!(ct.sub_type, "mixed");
        assert_eq!(ct.boundary(), Some("----=_Part_123"));
    }

    #[test]
    fn test_content_type_display() {
        let ct = ContentType::text_plain();
        let s = ct.to_string();
        assert!(s.contains("text/plain"));
        assert!(s.contains("charset=utf-8"));
    }

    #[test]
    fn test_content_type_with_parameter() {
        let ct = ContentType::new("text", "plain")
            .with_parameter("charset", "iso-8859-1")
            .with_parameter("format", "flowed");

        assert_eq!(ct.charset(), Some("iso-8859-1"));
        assert_eq!(ct.parameters.get("format"), Some(&"flowed".to_string()));
    }
}
