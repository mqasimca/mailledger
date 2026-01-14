//! Account setup state model.

use std::collections::HashMap;

/// State for the account setup form.
#[derive(Debug, Clone, Default)]
pub struct AccountSetupState {
    /// Account name.
    pub name: String,
    /// Email address.
    pub email: String,
    /// IMAP server host.
    pub imap_host: String,
    /// IMAP server port.
    pub imap_port: String,
    /// IMAP security mode (tls, starttls, none).
    pub imap_security: String,
    /// IMAP username.
    pub imap_username: String,
    /// IMAP password.
    pub imap_password: String,
    /// SMTP server host.
    pub smtp_host: String,
    /// SMTP server port.
    pub smtp_port: String,
    /// SMTP security mode.
    pub smtp_security: String,
    /// SMTP username.
    pub smtp_username: String,
    /// SMTP password.
    pub smtp_password: String,
    /// Validation errors by field name.
    pub errors: HashMap<String, String>,
    /// Error from save operation.
    pub save_error: Option<String>,
    /// Whether save is in progress.
    pub is_saving: bool,
    /// Whether connection test is in progress.
    pub is_testing: bool,
    /// Connection test result.
    pub test_result: Option<Result<(), String>>,
}

impl AccountSetupState {
    /// Create a new empty account setup state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            imap_port: "993".to_string(),
            imap_security: "tls".to_string(),
            smtp_port: "465".to_string(),
            smtp_security: "tls".to_string(),
            ..Default::default()
        }
    }

    /// Auto-detect settings from email address.
    pub fn auto_detect_from_email(&mut self) {
        if let Some(domain) = self.email.split('@').nth(1) {
            match domain.to_lowercase().as_str() {
                "gmail.com" | "googlemail.com" => {
                    self.name = "Gmail".to_string();
                    self.imap_host = "imap.gmail.com".to_string();
                    self.imap_port = "993".to_string();
                    self.imap_security = "tls".to_string();
                    self.smtp_host = "smtp.gmail.com".to_string();
                    self.smtp_port = "465".to_string();
                    self.smtp_security = "tls".to_string();
                }
                "outlook.com" | "hotmail.com" | "live.com" => {
                    self.name = "Outlook".to_string();
                    self.imap_host = "outlook.office365.com".to_string();
                    self.imap_port = "993".to_string();
                    self.imap_security = "tls".to_string();
                    self.smtp_host = "smtp.office365.com".to_string();
                    self.smtp_port = "587".to_string();
                    self.smtp_security = "starttls".to_string();
                }
                "yahoo.com" | "ymail.com" => {
                    self.name = "Yahoo".to_string();
                    self.imap_host = "imap.mail.yahoo.com".to_string();
                    self.imap_port = "993".to_string();
                    self.imap_security = "tls".to_string();
                    self.smtp_host = "smtp.mail.yahoo.com".to_string();
                    self.smtp_port = "465".to_string();
                    self.smtp_security = "tls".to_string();
                }
                "icloud.com" | "me.com" | "mac.com" => {
                    self.name = "iCloud".to_string();
                    self.imap_host = "imap.mail.me.com".to_string();
                    self.imap_port = "993".to_string();
                    self.imap_security = "tls".to_string();
                    self.smtp_host = "smtp.mail.me.com".to_string();
                    self.smtp_port = "587".to_string();
                    self.smtp_security = "starttls".to_string();
                }
                _ => {
                    if self.name.is_empty() {
                        self.name = domain.to_string();
                    }
                }
            }
        }

        // Default username to email
        if self.imap_username.is_empty() {
            self.imap_username = self.email.clone();
        }
        if self.smtp_username.is_empty() {
            self.smtp_username = self.email.clone();
        }
    }

    /// Validate the form and return errors.
    pub fn validate(&mut self) -> bool {
        self.errors.clear();

        if self.name.trim().is_empty() {
            self.errors
                .insert("name".to_string(), "Account name is required".to_string());
        }

        if self.email.trim().is_empty() {
            self.errors
                .insert("email".to_string(), "Email is required".to_string());
        } else if !self.email.contains('@') || !self.email.contains('.') {
            self.errors
                .insert("email".to_string(), "Invalid email format".to_string());
        }

        if self.imap_host.trim().is_empty() {
            self.errors.insert(
                "imap_host".to_string(),
                "IMAP server is required".to_string(),
            );
        }
        if self.imap_username.trim().is_empty() {
            self.errors.insert(
                "imap_username".to_string(),
                "IMAP username is required".to_string(),
            );
        }
        if self.imap_password.is_empty() {
            self.errors.insert(
                "imap_password".to_string(),
                "IMAP password is required".to_string(),
            );
        }

        if self.smtp_host.trim().is_empty() {
            self.errors.insert(
                "smtp_host".to_string(),
                "SMTP server is required".to_string(),
            );
        }
        if self.smtp_username.trim().is_empty() {
            self.errors.insert(
                "smtp_username".to_string(),
                "SMTP username is required".to_string(),
            );
        }
        if self.smtp_password.is_empty() {
            self.errors.insert(
                "smtp_password".to_string(),
                "SMTP password is required".to_string(),
            );
        }

        self.errors.is_empty()
    }

    /// Convert to core Account type.
    #[must_use]
    pub fn to_account(&self) -> mailledger_core::Account {
        use mailledger_core::{ImapConfig, Security, SmtpConfig};

        let parse_security = |s: &str| match s {
            "starttls" => Security::StartTls,
            "none" => Security::None,
            _ => Security::Tls,
        };

        mailledger_core::Account {
            id: None,
            name: self.name.clone(),
            email: self.email.clone(),
            imap: ImapConfig {
                host: self.imap_host.clone(),
                port: self.imap_port.parse().unwrap_or(993),
                security: parse_security(&self.imap_security),
                username: self.imap_username.clone(),
                password: self.imap_password.clone(),
            },
            smtp: SmtpConfig {
                host: self.smtp_host.clone(),
                port: self.smtp_port.parse().unwrap_or(465),
                security: parse_security(&self.smtp_security),
                username: self.smtp_username.clone(),
                password: self.smtp_password.clone(),
            },
            is_default: true, // First account is default
        }
    }
}
