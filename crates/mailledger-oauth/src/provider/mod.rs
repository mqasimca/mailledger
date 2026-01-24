//! `OAuth2` provider configurations.

use crate::error::{Error, Result};
use url::Url;

/// `OAuth2` provider configuration.
#[derive(Debug, Clone)]
pub struct Provider {
    /// Provider name (e.g., "Google").
    pub name: String,
    /// Authorization endpoint URL.
    pub auth_url: Url,
    /// Token endpoint URL.
    pub token_url: Url,
    /// Device authorization endpoint (if supported).
    pub device_auth_url: Option<Url>,
    /// Default scopes.
    pub default_scopes: Vec<String>,
}

impl Provider {
    /// Creates a new provider configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if URLs are invalid.
    pub fn new(
        name: impl Into<String>,
        auth_url: impl AsRef<str>,
        token_url: impl AsRef<str>,
    ) -> Result<Self> {
        Ok(Self {
            name: name.into(),
            auth_url: Url::parse(auth_url.as_ref())?,
            token_url: Url::parse(token_url.as_ref())?,
            device_auth_url: None,
            default_scopes: Vec::new(),
        })
    }

    /// Sets the device authorization URL.
    ///
    /// # Errors
    ///
    /// Returns an error if the URL is invalid.
    pub fn with_device_auth_url(mut self, url: impl AsRef<str>) -> Result<Self> {
        self.device_auth_url = Some(Url::parse(url.as_ref())?);
        Ok(self)
    }

    /// Sets the default scopes.
    #[must_use]
    pub fn with_default_scopes(mut self, scopes: Vec<String>) -> Self {
        self.default_scopes = scopes;
        self
    }

    /// Google `OAuth2` provider configuration.
    ///
    /// Scopes:
    /// - `https://mail.google.com/` - Full Gmail access (IMAP/SMTP)
    ///
    /// # Errors
    ///
    /// Returns an error if URL parsing fails.
    pub fn google() -> Result<Self> {
        Ok(Self::new(
            "Google",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        )?
        .with_device_auth_url("https://oauth2.googleapis.com/device/code")?
        .with_default_scopes(vec!["https://mail.google.com/".to_string()]))
    }

    /// Microsoft/Outlook `OAuth2` provider configuration.
    ///
    /// Scopes:
    /// - `https://outlook.office.com/IMAP.AccessAsUser.All` - IMAP access
    /// - `https://outlook.office.com/SMTP.Send` - SMTP access
    /// - `offline_access` - Refresh token
    ///
    /// # Errors
    ///
    /// Returns an error if URL parsing fails.
    pub fn microsoft() -> Result<Self> {
        Ok(Self::new(
            "Microsoft",
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
            "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        )?
        .with_device_auth_url("https://login.microsoftonline.com/common/oauth2/v2.0/devicecode")?
        .with_default_scopes(vec![
            "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
            "https://outlook.office.com/SMTP.Send".to_string(),
            "offline_access".to_string(),
        ]))
    }

    /// Yahoo `OAuth2` provider configuration.
    ///
    /// Scopes:
    /// - `mail-w` - Mail write access (SMTP)
    /// - `mail-r` - Mail read access (IMAP)
    ///
    /// # Errors
    ///
    /// Returns an error if URL parsing fails.
    pub fn yahoo() -> Result<Self> {
        Ok(Self::new(
            "Yahoo",
            "https://api.login.yahoo.com/oauth2/request_auth",
            "https://api.login.yahoo.com/oauth2/get_token",
        )?
        .with_default_scopes(vec!["mail-w".to_string(), "mail-r".to_string()]))
    }

    /// Validates that required URLs are set.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid.
    pub fn validate(&self) -> Result<()> {
        if self.auth_url.as_str().is_empty() {
            return Err(Error::InvalidConfig("auth_url is empty".into()));
        }
        if self.token_url.as_str().is_empty() {
            return Err(Error::InvalidConfig("token_url is empty".into()));
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
    fn test_google_provider() {
        let provider = Provider::google().unwrap();
        assert_eq!(provider.name, "Google");
        assert!(provider.device_auth_url.is_some());
        assert!(!provider.default_scopes.is_empty());
        provider.validate().unwrap();
    }

    #[test]
    fn test_microsoft_provider() {
        let provider = Provider::microsoft().unwrap();
        assert_eq!(provider.name, "Microsoft");
        assert!(provider.device_auth_url.is_some());
        assert_eq!(provider.default_scopes.len(), 3);
        provider.validate().unwrap();
    }

    #[test]
    fn test_yahoo_provider() {
        let provider = Provider::yahoo().unwrap();
        assert_eq!(provider.name, "Yahoo");
        assert_eq!(provider.default_scopes.len(), 2);
        provider.validate().unwrap();
    }

    #[test]
    fn test_custom_provider() {
        let provider = Provider::new(
            "Custom",
            "https://auth.example.com/authorize",
            "https://auth.example.com/token",
        )
        .unwrap()
        .with_default_scopes(vec!["email".to_string()]);

        assert_eq!(provider.name, "Custom");
        assert_eq!(provider.default_scopes.len(), 1);
        provider.validate().unwrap();
    }
}
