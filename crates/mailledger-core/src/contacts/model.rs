//! Contact model for address autocomplete.

/// A contact extracted from email messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    /// Email address (unique identifier).
    pub email: String,
    /// Display name (may be empty).
    pub name: String,
    /// Number of times this contact has been used.
    pub use_count: u32,
}

impl Contact {
    /// Creates a new contact.
    #[must_use]
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: name.into(),
            use_count: 1,
        }
    }

    /// Returns a display string for the contact.
    ///
    /// If a name is present, returns "Name <email>", otherwise just "email".
    #[must_use]
    pub fn display(&self) -> String {
        if self.name.is_empty() {
            self.email.clone()
        } else {
            format!("{} <{}>", self.name, self.email)
        }
    }

    /// Checks if the contact matches a search query (case-insensitive prefix match).
    #[must_use]
    pub fn matches(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        self.email.to_lowercase().contains(&query_lower)
            || self.name.to_lowercase().contains(&query_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_with_name() {
        let contact = Contact::new("test@example.com", "John Doe");
        assert_eq!(contact.display(), "John Doe <test@example.com>");
    }

    #[test]
    fn test_display_without_name() {
        let contact = Contact::new("test@example.com", "");
        assert_eq!(contact.display(), "test@example.com");
    }

    #[test]
    fn test_matches_email() {
        let contact = Contact::new("john@example.com", "John Doe");
        assert!(contact.matches("john"));
        assert!(contact.matches("JOHN"));
        assert!(contact.matches("example"));
        assert!(!contact.matches("jane"));
    }

    #[test]
    fn test_matches_name() {
        let contact = Contact::new("john@example.com", "John Doe");
        assert!(contact.matches("doe"));
        assert!(contact.matches("DOE"));
    }
}
