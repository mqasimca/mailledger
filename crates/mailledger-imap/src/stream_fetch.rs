//! Async streaming fetch for memory-efficient message retrieval.
//!
//! This module provides streaming fetch utilities that yield messages
//! one at a time as they arrive from the server, without buffering the entire
//! response in memory.
//!
//! ## Design (inspired by `imap-next`)
//!
//! Unlike the batched fetch which collects all responses and returns them,
//! the streaming fetch yields each message immediately as it's parsed. This
//! is critical for:
//!
//! - Large mailboxes (thousands of messages)
//! - Limited memory environments
//! - Real-time UI updates

use crate::Error;
use crate::command::FetchItems;
use crate::fetch::FetchResult;
use crate::parser::{FetchItem, Response, ResponseParser, UntaggedResponse};
use crate::types::{SeqNum, Uid};

/// An async stream that yields fetch results one at a time.
///
/// This provides memory-efficient streaming of fetch results without
/// buffering the entire response.
pub struct FetchStreamState {
    /// The tag we're waiting for.
    tag: Option<String>,
    /// Buffer for partial responses.
    buffer: Vec<FetchResult>,
    /// Whether we've received the tagged response.
    complete: bool,
    /// Error that occurred (if any).
    error: Option<Error>,
}

impl FetchStreamState {
    /// Creates a new fetch stream state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tag: None,
            buffer: Vec::new(),
            complete: false,
            error: None,
        }
    }

    /// Processes a raw response and extracts fetch items.
    ///
    /// Returns `Some(FetchResult)` if a FETCH response was parsed.
    pub fn process_response(&mut self, data: &[u8]) -> Option<FetchResult> {
        let response = ResponseParser::parse(data).ok()?;

        match response {
            Response::Untagged(UntaggedResponse::Fetch { seq, items }) => {
                let mut result = FetchResult::new(seq);

                // Extract UID if present
                for item in &items {
                    if let FetchItem::Uid(uid) = item {
                        result.uid = Some(*uid);
                    }
                }
                result.items = items;

                Some(result)
            }
            Response::Tagged {
                tag, status, text, ..
            } => {
                if Some(&tag.as_str().to_string()) == self.tag.as_ref() {
                    self.complete = true;
                    if !matches!(status, crate::types::Status::Ok) {
                        self.error = Some(Error::No(text));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Returns true if the fetch is complete.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// Takes any buffered results.
    pub fn take_buffer(&mut self) -> Vec<FetchResult> {
        std::mem::take(&mut self.buffer)
    }

    /// Buffers a result.
    pub fn buffer_result(&mut self, result: FetchResult) {
        self.buffer.push(result);
    }
}

impl Default for FetchStreamState {
    fn default() -> Self {
        Self::new()
    }
}

/// A message from the fetch stream.
#[derive(Debug, Clone)]
pub struct FetchedMessage {
    /// Sequence number.
    pub seq: SeqNum,
    /// UID (if fetched).
    pub uid: Option<Uid>,
    /// Raw fetch items.
    pub items: Vec<FetchItem>,
}

impl FetchedMessage {
    /// Creates a new fetched message from a result.
    #[must_use]
    pub fn from_result(result: FetchResult) -> Self {
        Self {
            seq: result.seq,
            uid: result.uid,
            items: result.items,
        }
    }

    /// Returns the subject if available in the envelope.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.items.iter().find_map(|item| {
            if let FetchItem::Envelope(env) = item {
                env.subject.as_deref()
            } else {
                None
            }
        })
    }

    /// Returns the sender if available in the envelope.
    #[must_use]
    pub fn from(&self) -> Option<String> {
        self.items.iter().find_map(|item| {
            if let FetchItem::Envelope(env) = item {
                env.from.first().and_then(crate::parser::Address::email)
            } else {
                None
            }
        })
    }

    /// Returns the date if available in the envelope.
    #[must_use]
    pub fn date(&self) -> Option<&str> {
        self.items.iter().find_map(|item| {
            if let FetchItem::Envelope(env) = item {
                env.date.as_deref()
            } else {
                None
            }
        })
    }

    /// Returns the flags.
    #[must_use]
    pub fn flags(&self) -> Option<&crate::types::Flags> {
        self.items.iter().find_map(|item| {
            if let FetchItem::Flags(flags) = item {
                Some(flags)
            } else {
                None
            }
        })
    }

    /// Returns true if the message has been read.
    #[must_use]
    pub fn is_read(&self) -> bool {
        self.flags().is_some_and(|f| {
            f.iter()
                .any(|flag| matches!(flag, crate::types::Flag::Seen))
        })
    }

    /// Returns the message size (RFC822.SIZE).
    #[must_use]
    pub fn size(&self) -> Option<u32> {
        self.items.iter().find_map(|item| {
            if let FetchItem::Rfc822Size(size) = item {
                Some(*size)
            } else {
                None
            }
        })
    }
}

/// Callback type for streaming fetch progress.
pub type FetchCallback = Box<dyn FnMut(FetchedMessage) + Send>;

/// Options for streaming fetch.
#[derive(Debug, Clone)]
pub struct StreamFetchOptions {
    /// Batch size for chunking large requests.
    pub batch_size: usize,
    /// Whether to use UID FETCH.
    pub use_uids: bool,
    /// Items to fetch.
    pub items: FetchItems,
}

impl Default for StreamFetchOptions {
    fn default() -> Self {
        Self {
            batch_size: 100,
            use_uids: true,
            items: FetchItems::Fast,
        }
    }
}

impl StreamFetchOptions {
    /// Creates new options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the batch size.
    #[must_use]
    pub const fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = size;
        self
    }

    /// Sets whether to use UIDs.
    #[must_use]
    pub const fn use_uids(mut self, use_uids: bool) -> Self {
        self.use_uids = use_uids;
        self
    }

    /// Sets the items to fetch.
    #[must_use]
    pub fn items(mut self, items: FetchItems) -> Self {
        self.items = items;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_stream_state_new() {
        let state = FetchStreamState::new();
        assert!(state.tag.is_none());
        assert!(state.buffer.is_empty());
        assert!(!state.is_complete());
    }

    #[test]
    fn test_fetched_message_from_result() {
        let seq = SeqNum::new(1).unwrap();
        let result = FetchResult {
            seq,
            uid: Uid::new(100),
            items: vec![],
        };

        let msg = FetchedMessage::from_result(result);
        assert_eq!(msg.seq, seq);
        assert_eq!(msg.uid, Uid::new(100));
    }

    #[test]
    fn test_stream_fetch_options() {
        let opts = StreamFetchOptions::new()
            .batch_size(50)
            .use_uids(false)
            .items(FetchItems::Fast);

        assert_eq!(opts.batch_size, 50);
        assert!(!opts.use_uids);
    }
}
