//! Message cache for offline support.
//!
//! This module provides caching of messages for offline viewing when
//! the IMAP server is unavailable.

mod model;
mod repository;

pub use model::{CachedMessageContent, CachedMessageSummary};
pub use repository::CacheRepository;
