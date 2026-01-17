//! Batched fetch operations for efficient message retrieval.
//!
// Allow missing_const_for_fn since many can't be const due to Vec operations in stable Rust.
#![allow(clippy::missing_const_for_fn)]
//!
//! This module provides utilities for efficiently fetching message data
//! in batches, which is critical for good IMAP performance per RFC 2683.
//!
//! ## Why Batching Matters
//!
//! IMAP servers can handle large fetch requests, but:
//! - Very large requests may timeout
//! - Memory usage spikes with large responses
//! - Progress reporting needs intermediate results
//!
//! ## Strategy
//!
//! The batched fetcher:
//! 1. Splits large UID ranges into manageable chunks
//! 2. Pipelines multiple fetch requests (when enabled)
//! 3. Streams results as they arrive
//! 4. Reports progress for UI updates
//!
//! # Example
//!
//! ```ignore
//! use mailledger_imap::fetch::{BatchedFetch, FetchProgress};
//!
//! // Fetch all messages in batches of 50
//! let fetcher = BatchedFetch::new()
//!     .batch_size(50)
//!     .items(FetchItems::Envelope);
//!
//! let mut stream = fetcher.fetch(&mut client, "1:*").await?;
//! while let Some(result) = stream.next().await {
//!     match result {
//!         FetchProgress::Item(msg) => println!("Got: {}", msg.uid),
//!         FetchProgress::BatchComplete(n) => println!("Completed batch {n}"),
//!         FetchProgress::Error(e) => eprintln!("Error: {e}"),
//!     }
//! }
//! ```

use std::collections::VecDeque;
use std::num::NonZeroUsize;

use crate::command::FetchItems;
use crate::parser::FetchItem;
use crate::types::{SeqNum, Uid};

/// Default batch size for fetch operations.
pub const DEFAULT_BATCH_SIZE: usize = 100;

/// Maximum recommended batch size.
pub const MAX_BATCH_SIZE: usize = 500;

/// Configuration for batched fetch operations.
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Number of messages per batch.
    pub batch_size: NonZeroUsize,

    /// Items to fetch for each message.
    pub items: FetchItems,

    /// Whether to use UID FETCH (vs sequence-based FETCH).
    pub use_uids: bool,

    /// Maximum number of pipelined requests.
    pub pipeline_depth: usize,

    /// Whether to report progress after each batch.
    pub report_progress: bool,
}

/// Compile-time verified default batch size.
const DEFAULT_BATCH_SIZE_NONZERO: NonZeroUsize = {
    // SAFETY: DEFAULT_BATCH_SIZE is 100, which is non-zero.
    match NonZeroUsize::new(DEFAULT_BATCH_SIZE) {
        Some(v) => v,
        None => panic!("DEFAULT_BATCH_SIZE must be non-zero"),
    }
};

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: DEFAULT_BATCH_SIZE_NONZERO,
            items: FetchItems::Fast,
            use_uids: true,
            pipeline_depth: 1,
            report_progress: true,
        }
    }
}

impl BatchConfig {
    /// Creates a new batch configuration with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the batch size.
    #[must_use]
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_size = NonZeroUsize::new(size.min(MAX_BATCH_SIZE)).unwrap_or(self.batch_size);
        self
    }

    /// Sets the items to fetch.
    #[must_use]
    pub fn items(mut self, items: FetchItems) -> Self {
        self.items = items;
        self
    }

    /// Sets whether to use UID FETCH.
    #[must_use]
    pub fn use_uids(mut self, use_uids: bool) -> Self {
        self.use_uids = use_uids;
        self
    }

    /// Sets the pipeline depth (number of concurrent requests).
    #[must_use]
    pub fn pipeline_depth(mut self, depth: usize) -> Self {
        self.pipeline_depth = depth.max(1);
        self
    }

    /// Sets whether to report progress after each batch.
    #[must_use]
    pub fn report_progress(mut self, report: bool) -> Self {
        self.report_progress = report;
        self
    }
}

/// Builder for batched fetch operations.
#[derive(Debug, Clone)]
pub struct BatchedFetch {
    config: BatchConfig,
}

impl Default for BatchedFetch {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchedFetch {
    /// Creates a new batched fetch builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BatchConfig::default(),
        }
    }

    /// Sets the batch size.
    #[must_use]
    pub fn batch_size(mut self, size: usize) -> Self {
        self.config = self.config.batch_size(size);
        self
    }

    /// Sets the items to fetch.
    #[must_use]
    pub fn items(mut self, items: FetchItems) -> Self {
        self.config = self.config.items(items);
        self
    }

    /// Sets whether to use UID FETCH.
    #[must_use]
    pub fn use_uids(mut self, use_uids: bool) -> Self {
        self.config = self.config.use_uids(use_uids);
        self
    }

    /// Sets the pipeline depth.
    #[must_use]
    pub fn pipeline_depth(mut self, depth: usize) -> Self {
        self.config = self.config.pipeline_depth(depth);
        self
    }

    /// Returns the configuration.
    #[must_use]
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// Creates batches from a range specification.
    ///
    /// Parses a range like "1:100" or "1:*" and creates appropriate batches.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn create_range_batches(&self, total: u32) -> Vec<(u32, u32)> {
        if total == 0 {
            return Vec::new();
        }

        // batch_size is capped at MAX_BATCH_SIZE (500), so truncation is safe
        let batch_size = self.config.batch_size.get() as u32;
        let mut batches = Vec::new();
        let mut start = 1u32;

        while start <= total {
            let end = (start + batch_size - 1).min(total);
            batches.push((start, end));
            start = end + 1;
        }

        batches
    }
}

/// Progress event during batched fetch.
#[derive(Debug)]
pub enum FetchProgress {
    /// A single fetch result.
    Item(FetchResult),

    /// A batch has completed.
    BatchComplete {
        /// Batch index (0-based).
        batch_index: usize,
        /// Total batches.
        total_batches: usize,
        /// Items fetched in this batch.
        items_fetched: usize,
    },

    /// All batches complete.
    Complete {
        /// Total items fetched.
        total_items: usize,
    },

    /// An error occurred during fetch.
    Error(crate::Error),
}

/// A single fetch result.
#[derive(Debug, Clone)]
pub struct FetchResult {
    /// Message sequence number.
    pub seq: SeqNum,

    /// Message UID (if fetched).
    pub uid: Option<Uid>,

    /// Fetched items.
    pub items: Vec<FetchItem>,
}

impl FetchResult {
    /// Creates a new fetch result.
    #[must_use]
    pub fn new(seq: SeqNum) -> Self {
        Self {
            seq,
            uid: None,
            items: Vec::new(),
        }
    }

    /// Gets the FLAGS item if present.
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

    /// Gets the INTERNALDATE item if present.
    #[must_use]
    pub fn internal_date(&self) -> Option<&str> {
        self.items.iter().find_map(|item| {
            if let FetchItem::InternalDate(date) = item {
                Some(date.as_str())
            } else {
                None
            }
        })
    }

    /// Gets the RFC822.SIZE item if present.
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

/// Accumulator for collecting fetch results.
#[derive(Debug, Default)]
pub struct FetchAccumulator {
    results: VecDeque<FetchResult>,
    total_fetched: usize,
    current_batch: usize,
}

impl FetchAccumulator {
    /// Creates a new accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a fetch result.
    pub fn push(&mut self, result: FetchResult) {
        self.total_fetched += 1;
        self.results.push_back(result);
    }

    /// Takes all accumulated results.
    pub fn take(&mut self) -> Vec<FetchResult> {
        self.results.drain(..).collect()
    }

    /// Returns the number of results.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Returns `true` if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    /// Returns total items fetched across all batches.
    #[must_use]
    pub fn total_fetched(&self) -> usize {
        self.total_fetched
    }

    /// Advances to the next batch.
    pub fn next_batch(&mut self) {
        self.current_batch += 1;
    }

    /// Returns the current batch index.
    #[must_use]
    pub fn current_batch(&self) -> usize {
        self.current_batch
    }
}

/// Strategies for ordering fetch batches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchOrder {
    /// Fetch newest messages first (descending UID/sequence).
    #[default]
    NewestFirst,

    /// Fetch oldest messages first (ascending UID/sequence).
    OldestFirst,
}

impl BatchOrder {
    /// Applies the ordering to a list of batches.
    pub fn apply<T>(&self, batches: &mut [T]) {
        if matches!(self, Self::NewestFirst) {
            batches.reverse();
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config_defaults() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size.get(), DEFAULT_BATCH_SIZE);
        assert!(config.use_uids);
        assert_eq!(config.pipeline_depth, 1);
    }

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new()
            .batch_size(50)
            .use_uids(false)
            .pipeline_depth(3);

        assert_eq!(config.batch_size.get(), 50);
        assert!(!config.use_uids);
        assert_eq!(config.pipeline_depth, 3);
    }

    #[test]
    fn test_batch_size_clamping() {
        let config = BatchConfig::new().batch_size(1000);
        assert_eq!(config.batch_size.get(), MAX_BATCH_SIZE);

        let config = BatchConfig::new().batch_size(0);
        assert_eq!(config.batch_size.get(), DEFAULT_BATCH_SIZE);
    }

    #[test]
    fn test_create_range_batches() {
        let fetcher = BatchedFetch::new().batch_size(10);

        let batches = fetcher.create_range_batches(25);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0], (1, 10));
        assert_eq!(batches[1], (11, 20));
        assert_eq!(batches[2], (21, 25));
    }

    #[test]
    fn test_create_range_batches_exact() {
        let fetcher = BatchedFetch::new().batch_size(10);

        let batches = fetcher.create_range_batches(20);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0], (1, 10));
        assert_eq!(batches[1], (11, 20));
    }

    #[test]
    fn test_create_range_batches_empty() {
        let fetcher = BatchedFetch::new();
        let batches = fetcher.create_range_batches(0);
        assert!(batches.is_empty());
    }

    #[test]
    fn test_fetch_result() {
        let seq = SeqNum::new(1).unwrap();
        let mut result = FetchResult::new(seq);
        result.uid = Uid::new(100);

        assert_eq!(result.seq, seq);
        assert!(result.uid.is_some());
        assert!(result.items.is_empty());
    }

    #[test]
    fn test_fetch_accumulator() {
        let mut acc = FetchAccumulator::new();
        assert!(acc.is_empty());

        acc.push(FetchResult::new(SeqNum::new(1).unwrap()));
        acc.push(FetchResult::new(SeqNum::new(2).unwrap()));

        assert_eq!(acc.len(), 2);
        assert_eq!(acc.total_fetched(), 2);

        let results = acc.take();
        assert_eq!(results.len(), 2);
        assert!(acc.is_empty());
        assert_eq!(acc.total_fetched(), 2); // Total remains
    }

    #[test]
    fn test_batch_order() {
        let mut batches = vec![1, 2, 3, 4, 5];

        BatchOrder::OldestFirst.apply(&mut batches);
        assert_eq!(batches, vec![1, 2, 3, 4, 5]);

        BatchOrder::NewestFirst.apply(&mut batches);
        assert_eq!(batches, vec![5, 4, 3, 2, 1]);
    }
}
