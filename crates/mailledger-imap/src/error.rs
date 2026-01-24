//! Error types for the IMAP library.
//!
//! This module provides comprehensive error types with context for debugging
//! and user-facing error messages. Errors capture both the operation that
//! failed and details about what went wrong.

use std::time::Duration;

use thiserror::Error;

/// Errors that can occur during IMAP operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error during network operations.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TLS handshake or encryption error.
    #[error("TLS error: {0}")]
    Tls(#[from] rustls::Error),

    /// Invalid DNS name for TLS.
    #[error("Invalid DNS name: {0}")]
    InvalidDnsName(#[from] rustls::pki_types::InvalidDnsNameError),

    /// Protocol parsing error.
    #[error("Protocol error at position {position}: {message}")]
    Parse {
        /// Byte position where the error occurred.
        position: usize,
        /// Description of what went wrong.
        message: String,
    },

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Server returned NO response.
    #[error("Server returned NO: {0}")]
    No(String),

    /// Server returned BAD response.
    #[error("Server returned BAD: {0}")]
    Bad(String),

    /// Server sent BYE (disconnecting).
    #[error("Server sent BYE: {0}")]
    Bye(String),

    /// Operation timed out.
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),

    /// Invalid state for the requested operation.
    #[error("Invalid state: {0}")]
    InvalidState(String),

    /// Protocol violation or unexpected data.
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Command failed with context.
    #[error("{command} failed: {source}")]
    Command {
        /// The command that was being executed.
        command: CommandContext,
        /// The underlying error.
        #[source]
        source: Box<Self>,
    },

    /// Connection was lost.
    #[error("Connection lost: {0}")]
    ConnectionLost(String),

    /// Server is unavailable.
    #[error("Server unavailable: {0}")]
    Unavailable(String),
}

impl Error {
    /// Wraps this error with command context.
    #[must_use]
    pub fn with_command(self, command: impl Into<CommandContext>) -> Self {
        Self::Command {
            command: command.into(),
            source: Box::new(self),
        }
    }

    /// Returns true if this error is recoverable (e.g., temporary failure).
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Timeout(_) | Self::ConnectionLost(_) | Self::Unavailable(_)
        )
    }

    /// Returns true if this error indicates the connection is dead.
    #[must_use]
    pub const fn is_connection_dead(&self) -> bool {
        matches!(
            self,
            Self::Io(_) | Self::Bye(_) | Self::ConnectionLost(_) | Self::Tls(_)
        )
    }

    /// Returns true if this is an authentication error.
    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
            || matches!(self, Self::No(text) if text.to_lowercase().contains("auth"))
    }
}

/// Context about which command failed.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// The command name (e.g., "SELECT", "FETCH").
    pub name: String,
    /// Optional argument (e.g., mailbox name for SELECT).
    pub arg: Option<String>,
}

impl CommandContext {
    /// Creates a new command context.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arg: None,
        }
    }

    /// Adds an argument to the context.
    #[must_use]
    pub fn with_arg(mut self, arg: impl Into<String>) -> Self {
        self.arg = Some(arg.into());
        self
    }
}

impl std::fmt::Display for CommandContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref arg) = self.arg {
            write!(f, " {arg}")?;
        }
        Ok(())
    }
}

impl From<&str> for CommandContext {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl From<String> for CommandContext {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

/// Result type alias using our Error type.
pub type Result<T> = std::result::Result<T, Error>;

/// Extension trait for adding context to Results.
pub trait ResultExt<T> {
    /// Wraps the error with command context.
    ///
    /// # Errors
    ///
    /// Returns the original error wrapped with command context.
    fn with_command(self, command: impl Into<CommandContext>) -> Result<T>;
}

impl<T> ResultExt<T> for Result<T> {
    fn with_command(self, command: impl Into<CommandContext>) -> Self {
        self.map_err(|e| e.with_command(command))
    }
}
