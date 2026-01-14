//! IMAP IDLE command support (RFC 2177).
//!
//! IDLE allows the client to receive real-time notifications from the server
//! about mailbox changes without polling.

#![allow(clippy::missing_errors_doc)]

use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::timeout;

use super::client::{Client, Selected};
use super::framed::FramedStream;
use crate::parser::{Response, ResponseParser, UntaggedResponse};
use crate::types::{Flags, SeqNum};
use crate::{Error, Result};

/// Event received during IDLE.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdleEvent {
    /// New message count (EXISTS response).
    Exists(u32),
    /// Message expunged (EXPUNGE response).
    Expunge(SeqNum),
    /// Message flags changed (FETCH response).
    Fetch {
        /// Message sequence number.
        seq: SeqNum,
        /// Updated flags.
        flags: Flags,
    },
    /// Recent count changed.
    Recent(u32),
    /// Timeout occurred without receiving an event.
    Timeout,
}

/// Handle for an active IDLE session.
///
/// This type holds a mutable reference to the client and manages the IDLE state.
/// Call `wait()` to receive events, and `done()` to exit IDLE mode.
pub struct IdleHandle<'a, S> {
    stream: &'a mut FramedStream<S>,
    tag: String,
}

impl<'a, S> IdleHandle<'a, S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new IDLE handle.
    pub(crate) const fn new(stream: &'a mut FramedStream<S>, tag: String) -> Self {
        Self { stream, tag }
    }

    /// Waits for a server event or timeout.
    ///
    /// This method blocks until the server sends an untagged response
    /// (EXISTS, EXPUNGE, FETCH, etc.) or the specified timeout elapses.
    ///
    /// # Notes
    ///
    /// RFC 2177 recommends re-issuing IDLE every 29 minutes. Most servers
    /// have shorter timeouts (10-30 minutes), so consider using shorter
    /// timeouts in practice.
    pub async fn wait(&mut self, duration: Duration) -> Result<IdleEvent> {
        match timeout(duration, self.stream.read_response()).await {
            Ok(Ok(response)) => self.parse_event(&response),
            Ok(Err(e)) => Err(e),
            Err(_) => Ok(IdleEvent::Timeout),
        }
    }

    /// Parses a response into an `IdleEvent`.
    fn parse_event(&self, response: &[u8]) -> Result<IdleEvent> {
        match ResponseParser::parse(response)? {
            Response::Untagged(untagged) => match untagged {
                UntaggedResponse::Exists(n) => Ok(IdleEvent::Exists(n)),
                UntaggedResponse::Recent(n) => Ok(IdleEvent::Recent(n)),
                UntaggedResponse::Expunge(seq) => Ok(IdleEvent::Expunge(seq)),
                UntaggedResponse::Fetch { seq, items } => {
                    // Extract flags from fetch items
                    let flags = items
                        .into_iter()
                        .find_map(|item| {
                            if let crate::parser::FetchItem::Flags(f) = item {
                                Some(f)
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    Ok(IdleEvent::Fetch { seq, flags })
                }
                _ => {
                    // Ignore other untagged responses during IDLE
                    // This shouldn't happen often, but we handle it gracefully
                    Ok(IdleEvent::Timeout)
                }
            },
            Response::Continuation { .. } => {
                // Continuation during IDLE is unexpected
                Err(Error::Protocol(
                    "unexpected continuation during IDLE".to_string(),
                ))
            }
            Response::Tagged {
                tag, status, text, ..
            } => {
                // If we receive a tagged response, IDLE was terminated by the server
                if tag.as_str() == self.tag {
                    match status {
                        crate::types::Status::Ok => {
                            // Server terminated IDLE normally (unusual but valid)
                            Ok(IdleEvent::Timeout)
                        }
                        crate::types::Status::No => Err(Error::No(text)),
                        crate::types::Status::Bad => Err(Error::Bad(text)),
                        crate::types::Status::Bye => Err(Error::Bye(text)),
                        crate::types::Status::PreAuth => {
                            Err(Error::Protocol("unexpected PREAUTH in IDLE".to_string()))
                        }
                    }
                } else {
                    Err(Error::Protocol(format!(
                        "unexpected tag {} during IDLE",
                        tag.as_str()
                    )))
                }
            }
        }
    }

    /// Exits IDLE mode by sending DONE.
    ///
    /// This consumes the handle and returns control to the client.
    /// After calling `done()`, the client can issue other commands.
    pub async fn done(self) -> Result<()> {
        use crate::Command;

        // Send DONE (no tag)
        let cmd = Command::Done.serialize("");
        self.stream.write_command(&cmd).await?;

        // Read the tagged response
        loop {
            let response = self.stream.read_response().await?;
            if let Ok(Response::Tagged {
                tag, status, text, ..
            }) = ResponseParser::parse(&response)
                && tag.as_str() == self.tag
            {
                return match status {
                    crate::types::Status::Ok => Ok(()),
                    crate::types::Status::No => Err(Error::No(text)),
                    crate::types::Status::Bad => Err(Error::Bad(text)),
                    crate::types::Status::Bye => Err(Error::Bye(text)),
                    crate::types::Status::PreAuth => {
                        Err(Error::Protocol("unexpected PREAUTH after DONE".to_string()))
                    }
                };
            }
            // Ignore untagged responses that may arrive before the tagged response
        }
    }
}

/// Extension trait for adding IDLE support to the Selected client.
impl<S> Client<S, Selected>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Enters IDLE mode for real-time notifications.
    ///
    /// Returns an `IdleHandle` that can be used to wait for events.
    /// Call `done()` on the handle to exit IDLE mode.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut handle = client.idle().await?;
    /// loop {
    ///     match handle.wait(Duration::from_secs(600)).await? {
    ///         IdleEvent::Exists(n) => println!("New message count: {}", n),
    ///         IdleEvent::Timeout => break,
    ///         _ => {}
    ///     }
    /// }
    /// handle.done().await?;
    /// ```
    ///
    /// # Notes
    ///
    /// - Check `supports_idle()` before calling this method
    /// - IDLE should be re-issued periodically (every 10-29 minutes)
    /// - Some servers may drop idle connections after extended periods
    pub async fn idle(&mut self) -> Result<IdleHandle<'_, S>> {
        use crate::Command;

        let tag = self.tag_gen.next();
        let cmd = Command::Idle.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        // Wait for continuation response
        let response = self.stream.read_response().await?;
        if !response.starts_with(b"+") {
            let parsed = ResponseParser::parse(&response)?;
            if let Response::Tagged { status, text, .. } = parsed {
                return match status {
                    crate::types::Status::No => Err(Error::No(text)),
                    crate::types::Status::Bad => Err(Error::Bad(text)),
                    _ => Err(Error::Protocol("unexpected response to IDLE".to_string())),
                };
            }
            return Err(Error::Protocol(
                "expected continuation for IDLE".to_string(),
            ));
        }

        Ok(IdleHandle::new(&mut self.stream, tag))
    }
}
