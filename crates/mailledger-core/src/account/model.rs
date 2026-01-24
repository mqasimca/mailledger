//! Account model types.

use serde::{Deserialize, Serialize};

/// Unique identifier for an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(pub i64);

impl AccountId {
    /// Create a new account ID.
    #[must_use]
    pub const fn new(id: i64) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Security/encryption mode for connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Security {
    /// No encryption (not recommended).
    None,
    /// Implicit TLS (connect directly with TLS).
    #[default]
    Tls,
    /// STARTTLS upgrade after plaintext connect.
    StartTls,
}

impl Security {
    /// Get display name for the security mode.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::None => "None (insecure)",
            Self::Tls => "SSL/TLS",
            Self::StartTls => "STARTTLS",
        }
    }
}

/// IMAP server configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImapConfig {
    /// Server hostname.
    pub host: String,
    /// Server port (default: 993 for TLS, 143 for STARTTLS).
    pub port: u16,
    /// Security mode.
    pub security: Security,
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
}

impl ImapConfig {
    /// Get default port for the security mode.
    #[must_use]
    pub const fn default_port(security: Security) -> u16 {
        match security {
            Security::None | Security::StartTls => 143,
            Security::Tls => 993,
        }
    }
}

/// SMTP server configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// Server hostname.
    pub host: String,
    /// Server port (default: 465 for TLS, 587 for STARTTLS).
    pub port: u16,
    /// Security mode.
    pub security: Security,
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
}

impl SmtpConfig {
    /// Get default port for the security mode.
    #[must_use]
    pub const fn default_port(security: Security) -> u16 {
        match security {
            Security::None => 25,
            Security::StartTls => 587,
            Security::Tls => 465,
        }
    }
}

/// Email account configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    /// Unique identifier (None for unsaved accounts).
    pub id: Option<AccountId>,
    /// Display name for the account.
    pub name: String,
    /// Email address.
    pub email: String,
    /// IMAP configuration.
    pub imap: ImapConfig,
    /// SMTP configuration.
    pub smtp: SmtpConfig,
    /// Whether this is the default account.
    pub is_default: bool,
}

impl Account {
    /// Create a new empty account.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create account with common defaults for well-known providers.
    #[must_use]
    pub fn with_email(email: &str) -> Self {
        let mut account = Self {
            email: email.to_string(),
            ..Default::default()
        };

        // Auto-detect provider settings
        if let Some(domain) = email.split('@').nth(1) {
            match domain.to_lowercase().as_str() {
                "gmail.com" | "googlemail.com" => {
                    account.name = "Gmail".to_string();
                    account.imap.host = "imap.gmail.com".to_string();
                    account.imap.port = 993;
                    account.imap.security = Security::Tls;
                    account.smtp.host = "smtp.gmail.com".to_string();
                    account.smtp.port = 465;
                    account.smtp.security = Security::Tls;
                }
                "outlook.com" | "hotmail.com" | "live.com" => {
                    account.name = "Outlook".to_string();
                    account.imap.host = "outlook.office365.com".to_string();
                    account.imap.port = 993;
                    account.imap.security = Security::Tls;
                    account.smtp.host = "smtp.office365.com".to_string();
                    account.smtp.port = 587;
                    account.smtp.security = Security::StartTls;
                }
                "yahoo.com" | "ymail.com" => {
                    account.name = "Yahoo".to_string();
                    account.imap.host = "imap.mail.yahoo.com".to_string();
                    account.imap.port = 993;
                    account.imap.security = Security::Tls;
                    account.smtp.host = "smtp.mail.yahoo.com".to_string();
                    account.smtp.port = 465;
                    account.smtp.security = Security::Tls;
                }
                "icloud.com" | "me.com" | "mac.com" => {
                    account.name = "iCloud".to_string();
                    account.imap.host = "imap.mail.me.com".to_string();
                    account.imap.port = 993;
                    account.imap.security = Security::Tls;
                    account.smtp.host = "smtp.mail.me.com".to_string();
                    account.smtp.port = 587;
                    account.smtp.security = Security::StartTls;
                }
                _ => {
                    // Use domain as account name
                    account.name = domain.to_string();
                }
            }
        }

        // Set username to email by default
        account.imap.username = email.to_string();
        account.smtp.username = email.to_string();

        account
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

    mod account_id_tests {
        use super::*;

        #[test]
        fn new() {
            let id = AccountId::new(42);
            assert_eq!(id.0, 42);
        }

        #[test]
        fn display() {
            let id = AccountId::new(123);
            assert_eq!(format!("{id}"), "123");
        }

        #[test]
        fn equality() {
            let id1 = AccountId::new(1);
            let id2 = AccountId::new(1);
            let id3 = AccountId::new(2);
            assert_eq!(id1, id2);
            assert_ne!(id1, id3);
        }
    }

    mod security_tests {
        use super::*;

        #[test]
        fn default_is_tls() {
            assert_eq!(Security::default(), Security::Tls);
        }

        #[test]
        fn display_names() {
            assert_eq!(Security::None.display_name(), "None (insecure)");
            assert_eq!(Security::Tls.display_name(), "SSL/TLS");
            assert_eq!(Security::StartTls.display_name(), "STARTTLS");
        }
    }

    mod imap_config_tests {
        use super::*;

        #[test]
        fn default_port_tls() {
            assert_eq!(ImapConfig::default_port(Security::Tls), 993);
        }

        #[test]
        fn default_port_starttls() {
            assert_eq!(ImapConfig::default_port(Security::StartTls), 143);
        }

        #[test]
        fn default_port_none() {
            assert_eq!(ImapConfig::default_port(Security::None), 143);
        }

        #[test]
        fn default() {
            let config = ImapConfig::default();
            assert!(config.host.is_empty());
            assert_eq!(config.port, 0);
            assert_eq!(config.security, Security::Tls);
        }
    }

    mod smtp_config_tests {
        use super::*;

        #[test]
        fn default_port_tls() {
            assert_eq!(SmtpConfig::default_port(Security::Tls), 465);
        }

        #[test]
        fn default_port_starttls() {
            assert_eq!(SmtpConfig::default_port(Security::StartTls), 587);
        }

        #[test]
        fn default_port_none() {
            assert_eq!(SmtpConfig::default_port(Security::None), 25);
        }

        #[test]
        fn default() {
            let config = SmtpConfig::default();
            assert!(config.host.is_empty());
            assert_eq!(config.port, 0);
            assert_eq!(config.security, Security::Tls);
        }
    }

    mod account_tests {
        use super::*;

        #[test]
        fn new_creates_empty() {
            let account = Account::new();
            assert!(account.id.is_none());
            assert!(account.name.is_empty());
            assert!(account.email.is_empty());
            assert!(!account.is_default);
        }

        #[test]
        fn with_email_gmail() {
            let account = Account::with_email("user@gmail.com");
            assert_eq!(account.name, "Gmail");
            assert_eq!(account.email, "user@gmail.com");
            assert_eq!(account.imap.host, "imap.gmail.com");
            assert_eq!(account.imap.port, 993);
            assert_eq!(account.imap.security, Security::Tls);
            assert_eq!(account.smtp.host, "smtp.gmail.com");
            assert_eq!(account.smtp.port, 465);
            assert_eq!(account.smtp.security, Security::Tls);
            assert_eq!(account.imap.username, "user@gmail.com");
            assert_eq!(account.smtp.username, "user@gmail.com");
        }

        #[test]
        fn with_email_googlemail() {
            let account = Account::with_email("user@googlemail.com");
            assert_eq!(account.name, "Gmail");
            assert_eq!(account.imap.host, "imap.gmail.com");
        }

        #[test]
        fn with_email_outlook() {
            let account = Account::with_email("user@outlook.com");
            assert_eq!(account.name, "Outlook");
            assert_eq!(account.imap.host, "outlook.office365.com");
            assert_eq!(account.smtp.host, "smtp.office365.com");
            assert_eq!(account.smtp.port, 587);
            assert_eq!(account.smtp.security, Security::StartTls);
        }

        #[test]
        fn with_email_hotmail() {
            let account = Account::with_email("user@hotmail.com");
            assert_eq!(account.name, "Outlook");
        }

        #[test]
        fn with_email_live() {
            let account = Account::with_email("user@live.com");
            assert_eq!(account.name, "Outlook");
        }

        #[test]
        fn with_email_yahoo() {
            let account = Account::with_email("user@yahoo.com");
            assert_eq!(account.name, "Yahoo");
            assert_eq!(account.imap.host, "imap.mail.yahoo.com");
            assert_eq!(account.smtp.host, "smtp.mail.yahoo.com");
        }

        #[test]
        fn with_email_ymail() {
            let account = Account::with_email("user@ymail.com");
            assert_eq!(account.name, "Yahoo");
        }

        #[test]
        fn with_email_icloud() {
            let account = Account::with_email("user@icloud.com");
            assert_eq!(account.name, "iCloud");
            assert_eq!(account.imap.host, "imap.mail.me.com");
            assert_eq!(account.smtp.host, "smtp.mail.me.com");
            assert_eq!(account.smtp.port, 587);
            assert_eq!(account.smtp.security, Security::StartTls);
        }

        #[test]
        fn with_email_me_com() {
            let account = Account::with_email("user@me.com");
            assert_eq!(account.name, "iCloud");
        }

        #[test]
        fn with_email_mac_com() {
            let account = Account::with_email("user@mac.com");
            assert_eq!(account.name, "iCloud");
        }

        #[test]
        fn with_email_unknown_domain() {
            let account = Account::with_email("user@example.org");
            assert_eq!(account.name, "example.org");
            // Host should not be auto-filled for unknown domains
            assert!(account.imap.host.is_empty());
            assert!(account.smtp.host.is_empty());
        }

        #[test]
        fn with_email_sets_username() {
            let account = Account::with_email("test@example.com");
            assert_eq!(account.imap.username, "test@example.com");
            assert_eq!(account.smtp.username, "test@example.com");
        }
    }
}
