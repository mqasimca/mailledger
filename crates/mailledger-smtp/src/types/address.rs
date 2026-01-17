//! Email address types.

use crate::error::{Error, Result};

/// Email address for SMTP envelope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
    /// Creates a new address from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(addr: impl Into<String>) -> Result<Self> {
        let addr = addr.into();
        Self::validate(&addr)?;
        Ok(Self(addr))
    }

    /// Returns the address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Validates an email address (basic validation).
    fn validate(addr: &str) -> Result<()> {
        if addr.is_empty() {
            return Err(Error::InvalidAddress("Address cannot be empty".into()));
        }

        if !addr.contains('@') {
            return Err(Error::InvalidAddress("Address must contain @".into()));
        }

        let parts: Vec<&str> = addr.split('@').collect();
        if parts.len() != 2 {
            return Err(Error::InvalidAddress(
                "Address must have exactly one @".into(),
            ));
        }

        if parts[0].is_empty() || parts[1].is_empty() {
            return Err(Error::InvalidAddress(
                "Local and domain parts cannot be empty".into(),
            ));
        }

        Ok(())
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Mailbox (optional display name + address).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailbox {
    /// Display name (optional).
    pub name: Option<String>,
    /// Email address.
    pub address: Address,
}

impl Mailbox {
    /// Creates a new mailbox with just an address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn new(address: impl Into<String>) -> Result<Self> {
        Ok(Self {
            name: None,
            address: Address::new(address)?,
        })
    }

    /// Creates a new mailbox with a display name and address.
    ///
    /// # Errors
    ///
    /// Returns an error if the address is invalid.
    pub fn with_name(name: impl Into<String>, address: impl Into<String>) -> Result<Self> {
        Ok(Self {
            name: Some(name.into()),
            address: Address::new(address)?,
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_address() {
        let addr = Address::new("user@example.com").unwrap();
        assert_eq!(addr.as_str(), "user@example.com");
    }

    #[test]
    fn test_invalid_address_no_at() {
        assert!(Address::new("userexample.com").is_err());
    }

    #[test]
    fn test_invalid_address_empty() {
        assert!(Address::new("").is_err());
    }

    #[test]
    fn test_invalid_address_empty_local() {
        assert!(Address::new("@example.com").is_err());
    }

    #[test]
    fn test_invalid_address_empty_domain() {
        assert!(Address::new("user@").is_err());
    }

    #[test]
    fn test_mailbox_new() {
        let mailbox = Mailbox::new("user@example.com").unwrap();
        assert_eq!(mailbox.address.as_str(), "user@example.com");
        assert!(mailbox.name.is_none());
    }

    #[test]
    fn test_mailbox_with_name() {
        let mailbox = Mailbox::with_name("John Doe", "john@example.com").unwrap();
        assert_eq!(mailbox.name.as_deref(), Some("John Doe"));
        assert_eq!(mailbox.address.as_str(), "john@example.com");
    }
}
