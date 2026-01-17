//! Sans-I/O IMAP protocol implementation.
//!
// Allow missing_const_for_fn since many functions can't be const in stable Rust.
#![allow(clippy::missing_const_for_fn)]
//!
//! This module provides a pure state machine implementation of the IMAP protocol,
//! completely separated from I/O operations. This design enables:
//!
//! - Deterministic testing without network mocks
//! - Time manipulation in tests
//! - Reuse across different async runtimes
//! - Clear separation between protocol logic and I/O
//!
//! # Architecture
//!
//! The protocol is implemented as a state machine that:
//! - Receives bytes via `handle_input()`
//! - Produces bytes to send via `poll_transmit()`
//! - Reports timeouts via `poll_timeout()`
//! - Handles timeouts via `handle_timeout()`
//!
//! # Example
//!
//! ```ignore
//! use mailledger_imap::protocol::{Protocol, ProtocolEvent};
//!
//! let mut protocol = Protocol::new();
//!
//! // Queue a command
//! let handle = protocol.login("user", "pass");
//!
//! // Get bytes to send
//! while let Some(transmit) = protocol.poll_transmit() {
//!     send_to_server(&transmit.data);
//! }
//!
//! // Feed response bytes
//! let events = protocol.handle_input(response_bytes);
//! for event in events {
//!     match event {
//!         ProtocolEvent::CommandComplete { tag, result } => { /* ... */ }
//!         ProtocolEvent::Unsolicited(resp) => { /* ... */ }
//!     }
//! }
//! ```

mod state;
mod transmit;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub use state::{ProtocolState, SelectedState};
pub use transmit::Transmit;

use crate::command::{Command, TagGenerator};
use crate::handler::ResponseHandler;
use crate::parser::{Response, ResponseParser, UntaggedResponse};
use crate::types::{Capability, MailboxStatus, ResponseCode, Status, Tag};
use crate::{Error, Result};

/// A handle to a pending command.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandHandle {
    tag: Tag,
}

impl CommandHandle {
    /// Returns the tag associated with this command.
    #[must_use]
    pub fn tag(&self) -> &Tag {
        &self.tag
    }
}

/// Result of a completed command.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command succeeded.
    pub status: Status,
    /// Optional response code.
    pub code: Option<ResponseCode>,
    /// Human-readable text.
    pub text: String,
    /// Collected untagged responses for this command.
    pub responses: Vec<UntaggedResponse>,
}

impl CommandResult {
    /// Returns true if the command succeeded (OK status).
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self.status, Status::Ok | Status::PreAuth)
    }

    /// Converts to a Result, returning an error if the command failed.
    ///
    /// # Errors
    ///
    /// Returns an error if the status is NO, BAD, or BYE.
    pub fn into_result(self) -> Result<Vec<UntaggedResponse>> {
        match self.status {
            Status::Ok | Status::PreAuth => Ok(self.responses),
            Status::No => Err(Error::No(self.text)),
            Status::Bad => Err(Error::Bad(self.text)),
            Status::Bye => Err(Error::Bye(self.text)),
        }
    }
}

/// Events produced by the protocol state machine.
#[derive(Debug)]
pub enum ProtocolEvent {
    /// A command completed.
    CommandComplete {
        /// The command handle.
        handle: CommandHandle,
        /// The result.
        result: CommandResult,
    },
    /// Server greeting received (initial connection).
    Greeting {
        /// Greeting status.
        status: Status,
        /// Optional response code.
        code: Option<ResponseCode>,
        /// Greeting text.
        text: String,
    },
    /// Continuation request from server (for literals, IDLE, etc.).
    Continuation {
        /// Continuation text.
        text: String,
    },
    /// Connection closed by server.
    Disconnected {
        /// BYE message text.
        text: String,
    },
}

/// A pending command waiting for completion.
struct PendingCommand {
    handle: CommandHandle,
    responses: Vec<UntaggedResponse>,
}

/// Sans-I/O IMAP protocol state machine.
///
/// This struct manages the IMAP protocol state without performing any I/O.
/// Feed it bytes, and it will produce bytes to send and events to process.
pub struct Protocol {
    /// Current protocol state.
    state: ProtocolState,
    /// Tag generator for commands.
    tag_gen: TagGenerator,
    /// Server capabilities.
    capabilities: Vec<Capability>,
    /// Pending commands awaiting responses.
    pending: VecDeque<PendingCommand>,
    /// Outbound data queue.
    outbound: VecDeque<Transmit>,
    /// Inbound buffer for partial data.
    inbound: Vec<u8>,
    /// Whether we've received the initial greeting.
    greeting_received: bool,
    /// IDLE state tracking.
    idle_tag: Option<Tag>,
    /// Last activity time (for timeout tracking).
    last_activity: Option<Instant>,
    /// Current mailbox status (when selected).
    mailbox_status: Option<MailboxStatus>,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::new()
    }
}

impl Protocol {
    /// Creates a new protocol instance in the not-authenticated state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: ProtocolState::NotAuthenticated,
            tag_gen: TagGenerator::default(),
            capabilities: Vec::new(),
            pending: VecDeque::new(),
            outbound: VecDeque::new(),
            inbound: Vec::new(),
            greeting_received: false,
            idle_tag: None,
            last_activity: None,
            mailbox_status: None,
        }
    }

    /// Returns the current protocol state.
    #[must_use]
    pub fn state(&self) -> &ProtocolState {
        &self.state
    }

    /// Returns the server capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Checks if the server has a specific capability.
    #[must_use]
    pub fn has_capability(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Returns the current mailbox status (when selected).
    #[must_use]
    pub fn mailbox_status(&self) -> Option<&MailboxStatus> {
        self.mailbox_status.as_ref()
    }

    /// Returns whether we're in IDLE mode.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.idle_tag.is_some()
    }

    /// Returns the next timeout, if any.
    ///
    /// Returns `None` if no timeout is pending.
    /// The caller should call `handle_timeout()` when this instant is reached.
    #[must_use]
    pub fn poll_timeout(&self) -> Option<Instant> {
        // IDLE should be refreshed every 29 minutes per RFC 2177
        if self.idle_tag.is_some() {
            self.last_activity.map(|t| t + Duration::from_secs(29 * 60))
        } else {
            None
        }
    }

    /// Handles a timeout expiration.
    ///
    /// Call this when `poll_timeout()` returns an instant that has passed.
    pub fn handle_timeout(&mut self, _now: Instant) {
        // Currently only used for IDLE timeout tracking
        // The actual timeout handling is done by the caller
    }

    /// Returns the next data to transmit, if any.
    pub fn poll_transmit(&mut self) -> Option<Transmit> {
        self.outbound.pop_front()
    }

    /// Feeds received data into the protocol.
    ///
    /// Returns a list of events produced by processing the data.
    pub fn handle_input(
        &mut self,
        data: &[u8],
        handler: &mut dyn ResponseHandler,
    ) -> Vec<ProtocolEvent> {
        self.inbound.extend_from_slice(data);
        self.last_activity = Some(Instant::now());

        let mut events = Vec::new();

        // Process complete lines
        while let Some(line_end) = self.find_complete_response() {
            let response_data: Vec<u8> = self.inbound.drain(..=line_end).collect();

            if let Some(event) = self.process_response(&response_data, handler) {
                events.push(event);
            }
        }

        events
    }

    /// Finds the end of a complete response in the inbound buffer.
    fn find_complete_response(&self) -> Option<usize> {
        // Look for CRLF
        for i in 0..self.inbound.len().saturating_sub(1) {
            if self.inbound[i] == b'\r' && self.inbound[i + 1] == b'\n' {
                // Check for literal
                if let Some(literal_len) = self.parse_literal_at_end(&self.inbound[..=i + 1]) {
                    // Need more data for the literal
                    let total_needed = i + 2 + literal_len;
                    if self.inbound.len() >= total_needed {
                        // Have the literal, look for the next CRLF
                        for j in total_needed..self.inbound.len().saturating_sub(1) {
                            if self.inbound[j] == b'\r' && self.inbound[j + 1] == b'\n' {
                                return Some(j + 1);
                            }
                        }
                    }
                    return None;
                }
                return Some(i + 1);
            }
        }
        None
    }

    /// Parses a literal length from the end of a line.
    #[allow(clippy::unused_self)] // Method for potential future use of self
    fn parse_literal_at_end(&self, line: &[u8]) -> Option<usize> {
        if !line.ends_with(b"\r\n") {
            return None;
        }
        let line = &line[..line.len() - 2];

        let open = line.iter().rposition(|&b| b == b'{')?;
        if !line.ends_with(b"}") && !line.ends_with(b"+}") {
            return None;
        }

        let num_start = open + 1;
        let num_end = if line.ends_with(b"+}") {
            line.len() - 2
        } else {
            line.len() - 1
        };

        let num_str = std::str::from_utf8(&line[num_start..num_end]).ok()?;
        num_str.parse().ok()
    }

    /// Processes a complete response.
    fn process_response(
        &mut self,
        data: &[u8],
        handler: &mut dyn ResponseHandler,
    ) -> Option<ProtocolEvent> {
        let Ok(response) = ResponseParser::parse(data) else {
            return None;
        };

        match response {
            Response::Tagged {
                tag,
                status,
                code,
                text,
            } => self.handle_tagged(tag, status, code, text),

            Response::Untagged(untagged) => {
                self.handle_untagged(untagged, handler);
                None
            }

            Response::Continuation { text } => Some(ProtocolEvent::Continuation {
                text: text.unwrap_or_default(),
            }),
        }
    }

    /// Handles a tagged response.
    #[allow(clippy::needless_pass_by_value)] // Tag is small and consumed in comparisons
    fn handle_tagged(
        &mut self,
        tag: Tag,
        status: Status,
        code: Option<ResponseCode>,
        text: String,
    ) -> Option<ProtocolEvent> {
        // Check for IDLE completion
        if self.idle_tag.as_ref() == Some(&tag) {
            self.idle_tag = None;
        }

        // Find the pending command
        let position = self.pending.iter().position(|p| p.handle.tag == tag)?;

        let pending = self.pending.remove(position)?;

        // Update state based on command result
        if status == Status::Ok {
            self.update_state_on_success(&pending.handle.tag);
        }

        Some(ProtocolEvent::CommandComplete {
            handle: pending.handle,
            result: CommandResult {
                status,
                code,
                text,
                responses: pending.responses,
            },
        })
    }

    /// Updates protocol state after a successful command.
    #[allow(clippy::needless_pass_by_ref_mut, clippy::unused_self)]
    fn update_state_on_success(&mut self, _tag: &Tag) {
        // State transitions are handled by the specific command methods
    }

    /// Handles an untagged response.
    fn handle_untagged(&mut self, response: UntaggedResponse, handler: &mut dyn ResponseHandler) {
        // First, notify the handler
        match &response {
            UntaggedResponse::Exists(n) => handler.on_exists(*n),
            UntaggedResponse::Recent(n) => handler.on_recent(*n),
            UntaggedResponse::Expunge(seq) => handler.on_expunge(*seq),
            UntaggedResponse::Fetch { seq, items } => handler.on_fetch(*seq, items),
            UntaggedResponse::Flags(flags) => handler.on_flags(flags),
            UntaggedResponse::Bye { text, .. } => handler.on_bye(text),
            UntaggedResponse::Ok { code, text } => {
                if matches!(code, Some(ResponseCode::Alert)) {
                    handler.on_alert(text);
                } else {
                    handler.on_ok(text);
                }
            }
            UntaggedResponse::No { text, .. } => handler.on_no(text),
            UntaggedResponse::Bad { text, .. } => handler.on_bad(text),
            UntaggedResponse::Capability(caps) => {
                self.capabilities.clone_from(caps);
            }
            _ => {}
        }

        // Update mailbox status
        if let Some(status) = &mut self.mailbox_status {
            match &response {
                UntaggedResponse::Exists(n) => status.exists = *n,
                UntaggedResponse::Recent(n) => status.recent = *n,
                UntaggedResponse::Flags(flags) => status.flags = flags.clone(),
                _ => {}
            }
        }

        // Add to pending command responses (if any command is pending)
        if let Some(pending) = self.pending.back_mut() {
            pending.responses.push(response);
        }
    }

    /// Queues a command for sending.
    fn queue_command(&mut self, cmd: &Command) -> CommandHandle {
        let tag = self.tag_gen.next();
        let data = cmd.serialize(&tag);

        self.outbound.push_back(Transmit { data });

        let handle = CommandHandle {
            tag: Tag::new(&tag),
        };

        self.pending.push_back(PendingCommand {
            handle: handle.clone(),
            responses: Vec::new(),
        });

        handle
    }

    // === Command Methods ===

    /// Queues a LOGIN command.
    pub fn login(&mut self, username: &str, password: &str) -> CommandHandle {
        // State transition happens on successful response
        self.queue_command(&Command::Login {
            username: username.to_string(),
            password: password.to_string(),
        })
    }

    /// Queues a CAPABILITY command.
    pub fn capability(&mut self) -> CommandHandle {
        self.queue_command(&Command::Capability)
    }

    /// Queues a NOOP command.
    pub fn noop(&mut self) -> CommandHandle {
        self.queue_command(&Command::Noop)
    }

    /// Queues a LOGOUT command.
    pub fn logout(&mut self) -> CommandHandle {
        self.queue_command(&Command::Logout)
    }

    /// Queues a SELECT command.
    pub fn select(&mut self, mailbox: &str) -> CommandHandle {
        use crate::types::Mailbox;
        self.mailbox_status = Some(MailboxStatus::default());
        self.queue_command(&Command::Select {
            mailbox: Mailbox::new(mailbox),
            condstore: self.has_capability(&Capability::CondStore),
        })
    }

    /// Queues an EXAMINE command.
    pub fn examine(&mut self, mailbox: &str) -> CommandHandle {
        use crate::types::Mailbox;
        self.mailbox_status = Some(MailboxStatus::default());
        self.queue_command(&Command::Examine {
            mailbox: Mailbox::new(mailbox),
        })
    }

    /// Queues a LIST command.
    pub fn list(&mut self, reference: &str, pattern: &str) -> CommandHandle {
        self.queue_command(&Command::List {
            reference: reference.to_string(),
            pattern: pattern.to_string(),
        })
    }

    /// Queues a CLOSE command.
    pub fn close(&mut self) -> CommandHandle {
        self.mailbox_status = None;
        self.queue_command(&Command::Close)
    }

    /// Queues an IDLE command.
    pub fn idle(&mut self) -> CommandHandle {
        let handle = self.queue_command(&Command::Idle);
        self.idle_tag = Some(handle.tag.clone());
        self.last_activity = Some(Instant::now());
        handle
    }

    /// Queues a DONE command (to exit IDLE).
    pub fn done(&mut self) {
        self.outbound.push_back(Transmit {
            data: b"DONE\r\n".to_vec(),
        });
        // idle_tag will be cleared when we receive the tagged response
    }

    /// Transitions to authenticated state.
    pub fn set_authenticated(&mut self) {
        self.state = ProtocolState::Authenticated;
    }

    /// Transitions to selected state.
    pub fn set_selected(&mut self, mailbox: String, read_only: bool) {
        self.state = ProtocolState::Selected(SelectedState { mailbox, read_only });
    }

    /// Transitions back to authenticated state (from selected).
    pub fn set_unselected(&mut self) {
        if matches!(self.state, ProtocolState::Selected(_)) {
            self.state = ProtocolState::Authenticated;
            self.mailbox_status = None;
        }
    }
}

impl std::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Protocol")
            .field("state", &self.state)
            .field("capabilities", &self.capabilities)
            .field("pending_count", &self.pending.len())
            .field("outbound_count", &self.outbound.len())
            .field("greeting_received", &self.greeting_received)
            .field("is_idle", &self.idle_tag.is_some())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;
    use crate::handler::NoopHandler;

    #[test]
    fn test_protocol_new() {
        let protocol = Protocol::new();
        assert!(matches!(protocol.state(), ProtocolState::NotAuthenticated));
        assert!(protocol.capabilities().is_empty());
    }

    #[test]
    fn test_queue_command() {
        let mut protocol = Protocol::new();
        let handle = protocol.noop();

        // Should have data to transmit
        let transmit = protocol.poll_transmit();
        assert!(transmit.is_some());

        let transmit_data = transmit.unwrap();
        let data = String::from_utf8_lossy(&transmit_data.data);
        assert!(data.contains("NOOP"));
        assert!(data.contains(handle.tag().as_str()));
    }

    #[test]
    fn test_handle_tagged_response() {
        let mut protocol = Protocol::new();
        let mut handler = NoopHandler;

        let handle = protocol.noop();
        let tag = handle.tag().as_str().to_string();

        // Consume the outbound data
        let _ = protocol.poll_transmit();

        // Feed a response
        let response = format!("{tag} OK NOOP completed\r\n");
        let events = protocol.handle_input(response.as_bytes(), &mut handler);

        assert_eq!(events.len(), 1);
        if let ProtocolEvent::CommandComplete { result, .. } = &events[0] {
            assert!(result.is_ok());
        } else {
            panic!("Expected CommandComplete event");
        }
    }

    #[test]
    fn test_handle_untagged_exists() {
        let mut protocol = Protocol::new();
        let mut handler = crate::handler::CollectingHandler::new();

        let response = b"* 150 EXISTS\r\n";
        protocol.handle_input(response, &mut handler);

        assert_eq!(handler.events.len(), 1);
    }
}
