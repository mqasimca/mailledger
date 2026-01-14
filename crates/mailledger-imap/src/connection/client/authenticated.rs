//! Implementation for the authenticated state.

use std::fmt::Write;
use std::marker::PhantomData;

use tokio::io::{AsyncRead, AsyncWrite};

use super::Client;
use super::states::{Authenticated, Selected};
use crate::command::Command;
use crate::parser::{Response, ResponseParser, StatusItem, UntaggedResponse};
use crate::types::{Mailbox, MailboxStatus, ResponseCode, Status};
use crate::{Error, Result};

impl<S> Client<S, Authenticated>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Selects a mailbox for read-write access.
    ///
    /// Consumes self and returns a selected client on success.
    pub async fn select(mut self, mailbox: &str) -> Result<(Client<S, Selected>, MailboxStatus)> {
        let tag = self.tag_gen.next();
        let cmd = Command::Select {
            mailbox: Mailbox::new(mailbox),
            condstore: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let status = Self::parse_mailbox_status(&responses);
        Self::check_tagged_ok(&responses, &tag)?;

        Ok((
            Client {
                stream: self.stream,
                tag_gen: self.tag_gen,
                capabilities: self.capabilities,
                _state: PhantomData,
            },
            status,
        ))
    }

    /// Examines a mailbox for read-only access.
    ///
    /// Consumes self and returns a selected client on success.
    pub async fn examine(mut self, mailbox: &str) -> Result<(Client<S, Selected>, MailboxStatus)> {
        let tag = self.tag_gen.next();
        let cmd = Command::Examine {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let status = Self::parse_mailbox_status(&responses);
        Self::check_tagged_ok(&responses, &tag)?;

        Ok((
            Client {
                stream: self.stream,
                tag_gen: self.tag_gen,
                capabilities: self.capabilities,
                _state: PhantomData,
            },
            status,
        ))
    }

    /// Lists mailboxes matching a pattern.
    pub async fn list(
        &mut self,
        reference: &str,
        pattern: &str,
    ) -> Result<Vec<crate::types::ListResponse>> {
        let tag = self.tag_gen.next();
        let cmd = Command::List {
            reference: reference.to_string(),
            pattern: pattern.to_string(),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut list_responses = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::List(item))) =
                ResponseParser::parse(response_bytes)
            {
                list_responses.push(item);
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(list_responses)
    }

    /// Creates a new mailbox.
    pub async fn create(&mut self, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Create {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Deletes a mailbox.
    pub async fn delete(&mut self, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Delete {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Renames a mailbox.
    pub async fn rename(&mut self, from: &str, to: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Rename {
            from: Mailbox::new(from),
            to: Mailbox::new(to),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Subscribes to a mailbox.
    pub async fn subscribe(&mut self, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Subscribe {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Unsubscribes from a mailbox.
    pub async fn unsubscribe(&mut self, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Unsubscribe {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Gets the status of a mailbox without selecting it.
    pub async fn status(
        &mut self,
        mailbox: &str,
        items: Vec<crate::command::StatusAttribute>,
    ) -> Result<Vec<StatusItem>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Status {
            mailbox: Mailbox::new(mailbox),
            items,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut result = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Status { items, .. })) =
                ResponseParser::parse(response_bytes)
            {
                result.extend(items);
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(result)
    }

    /// Appends a message to a mailbox.
    ///
    /// The message should be a complete RFC 5322 message.
    pub async fn append(
        &mut self,
        mailbox: &str,
        flags: Option<Vec<crate::types::Flag>>,
        message: &[u8],
    ) -> Result<()> {
        let tag = self.tag_gen.next();

        // APPEND uses literals which require continuation handling
        // First, send the command with literal size
        let mut cmd = format!("{tag} APPEND ");
        cmd.push_str(&Mailbox::new(mailbox).to_string());
        if let Some(ref f) = flags {
            cmd.push_str(" (");
            for (i, flag) in f.iter().enumerate() {
                if i > 0 {
                    cmd.push(' ');
                }
                cmd.push_str(flag.as_str());
            }
            cmd.push(')');
        }
        // Writing to a String never fails
        let _ = write!(cmd, " {{{}}}\r\n", message.len());

        self.stream.write_command(cmd.as_bytes()).await?;

        // Wait for continuation response
        let response = self.stream.read_response().await?;
        if !response.starts_with(b"+") {
            let parsed = ResponseParser::parse(&response)?;
            if let Response::Tagged { status, text, .. } = parsed {
                return match status {
                    Status::No => Err(Error::No(text)),
                    Status::Bad => Err(Error::Bad(text)),
                    _ => Err(Error::Protocol("unexpected response to APPEND".to_string())),
                };
            }
            return Err(Error::Protocol(
                "expected continuation for APPEND".to_string(),
            ));
        }

        // Send the message data
        self.stream.write_command(message).await?;
        self.stream.write_command(b"\r\n").await?;

        // Read the tagged response
        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Gracefully disconnects from the server.
    pub async fn logout(mut self) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Logout.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let _ = self.read_until_tagged(&tag).await;
        Ok(())
    }

    /// Parses mailbox status from SELECT/EXAMINE responses.
    pub(super) fn parse_mailbox_status(responses: &[Vec<u8>]) -> MailboxStatus {
        let mut status = MailboxStatus::default();

        for response_bytes in responses {
            if let Ok(Response::Untagged(untagged)) = ResponseParser::parse(response_bytes) {
                match untagged {
                    UntaggedResponse::Exists(n) => status.exists = n,
                    UntaggedResponse::Recent(n) => status.recent = n,
                    UntaggedResponse::Flags(flags) => status.flags = flags,
                    UntaggedResponse::Ok {
                        code: Some(code), ..
                    } => match code {
                        ResponseCode::UidValidity(v) => {
                            status.uid_validity = Some(v);
                        }
                        ResponseCode::UidNext(v) => {
                            status.uid_next = Some(v);
                        }
                        ResponseCode::Unseen(v) => {
                            status.unseen = Some(v);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        status
    }
}
