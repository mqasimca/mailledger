//! Account validation.

use super::model::Account;

/// Validation error for account configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Account name is empty.
    EmptyName,
    /// Email address is empty.
    EmptyEmail,
    /// Email address format is invalid.
    InvalidEmail,
    /// IMAP host is empty.
    EmptyImapHost,
    /// IMAP port is invalid.
    InvalidImapPort,
    /// IMAP username is empty.
    EmptyImapUsername,
    /// IMAP password is empty.
    EmptyImapPassword,
    /// SMTP host is empty.
    EmptySmtpHost,
    /// SMTP port is invalid.
    InvalidSmtpPort,
    /// SMTP username is empty.
    EmptySmtpUsername,
    /// SMTP password is empty.
    EmptySmtpPassword,
}

impl ValidationError {
    /// Get human-readable error message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            Self::EmptyName => "Account name is required",
            Self::EmptyEmail => "Email address is required",
            Self::InvalidEmail => "Invalid email address format",
            Self::EmptyImapHost => "IMAP server is required",
            Self::InvalidImapPort => "IMAP port must be 1-65535",
            Self::EmptyImapUsername => "IMAP username is required",
            Self::EmptyImapPassword => "IMAP password is required",
            Self::EmptySmtpHost => "SMTP server is required",
            Self::InvalidSmtpPort => "SMTP port must be 1-65535",
            Self::EmptySmtpUsername => "SMTP username is required",
            Self::EmptySmtpPassword => "SMTP password is required",
        }
    }

    /// Get the field name this error relates to.
    #[must_use]
    pub const fn field(&self) -> &'static str {
        match self {
            Self::EmptyName => "name",
            Self::EmptyEmail | Self::InvalidEmail => "email",
            Self::EmptyImapHost => "imap_host",
            Self::InvalidImapPort => "imap_port",
            Self::EmptyImapUsername => "imap_username",
            Self::EmptyImapPassword => "imap_password",
            Self::EmptySmtpHost => "smtp_host",
            Self::InvalidSmtpPort => "smtp_port",
            Self::EmptySmtpUsername => "smtp_username",
            Self::EmptySmtpPassword => "smtp_password",
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ValidationError {}

/// Result of validating an account.
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Validate an account configuration.
///
/// Returns `Ok(())` if valid, or `Err(Vec<ValidationError>)` with all errors.
///
/// # Errors
///
/// Returns a vector of `ValidationError` if any fields are invalid.
pub fn validate_account(account: &Account) -> ValidationResult {
    let mut errors = Vec::new();

    // Name validation
    if account.name.trim().is_empty() {
        errors.push(ValidationError::EmptyName);
    }

    // Email validation
    if account.email.trim().is_empty() {
        errors.push(ValidationError::EmptyEmail);
    } else if !is_valid_email(&account.email) {
        errors.push(ValidationError::InvalidEmail);
    }

    // IMAP validation
    if account.imap.host.trim().is_empty() {
        errors.push(ValidationError::EmptyImapHost);
    }
    if account.imap.port == 0 {
        errors.push(ValidationError::InvalidImapPort);
    }
    if account.imap.username.trim().is_empty() {
        errors.push(ValidationError::EmptyImapUsername);
    }
    if account.imap.password.is_empty() {
        errors.push(ValidationError::EmptyImapPassword);
    }

    // SMTP validation
    if account.smtp.host.trim().is_empty() {
        errors.push(ValidationError::EmptySmtpHost);
    }
    if account.smtp.port == 0 {
        errors.push(ValidationError::InvalidSmtpPort);
    }
    if account.smtp.username.trim().is_empty() {
        errors.push(ValidationError::EmptySmtpUsername);
    }
    if account.smtp.password.is_empty() {
        errors.push(ValidationError::EmptySmtpPassword);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Basic email validation.
fn is_valid_email(email: &str) -> bool {
    let email = email.trim();

    // Must contain exactly one @
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local = parts[0];
    let domain = parts[1];

    // Local part must not be empty
    if local.is_empty() {
        return false;
    }

    // Domain must contain at least one dot and not be empty
    if domain.is_empty() || !domain.contains('.') {
        return false;
    }

    // Domain parts must not be empty
    let domain_parts: Vec<&str> = domain.split('.').collect();
    if domain_parts.iter().any(|p| p.is_empty()) {
        return false;
    }

    true
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
    fn test_valid_email() {
        assert!(is_valid_email("user@example.com"));
        assert!(is_valid_email("user.name@example.com"));
        assert!(is_valid_email("user@sub.example.com"));
    }

    #[test]
    fn test_invalid_email() {
        assert!(!is_valid_email(""));
        assert!(!is_valid_email("user"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("user@"));
        assert!(!is_valid_email("user@example"));
        assert!(!is_valid_email("user@@example.com"));
    }

    #[test]
    fn test_validate_empty_account() {
        let account = Account::new();
        let result = validate_account(&account);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&ValidationError::EmptyName));
        assert!(errors.contains(&ValidationError::EmptyEmail));
    }

    #[test]
    fn test_validate_complete_account() {
        let mut account = Account::with_email("test@gmail.com");
        account.imap.password = "secret".to_string();
        account.smtp.password = "secret".to_string();
        let result = validate_account(&account);
        assert!(result.is_ok());
    }
}
