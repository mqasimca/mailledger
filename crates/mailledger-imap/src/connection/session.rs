//! High-level IMAP session with automatic reconnection.
//!
//! This module provides `Session`, a wrapper around the type-state `Client`
//! that handles reconnection, state recovery, and provides a simpler API
//! for common operations.
//!
//! ## Design
//!
//! The `Session` type uses interior state management to provide a mutable
//! reference-based API while handling state transitions internally. This
//! is inspired by `imap-next` and `pimalaya/imap-client`.
//!
//! ## Example
//!
//! ```ignore
//! use mailledger_imap::connection::{Session, SessionConfig};
//!
//! let config = SessionConfig::new("imap.example.com", 993)
//!     .credentials("user@example.com", "password");
//!
//! let mut session = Session::connect(config).await?;
//!
//! // Simple API - no need to track state transitions
//! let folders = session.list_folders().await?;
//! session.select("INBOX").await?;
//! let messages = session.fetch_recent(10).await?;
//!
//! // Auto-reconnect on connection loss
//! session.idle(Duration::from_secs(600)).await?;
//! ```

use std::time::Duration;

use super::client::{Authenticated, Client, NotAuthenticated, Selected};
use super::{ImapStream, connect_tls};
use crate::command::{FetchItems, StoreAction};
use crate::parser::FetchItem;
use crate::types::{ListResponse, MailboxStatus, SeqNum, SequenceSet, UidSet};
use crate::{Error, Result};

/// Configuration for an IMAP session.
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Server hostname.
    pub host: String,
    /// Server port (default: 993 for TLS).
    pub port: u16,
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Command timeout.
    pub command_timeout: Duration,
    /// Whether to auto-reconnect on connection loss.
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts.
    pub max_reconnect_attempts: u32,
}

impl SessionConfig {
    /// Creates a new session configuration.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: String::new(),
            password: String::new(),
            connect_timeout: Duration::from_secs(30),
            command_timeout: Duration::from_secs(60),
            auto_reconnect: true,
            max_reconnect_attempts: 3,
        }
    }

    /// Sets the credentials.
    #[must_use]
    pub fn credentials(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = username.into();
        self.password = password.into();
        self
    }

    /// Sets the connection timeout.
    #[must_use]
    pub const fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Sets the command timeout.
    #[must_use]
    pub const fn command_timeout(mut self, timeout: Duration) -> Self {
        self.command_timeout = timeout;
        self
    }

    /// Enables or disables auto-reconnect.
    #[must_use]
    pub const fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }
}

/// Current state of the session.
enum SessionState {
    /// Not connected.
    Disconnected,
    /// Connected but not authenticated.
    Connected(Client<ImapStream, NotAuthenticated>),
    /// Authenticated.
    Authenticated(Client<ImapStream, Authenticated>),
    /// Mailbox selected.
    Selected(Client<ImapStream, Selected>),
}

/// High-level IMAP session with automatic reconnection.
///
/// Provides a simpler API than the raw `Client` type by managing state
/// transitions internally and supporting auto-reconnect.
pub struct Session {
    config: SessionConfig,
    state: SessionState,
    /// Last selected mailbox (for reconnection).
    last_mailbox: Option<String>,
}

impl Session {
    /// Creates a new session and connects to the server.
    ///
    /// # Errors
    ///
    /// Returns an error if connection or authentication fails.
    pub async fn connect(config: SessionConfig) -> Result<Self> {
        let mut session = Self {
            config,
            state: SessionState::Disconnected,
            last_mailbox: None,
        };

        session.do_connect().await?;
        session.do_authenticate().await?;

        Ok(session)
    }

    /// Returns true if the session is connected.
    #[must_use]
    pub const fn is_connected(&self) -> bool {
        !matches!(self.state, SessionState::Disconnected)
    }

    /// Returns true if authenticated.
    #[must_use]
    pub const fn is_authenticated(&self) -> bool {
        matches!(
            self.state,
            SessionState::Authenticated(_) | SessionState::Selected(_)
        )
    }

    /// Returns the currently selected mailbox, if any.
    #[must_use]
    pub fn selected_mailbox(&self) -> Option<&str> {
        match &self.state {
            SessionState::Selected(client) => Some(client.mailbox()),
            _ => None,
        }
    }

    /// Lists all folders.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails.
    pub async fn list_folders(&mut self) -> Result<Vec<ListResponse>> {
        self.ensure_authenticated().await?;

        match &mut self.state {
            SessionState::Authenticated(client) => client.list("", "*").await,
            _ => Err(Error::InvalidState("not authenticated".into())),
        }
    }

    /// Selects a mailbox.
    ///
    /// # Errors
    ///
    /// Returns an error if the mailbox cannot be selected.
    pub async fn select(&mut self, mailbox: &str) -> Result<MailboxStatus> {
        // Don't call ensure_authenticated to avoid recursion
        match &self.state {
            SessionState::Authenticated(_) | SessionState::Selected(_) => {}
            SessionState::Disconnected if self.config.auto_reconnect => {
                self.do_reconnect_no_select().await?;
            }
            _ => return Err(Error::InvalidState("not authenticated".into())),
        }

        // Take ownership of the current client
        let client = match std::mem::replace(&mut self.state, SessionState::Disconnected) {
            SessionState::Authenticated(c) => c,
            SessionState::Selected(c) => c.close().await?,
            _ => return Err(Error::InvalidState("not authenticated".into())),
        };

        let (selected, status) = client.select(mailbox).await?;
        self.state = SessionState::Selected(selected);
        self.last_mailbox = Some(mailbox.to_string());

        Ok(status)
    }

    /// Fetches messages by sequence numbers.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or fetch fails.
    pub async fn fetch(
        &mut self,
        sequence: &SequenceSet,
        items: FetchItems,
    ) -> Result<Vec<(SeqNum, Vec<FetchItem>)>> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.fetch(sequence, items).await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Fetches messages by UIDs.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or fetch fails.
    pub async fn uid_fetch(
        &mut self,
        uids: &UidSet,
        items: FetchItems,
    ) -> Result<Vec<(SeqNum, Vec<FetchItem>)>> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.uid_fetch(uids, items).await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Stores flags on messages.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or store fails.
    pub async fn store(
        &mut self,
        sequence: &SequenceSet,
        action: StoreAction,
    ) -> Result<Vec<(SeqNum, Vec<FetchItem>)>> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.store(sequence, action).await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Copies messages to another mailbox.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or copy fails.
    pub async fn copy(&mut self, sequence: &SequenceSet, mailbox: &str) -> Result<()> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.copy(sequence, mailbox).await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Moves messages to another mailbox.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or move fails.
    pub async fn r#move(&mut self, sequence: &SequenceSet, mailbox: &str) -> Result<()> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.r#move(sequence, mailbox).await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Expunges deleted messages.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state or expunge fails.
    pub async fn expunge(&mut self) -> Result<Vec<SeqNum>> {
        self.ensure_selected().await?;

        match &mut self.state {
            SessionState::Selected(client) => client.expunge().await,
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }

    /// Closes the current mailbox.
    ///
    /// # Errors
    ///
    /// Returns an error if not in selected state.
    pub async fn close(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.state, SessionState::Disconnected) {
            SessionState::Selected(client) => {
                let authenticated = client.close().await?;
                self.state = SessionState::Authenticated(authenticated);
                self.last_mailbox = None;
                Ok(())
            }
            other => {
                self.state = other;
                Ok(())
            }
        }
    }

    /// Disconnects from the server.
    ///
    /// # Errors
    ///
    /// Returns an error if logout fails.
    pub async fn disconnect(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.state, SessionState::Disconnected) {
            SessionState::Selected(client) => {
                client.logout().await?;
            }
            SessionState::Authenticated(client) => {
                client.logout().await?;
            }
            SessionState::Connected(client) => {
                client.logout().await?;
            }
            SessionState::Disconnected => {}
        }
        self.last_mailbox = None;
        Ok(())
    }

    /// Attempts to reconnect to the server.
    ///
    /// # Errors
    ///
    /// Returns an error if reconnection fails after all attempts.
    pub async fn reconnect(&mut self) -> Result<()> {
        self.state = SessionState::Disconnected;
        let last_mailbox = self.last_mailbox.take();

        self.do_reconnect_no_select().await?;

        // Restore mailbox selection if we had one
        if let Some(mailbox) = last_mailbox {
            self.last_mailbox = Some(mailbox.clone());
            // Use internal select to avoid recursion
            if let SessionState::Authenticated(client) =
                std::mem::replace(&mut self.state, SessionState::Disconnected)
            {
                match client.select(&mailbox).await {
                    Ok((selected, _status)) => {
                        self.state = SessionState::Selected(selected);
                    }
                    Err(e) => {
                        tracing::warn!(?e, mailbox, "Failed to reselect mailbox");
                        // Put back authenticated state
                        // Note: we lost the client here, need to reconnect
                        self.do_reconnect_no_select().await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Reconnects without trying to restore mailbox selection (to avoid recursion).
    async fn do_reconnect_no_select(&mut self) -> Result<()> {
        for attempt in 1..=self.config.max_reconnect_attempts {
            tracing::info!(attempt, "Attempting to reconnect");

            if let Err(e) = self.do_connect().await {
                tracing::warn!(?e, "Connection attempt failed");
                if attempt == self.config.max_reconnect_attempts {
                    return Err(e);
                }
                tokio::time::sleep(Duration::from_secs(u64::from(attempt) * 2)).await;
                continue;
            }

            if let Err(e) = self.do_authenticate().await {
                tracing::warn!(?e, "Authentication failed");
                return Err(e);
            }

            return Ok(());
        }

        Err(Error::ConnectionLost(
            "Max reconnection attempts exceeded".into(),
        ))
    }

    // === Private helpers ===

    async fn do_connect(&mut self) -> Result<()> {
        let stream = connect_tls(&self.config.host, self.config.port).await?;
        let client = Client::from_stream(stream).await?;
        self.state = SessionState::Connected(client);
        Ok(())
    }

    async fn do_authenticate(&mut self) -> Result<()> {
        let SessionState::Connected(client) =
            std::mem::replace(&mut self.state, SessionState::Disconnected)
        else {
            return Err(Error::InvalidState("not connected".into()));
        };

        let authenticated = client
            .login(&self.config.username, &self.config.password)
            .await?;
        self.state = SessionState::Authenticated(authenticated);
        Ok(())
    }

    async fn ensure_authenticated(&mut self) -> Result<()> {
        match &self.state {
            SessionState::Authenticated(_) | SessionState::Selected(_) => Ok(()),
            SessionState::Disconnected if self.config.auto_reconnect => {
                self.do_reconnect_no_select().await
            }
            _ => Err(Error::InvalidState("not authenticated".into())),
        }
    }

    async fn ensure_selected(&mut self) -> Result<()> {
        match &self.state {
            SessionState::Selected(_) => Ok(()),
            SessionState::Authenticated(_) => {
                if let Some(mailbox) = self.last_mailbox.clone() {
                    // Internal select to avoid recursion
                    if let SessionState::Authenticated(client) =
                        std::mem::replace(&mut self.state, SessionState::Disconnected)
                    {
                        let (selected, _) = client.select(&mailbox).await?;
                        self.state = SessionState::Selected(selected);
                    }
                    Ok(())
                } else {
                    Err(Error::InvalidState("no mailbox selected".into()))
                }
            }
            SessionState::Disconnected if self.config.auto_reconnect => {
                self.reconnect().await?;
                if matches!(self.state, SessionState::Selected(_)) {
                    Ok(())
                } else {
                    Err(Error::InvalidState("no mailbox selected".into()))
                }
            }
            _ => Err(Error::InvalidState("not in selected state".into())),
        }
    }
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("host", &self.config.host)
            .field("connected", &self.is_connected())
            .field("authenticated", &self.is_authenticated())
            .field("selected_mailbox", &self.selected_mailbox())
            .finish_non_exhaustive()
    }
}
