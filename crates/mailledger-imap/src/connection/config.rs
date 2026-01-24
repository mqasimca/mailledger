//! Connection configuration types.

use std::time::Duration;

/// Connection security mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Security {
    /// No encryption (port 143). **Not recommended for production.**
    None,
    /// Start with plaintext, upgrade with STARTTLS (port 143).
    StartTls,
    /// TLS from the start (port 993). **Recommended.**
    #[default]
    Implicit,
}

impl Security {
    /// Returns the default port for this security mode.
    #[must_use]
    pub const fn default_port(self) -> u16 {
        match self {
            Self::None | Self::StartTls => 143,
            Self::Implicit => 993,
        }
    }
}

/// IMAP connection configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server hostname.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// Security mode.
    pub security: Security,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Read/write timeout.
    pub io_timeout: Duration,
}

impl Config {
    /// Creates a new configuration with implicit TLS on port 993.
    #[must_use]
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 993,
            security: Security::Implicit,
            connect_timeout: Duration::from_secs(30),
            io_timeout: Duration::from_secs(60),
        }
    }

    /// Creates a configuration builder.
    #[must_use]
    pub fn builder(host: impl Into<String>) -> ConfigBuilder {
        ConfigBuilder::new(host)
    }
}

/// Builder for connection configuration.
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    host: String,
    port: Option<u16>,
    security: Security,
    connect_timeout: Duration,
    io_timeout: Duration,
}

impl ConfigBuilder {
    /// Creates a new builder with the given hostname.
    #[must_use]
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: None,
            security: Security::Implicit,
            connect_timeout: Duration::from_secs(30),
            io_timeout: Duration::from_secs(60),
        }
    }

    /// Sets the port.
    #[must_use]
    pub const fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets the security mode.
    #[must_use]
    pub const fn security(mut self, security: Security) -> Self {
        self.security = security;
        self
    }

    /// Sets the connection timeout.
    #[must_use]
    pub const fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets the I/O timeout.
    #[must_use]
    pub const fn io_timeout(mut self, timeout: Duration) -> Self {
        self.io_timeout = timeout;
        self
    }

    /// Builds the configuration.
    #[must_use]
    pub fn build(self) -> Config {
        Config {
            host: self.host,
            port: self.port.unwrap_or_else(|| self.security.default_port()),
            security: self.security,
            connect_timeout: self.connect_timeout,
            io_timeout: self.io_timeout,
        }
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
    fn test_default_ports() {
        assert_eq!(Security::None.default_port(), 143);
        assert_eq!(Security::StartTls.default_port(), 143);
        assert_eq!(Security::Implicit.default_port(), 993);
    }

    #[test]
    fn test_config_new() {
        let config = Config::new("imap.example.com");
        assert_eq!(config.host, "imap.example.com");
        assert_eq!(config.port, 993);
        assert_eq!(config.security, Security::Implicit);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder("imap.example.com")
            .port(993)
            .security(Security::Implicit)
            .connect_timeout(Duration::from_secs(10))
            .build();

        assert_eq!(config.host, "imap.example.com");
        assert_eq!(config.port, 993);
        assert_eq!(config.security, Security::Implicit);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_config_builder_default_port() {
        let config = Config::builder("imap.example.com")
            .security(Security::StartTls)
            .build();

        assert_eq!(config.port, 143);
    }
}
