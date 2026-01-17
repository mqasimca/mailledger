//! Unsolicited response handler for IMAP clients.
//!
//! Per RFC 2683, IMAP clients must be prepared to receive certain responses
//! at any time, including EXISTS, EXPUNGE, and FETCH responses. This module
//! provides a trait for handling these unsolicited responses.
//!
//! # Example
//!
//! ```ignore
//! use mailledger_imap::handler::{ResponseHandler, NoopHandler};
//!
//! struct MyHandler {
//!     message_count: u32,
//! }
//!
//! impl ResponseHandler for MyHandler {
//!     fn on_exists(&mut self, count: u32) {
//!         self.message_count = count;
//!         println!("Mailbox now has {} messages", count);
//!     }
//!     // ... implement other methods
//! }
//! ```

use crate::parser::FetchItem;
use crate::types::{Flags, SeqNum};

/// Handler for unsolicited server responses.
///
/// IMAP servers can send certain responses at any time, not just in response
/// to client commands. Clients must be prepared to handle:
///
/// - `EXISTS` - Message count changes
/// - `EXPUNGE` - Messages removed
/// - `FETCH` - Flag changes on messages
/// - `FLAGS` - Available flags changed
/// - `BYE` - Server is closing connection
/// - `ALERT` - Important message that MUST be displayed to user (RFC 3501)
///
/// Implement this trait to receive callbacks for these events.
pub trait ResponseHandler: Send {
    /// Called when the message count changes (EXISTS response).
    ///
    /// This indicates the total number of messages in the mailbox has changed,
    /// typically because new messages arrived.
    fn on_exists(&mut self, count: u32) {
        let _ = count;
    }

    /// Called when a message is expunged (EXPUNGE response).
    ///
    /// The sequence number refers to the message's position before removal.
    /// Note: Sequence numbers of subsequent messages decrease by one.
    fn on_expunge(&mut self, seq: SeqNum) {
        let _ = seq;
    }

    /// Called when message metadata changes (unsolicited FETCH response).
    ///
    /// This typically indicates flag changes made by another client.
    fn on_fetch(&mut self, seq: SeqNum, items: &[FetchItem]) {
        let _ = (seq, items);
    }

    /// Called when the available flags for the mailbox change.
    fn on_flags(&mut self, flags: &Flags) {
        let _ = flags;
    }

    /// Called when the recent count changes.
    fn on_recent(&mut self, count: u32) {
        let _ = count;
    }

    /// Called when the server is closing the connection (BYE response).
    ///
    /// The connection will be closed after this; no more commands can be sent.
    fn on_bye(&mut self, text: &str) {
        let _ = text;
    }

    /// Called when the server sends an ALERT response code.
    ///
    /// Per RFC 3501, ALERT messages MUST be presented to the user in a way
    /// that calls attention to the message. Do not ignore these!
    fn on_alert(&mut self, text: &str) {
        let _ = text;
    }

    /// Called for any OK response with informational text.
    fn on_ok(&mut self, text: &str) {
        let _ = text;
    }

    /// Called for NO responses (warnings).
    fn on_no(&mut self, text: &str) {
        let _ = text;
    }

    /// Called for BAD responses (errors).
    fn on_bad(&mut self, text: &str) {
        let _ = text;
    }
}

/// A no-op handler that ignores all unsolicited responses.
///
/// Use this when you don't need to handle unsolicited responses,
/// but be aware that you may miss important state changes.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopHandler;

impl ResponseHandler for NoopHandler {}

/// A handler that logs unsolicited responses using tracing.
#[derive(Debug, Default, Clone, Copy)]
pub struct LoggingHandler;

impl ResponseHandler for LoggingHandler {
    fn on_exists(&mut self, count: u32) {
        tracing::debug!(count, "EXISTS");
    }

    fn on_expunge(&mut self, seq: SeqNum) {
        tracing::debug!(seq = seq.get(), "EXPUNGE");
    }

    fn on_fetch(&mut self, seq: SeqNum, items: &[FetchItem]) {
        tracing::debug!(seq = seq.get(), items = ?items, "FETCH");
    }

    fn on_flags(&mut self, flags: &Flags) {
        tracing::debug!(?flags, "FLAGS");
    }

    fn on_recent(&mut self, count: u32) {
        tracing::debug!(count, "RECENT");
    }

    fn on_bye(&mut self, text: &str) {
        tracing::info!(text, "BYE");
    }

    fn on_alert(&mut self, text: &str) {
        tracing::warn!(text, "ALERT");
    }

    fn on_ok(&mut self, text: &str) {
        tracing::trace!(text, "OK");
    }

    fn on_no(&mut self, text: &str) {
        tracing::warn!(text, "NO");
    }

    fn on_bad(&mut self, text: &str) {
        tracing::error!(text, "BAD");
    }
}

/// A handler that collects events for later processing.
///
/// Useful for testing or batch processing of events.
#[derive(Debug, Default, Clone)]
pub struct CollectingHandler {
    /// Collected events.
    pub events: Vec<UnsolicitedEvent>,
}

impl CollectingHandler {
    /// Creates a new collecting handler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all collected events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Takes all collected events, leaving the handler empty.
    pub fn take(&mut self) -> Vec<UnsolicitedEvent> {
        std::mem::take(&mut self.events)
    }
}

impl ResponseHandler for CollectingHandler {
    fn on_exists(&mut self, count: u32) {
        self.events.push(UnsolicitedEvent::Exists(count));
    }

    fn on_expunge(&mut self, seq: SeqNum) {
        self.events.push(UnsolicitedEvent::Expunge(seq));
    }

    fn on_fetch(&mut self, seq: SeqNum, items: &[FetchItem]) {
        self.events
            .push(UnsolicitedEvent::Fetch(seq, items.to_vec()));
    }

    fn on_flags(&mut self, flags: &Flags) {
        self.events.push(UnsolicitedEvent::Flags(flags.clone()));
    }

    fn on_recent(&mut self, count: u32) {
        self.events.push(UnsolicitedEvent::Recent(count));
    }

    fn on_bye(&mut self, text: &str) {
        self.events.push(UnsolicitedEvent::Bye(text.to_string()));
    }

    fn on_alert(&mut self, text: &str) {
        self.events.push(UnsolicitedEvent::Alert(text.to_string()));
    }
}

/// An unsolicited event collected by [`CollectingHandler`].
#[derive(Debug, Clone, PartialEq)]
pub enum UnsolicitedEvent {
    /// EXISTS response.
    Exists(u32),
    /// EXPUNGE response.
    Expunge(SeqNum),
    /// FETCH response with items.
    Fetch(SeqNum, Vec<FetchItem>),
    /// FLAGS response.
    Flags(Flags),
    /// RECENT response.
    Recent(u32),
    /// BYE response.
    Bye(String),
    /// ALERT response code.
    Alert(String),
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_handler() {
        let mut handler = NoopHandler;
        // Should not panic
        handler.on_exists(100);
        handler.on_expunge(SeqNum::new(1).unwrap());
        handler.on_bye("goodbye");
        handler.on_alert("important!");
    }

    #[test]
    fn test_collecting_handler() {
        let mut handler = CollectingHandler::new();

        handler.on_exists(50);
        handler.on_recent(5);
        handler.on_alert("Test alert");

        assert_eq!(handler.events.len(), 3);
        assert_eq!(handler.events[0], UnsolicitedEvent::Exists(50));
        assert_eq!(handler.events[1], UnsolicitedEvent::Recent(5));
        assert_eq!(
            handler.events[2],
            UnsolicitedEvent::Alert("Test alert".to_string())
        );

        let taken = handler.take();
        assert_eq!(taken.len(), 3);
        assert!(handler.events.is_empty());
    }

    #[test]
    fn test_collecting_handler_clear() {
        let mut handler = CollectingHandler::new();
        handler.on_exists(10);
        handler.on_exists(20);
        assert_eq!(handler.events.len(), 2);

        handler.clear();
        assert!(handler.events.is_empty());
    }
}
