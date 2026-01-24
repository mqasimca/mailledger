//! Transmit types for outgoing protocol data.
//!
// Allow missing_const_for_fn since Vec methods aren't const in stable Rust.
#![allow(clippy::missing_const_for_fn)]
//!
//! This module defines types for data that needs to be sent to the server.

/// Data to transmit to the server.
///
/// In a sans-I/O architecture, the protocol layer produces these transmit
/// structures, and the I/O layer is responsible for actually sending them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transmit {
    /// Raw bytes to send to the server.
    pub data: Vec<u8>,
}

impl Transmit {
    /// Creates a new transmit from bytes.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Creates a new transmit from a string.
    #[must_use]
    pub fn from_string(s: String) -> Self {
        Self {
            data: s.into_bytes(),
        }
    }

    /// Returns the data as a string slice, if valid UTF-8.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        std::str::from_utf8(&self.data).ok()
    }

    /// Returns the length of the data.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the transmit is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl From<Vec<u8>> for Transmit {
    fn from(data: Vec<u8>) -> Self {
        Self::new(data)
    }
}

impl From<String> for Transmit {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for Transmit {
    fn from(s: &str) -> Self {
        Self::new(s.as_bytes().to_vec())
    }
}

impl AsRef<[u8]> for Transmit {
    fn as_ref(&self) -> &[u8] {
        &self.data
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
    fn test_transmit_new() {
        let t = Transmit::new(vec![1, 2, 3]);
        assert_eq!(t.data, vec![1, 2, 3]);
        assert_eq!(t.len(), 3);
        assert!(!t.is_empty());
    }

    #[test]
    fn test_transmit_from_string() {
        let t = Transmit::from_string("A001 NOOP\r\n".to_string());
        assert_eq!(t.as_str(), Some("A001 NOOP\r\n"));
    }

    #[test]
    fn test_transmit_empty() {
        let t = Transmit::new(vec![]);
        assert!(t.is_empty());
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn test_transmit_from_impls() {
        let t1: Transmit = vec![1, 2, 3].into();
        assert_eq!(t1.len(), 3);

        let t2: Transmit = "hello".into();
        assert_eq!(t2.as_str(), Some("hello"));

        let t3: Transmit = String::from("world").into();
        assert_eq!(t3.as_str(), Some("world"));
    }

    #[test]
    fn test_transmit_as_ref() {
        let t = Transmit::new(vec![1, 2, 3]);
        let slice: &[u8] = t.as_ref();
        assert_eq!(slice, &[1, 2, 3]);
    }
}
