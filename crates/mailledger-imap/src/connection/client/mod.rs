//! Type-state IMAP client connection.
//!
//! Uses the type-state pattern to enforce valid state transitions at compile time.
//! The IMAP connection states are:
//!
//! - `NotAuthenticated`: Initial state after connection
//! - `Authenticated`: After successful LOGIN/AUTHENTICATE
//! - `Selected`: After successful SELECT/EXAMINE
//!
//! Each state only exposes methods that are valid for that state.

#![allow(clippy::missing_errors_doc)]

mod authenticated;
mod not_authenticated;
mod selected;
mod states;

use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncWrite};

pub use self::states::{Authenticated, NotAuthenticated, Selected};
use super::framed::FramedStream;
use crate::command::{Command, TagGenerator};
use crate::parser::{Response, ResponseParser, UntaggedResponse};
use crate::types::{Capability, Status};
use crate::{Error, Result};

/// IMAP client connection with type-state.
///
/// The type parameter `State` tracks the connection state at compile time.
pub struct Client<S, State> {
    pub(crate) stream: FramedStream<S>,
    pub(crate) tag_gen: TagGenerator,
    pub(crate) capabilities: Vec<Capability>,
    _state: PhantomData<State>,
}

// Manual Debug implementation since FramedStream doesn't implement Debug
impl<S, State> std::fmt::Debug for Client<S, State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("tag_gen", &self.tag_gen)
            .field("capabilities", &self.capabilities)
            .finish_non_exhaustive()
    }
}

/// Shared implementation for all states.
impl<S, State> Client<S, State>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
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

    /// Returns true if the server supports `IMAP4rev2`.
    #[must_use]
    pub fn supports_imap4rev2(&self) -> bool {
        self.has_capability(&Capability::Imap4Rev2)
    }

    /// Returns true if the server supports IDLE (RFC 2177).
    #[must_use]
    pub fn supports_idle(&self) -> bool {
        self.has_capability(&Capability::Idle)
    }

    /// Returns true if the server supports MOVE (RFC 6851).
    #[must_use]
    pub fn supports_move(&self) -> bool {
        self.has_capability(&Capability::Move)
    }

    /// Returns true if the server supports NAMESPACE (RFC 2342).
    #[must_use]
    pub fn supports_namespace(&self) -> bool {
        self.has_capability(&Capability::Namespace)
    }

    /// Returns true if the server supports CONDSTORE (RFC 7162).
    #[must_use]
    pub fn supports_condstore(&self) -> bool {
        self.has_capability(&Capability::CondStore)
    }

    /// Returns true if the server supports UIDPLUS (RFC 4315).
    #[must_use]
    pub fn supports_uidplus(&self) -> bool {
        self.has_capability(&Capability::UidPlus)
    }

    /// Returns true if LOGIN is disabled (e.g., before STARTTLS).
    #[must_use]
    pub fn login_disabled(&self) -> bool {
        self.has_capability(&Capability::LoginDisabled)
    }

    /// Returns true if the server supports AUTH=PLAIN (SASL PLAIN mechanism).
    #[must_use]
    pub fn supports_auth_plain(&self) -> bool {
        self.capabilities
            .iter()
            .any(|c| matches!(c, Capability::Auth(m) if m.eq_ignore_ascii_case("PLAIN")))
    }

    /// Sends a NOOP command to keep the connection alive.
    pub async fn noop(&mut self) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Noop.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;

        Ok(())
    }

    /// Sends a CAPABILITY command and updates the stored capabilities.
    pub async fn capability(&mut self) -> Result<Vec<Capability>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Capability.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;

        // Parse capabilities from untagged responses
        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Capability(caps))) =
                ResponseParser::parse(response_bytes)
            {
                self.capabilities.clone_from(&caps);
                return Ok(caps);
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(self.capabilities.clone())
    }

    /// Reads responses until we get a tagged response matching our tag.
    pub(crate) async fn read_until_tagged(&mut self, tag: &str) -> Result<Vec<Vec<u8>>> {
        let mut accumulator = super::framed::ResponseAccumulator::new(tag);
        accumulator.read_until_tagged(&mut self.stream).await
    }

    /// Checks that the tagged response is OK.
    pub(crate) fn check_tagged_ok(responses: &[Vec<u8>], tag: &str) -> Result<()> {
        // Find the tagged response (should be the last one)
        for response_bytes in responses.iter().rev() {
            if let Ok(Response::Tagged {
                tag: resp_tag,
                status,
                code: _,
                text,
            }) = ResponseParser::parse(response_bytes)
                && resp_tag.as_str() == tag
            {
                return match status {
                    Status::Ok | Status::PreAuth => Ok(()),
                    Status::No => Err(Error::No(text)),
                    Status::Bad => Err(Error::Bad(text)),
                    Status::Bye => Err(Error::Bye(text)),
                };
            }
        }

        Err(Error::Protocol("missing tagged response".to_string()))
    }
}
