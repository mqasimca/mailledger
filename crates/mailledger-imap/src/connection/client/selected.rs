//! Implementation for the selected state.

use tokio::io::{AsyncRead, AsyncWrite};

use super::Client;
use super::states::{Authenticated, Selected};
use crate::Result;
use crate::command::{Command, FetchItems, StoreAction};
use crate::parser::{FetchItem, Response, ResponseParser, UntaggedResponse};
use crate::types::{Mailbox, MailboxStatus, SequenceSet};

impl<S> Client<S, Selected>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Returns the name of the currently selected mailbox.
    #[must_use]
    pub fn mailbox(&self) -> &str {
        self.state.mailbox()
    }

    /// Returns true if the mailbox was opened read-only (via EXAMINE).
    #[must_use]
    pub const fn is_read_only(&self) -> bool {
        self.state.is_read_only()
    }

    /// Returns the number of messages in the mailbox (from SELECT/EXAMINE response).
    #[must_use]
    pub const fn exists(&self) -> u32 {
        self.state.exists()
    }

    /// Returns the number of recent messages (from SELECT/EXAMINE response).
    #[must_use]
    pub const fn recent(&self) -> u32 {
        self.state.recent()
    }

    /// Returns the cached mailbox status from SELECT/EXAMINE.
    ///
    /// Note: This is a snapshot from when the mailbox was selected. For current
    /// status, use the STATUS command or observe unsolicited responses.
    #[must_use]
    pub const fn cached_status(&self) -> &MailboxStatus {
        self.state.status()
    }

    /// Closes the current mailbox and returns to authenticated state.
    ///
    /// This performs an implicit EXPUNGE if the mailbox was opened read-write.
    pub async fn close(mut self) -> Result<Client<S, Authenticated>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Close.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;

        Ok(Client {
            stream: self.stream,
            tag_gen: self.tag_gen,
            capabilities: self.capabilities,
            state: Authenticated,
        })
    }

    /// Selects a different mailbox for read-write access.
    ///
    /// This implicitly closes the current mailbox (without EXPUNGE) and
    /// selects the new one.
    pub async fn select(mut self, mailbox: &str) -> Result<(Self, MailboxStatus)> {
        let tag = self.tag_gen.next();
        let cmd = Command::Select {
            mailbox: Mailbox::new(mailbox),
            condstore: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let status = Client::<S, Authenticated>::parse_mailbox_status(&responses);
        Self::check_tagged_ok(&responses, &tag)?;

        // Update the state with new mailbox info
        self.state = Selected::new(mailbox, false, status.clone());

        Ok((self, status))
    }

    /// Examines a different mailbox for read-only access.
    pub async fn examine(mut self, mailbox: &str) -> Result<(Self, MailboxStatus)> {
        let tag = self.tag_gen.next();
        let cmd = Command::Examine {
            mailbox: Mailbox::new(mailbox),
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let status = Client::<S, Authenticated>::parse_mailbox_status(&responses);
        Self::check_tagged_ok(&responses, &tag)?;

        // Update the state with new mailbox info (read-only)
        self.state = Selected::new(mailbox, true, status.clone());

        Ok((self, status))
    }

    /// Searches for messages matching the given criteria.
    pub async fn search(&mut self, criteria: &str) -> Result<Vec<crate::types::SeqNum>> {
        let tag = self.tag_gen.next();
        let cmd = format!("{tag} SEARCH {criteria}\r\n");
        self.stream.write_command(cmd.as_bytes()).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Search(ids))) =
                ResponseParser::parse(response_bytes)
            {
                results.extend(ids);
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Searches for messages by UID matching the given criteria.
    ///
    /// Returns the UIDs of matching messages.
    pub async fn uid_search(&mut self, criteria: &str) -> Result<Vec<crate::types::Uid>> {
        let tag = self.tag_gen.next();
        let cmd = format!("{tag} UID SEARCH {criteria}\r\n");
        self.stream.write_command(cmd.as_bytes()).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Search(ids))) =
                ResponseParser::parse(response_bytes)
            {
                // UID SEARCH returns UIDs in the SEARCH response
                // The parser returns SeqNum but the values are actually UIDs
                for seq in ids {
                    if let Some(uid) = crate::types::Uid::new(seq.get()) {
                        results.push(uid);
                    }
                }
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Fetches message data for the given sequence set.
    ///
    /// Returns a vector of (sequence number, fetch items) pairs.
    pub async fn fetch(
        &mut self,
        sequence: &SequenceSet,
        items: FetchItems,
    ) -> Result<Vec<(crate::types::SeqNum, Vec<FetchItem>)>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Fetch {
            sequence: sequence.clone(),
            items,
            uid: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Fetch { seq, items })) =
                ResponseParser::parse(response_bytes)
            {
                results.push((seq, items));
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Fetches message data using UIDs.
    ///
    /// Returns a vector of (sequence number, fetch items) pairs.
    pub async fn uid_fetch(
        &mut self,
        uid_set: &crate::types::UidSet,
        items: FetchItems,
    ) -> Result<Vec<(crate::types::SeqNum, Vec<FetchItem>)>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Fetch {
            sequence: uid_set.as_sequence_set(),
            items,
            uid: true,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Fetch { seq, items })) =
                ResponseParser::parse(response_bytes)
            {
                results.push((seq, items));
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Modifies message flags.
    ///
    /// Returns the updated flags for each affected message.
    pub async fn store(
        &mut self,
        sequence: &SequenceSet,
        action: StoreAction,
    ) -> Result<Vec<(crate::types::SeqNum, Vec<FetchItem>)>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Store {
            sequence: sequence.clone(),
            action,
            uid: false,
            silent: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Fetch { seq, items })) =
                ResponseParser::parse(response_bytes)
            {
                results.push((seq, items));
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Modifies message flags silently (no FETCH response).
    pub async fn store_silent(
        &mut self,
        sequence: &SequenceSet,
        action: StoreAction,
    ) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Store {
            sequence: sequence.clone(),
            action,
            uid: false,
            silent: true,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Modifies message flags using UIDs.
    pub async fn uid_store(
        &mut self,
        uid_set: &crate::types::UidSet,
        action: StoreAction,
    ) -> Result<Vec<(crate::types::SeqNum, Vec<FetchItem>)>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Store {
            sequence: uid_set.as_sequence_set(),
            action,
            uid: true,
            silent: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut results = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Fetch { seq, items })) =
                ResponseParser::parse(response_bytes)
            {
                results.push((seq, items));
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(results)
    }

    /// Copies messages to another mailbox.
    pub async fn copy(&mut self, sequence: &SequenceSet, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Copy {
            sequence: sequence.clone(),
            mailbox: Mailbox::new(mailbox),
            uid: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Copies messages to another mailbox using UIDs.
    pub async fn uid_copy(&mut self, uid_set: &crate::types::UidSet, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Copy {
            sequence: uid_set.as_sequence_set(),
            mailbox: Mailbox::new(mailbox),
            uid: true,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Moves messages to another mailbox.
    ///
    /// Requires the MOVE capability (RFC 6851).
    pub async fn r#move(&mut self, sequence: &SequenceSet, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Move {
            sequence: sequence.clone(),
            mailbox: Mailbox::new(mailbox),
            uid: false,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Moves messages to another mailbox using UIDs.
    ///
    /// Requires the MOVE capability (RFC 6851).
    pub async fn uid_move(&mut self, uid_set: &crate::types::UidSet, mailbox: &str) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Move {
            sequence: uid_set.as_sequence_set(),
            mailbox: Mailbox::new(mailbox),
            uid: true,
        }
        .serialize(&tag);

        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        Self::check_tagged_ok(&responses, &tag)?;
        Ok(())
    }

    /// Permanently removes messages marked as \Deleted.
    ///
    /// Returns the sequence numbers of expunged messages.
    pub async fn expunge(&mut self) -> Result<Vec<crate::types::SeqNum>> {
        let tag = self.tag_gen.next();
        let cmd = Command::Expunge.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let responses = self.read_until_tagged(&tag).await?;
        let mut expunged = Vec::new();

        for response_bytes in &responses {
            if let Ok(Response::Untagged(UntaggedResponse::Expunge(seq))) =
                ResponseParser::parse(response_bytes)
            {
                expunged.push(seq);
            }
        }

        Self::check_tagged_ok(&responses, &tag)?;
        Ok(expunged)
    }

    /// Gracefully disconnects from the server.
    pub async fn logout(mut self) -> Result<()> {
        let tag = self.tag_gen.next();
        let cmd = Command::Logout.serialize(&tag);
        self.stream.write_command(&cmd).await?;

        let _ = self.read_until_tagged(&tag).await;
        Ok(())
    }
}
