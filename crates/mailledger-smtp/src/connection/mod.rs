//! SMTP connection management with type-state pattern.

mod client;
mod stream;

pub use client::{
    Authenticated, Client, Connected, Data, MailTransaction, RecipientAdded, SmtpConnection,
};
pub use stream::{SmtpStream, connect, connect_tls};

use crate::types::Extension;
use std::collections::HashSet;

/// Server capabilities from EHLO response.
#[derive(Debug, Clone, Default)]
pub struct ServerInfo {
    /// Server hostname from greeting.
    pub hostname: String,
    /// Supported extensions.
    pub extensions: HashSet<Extension>,
}

impl ServerInfo {
    /// Checks if the server supports an extension.
    #[must_use]
    pub fn supports(&self, ext: &Extension) -> bool {
        self.extensions.contains(ext)
    }

    /// Checks if STARTTLS is supported.
    #[must_use]
    pub fn supports_starttls(&self) -> bool {
        self.supports(&Extension::StartTls)
    }

    /// Returns the maximum message size, if advertised.
    #[must_use]
    pub fn max_message_size(&self) -> Option<usize> {
        for ext in &self.extensions {
            if let Extension::Size(size) = ext {
                return *size;
            }
        }
        None
    }

    /// Returns supported authentication mechanisms.
    #[must_use]
    pub fn auth_mechanisms(&self) -> Vec<crate::types::AuthMechanism> {
        for ext in &self.extensions {
            if let Extension::Auth(mechanisms) = ext {
                return mechanisms.clone();
            }
        }
        Vec::new()
    }
}
