//! IMAP command tag generator.
//!
//! Tags are used to match commands with their responses.

use std::sync::atomic::{AtomicU32, Ordering};

/// Tag generator for IMAP commands.
///
/// Generates unique sequential tags in the format "A001", "A002", etc.
#[derive(Debug)]
pub struct TagGenerator {
    counter: AtomicU32,
    prefix: char,
}

impl TagGenerator {
    /// Creates a new tag generator with the given prefix.
    #[must_use]
    pub const fn new(prefix: char) -> Self {
        Self {
            counter: AtomicU32::new(0),
            prefix,
        }
    }

    /// Generates the next tag.
    ///
    /// # Panics
    ///
    /// Panics if the tag counter would overflow u32::MAX. In practice, this
    /// would require 4+ billion tags in a single session, which is unrealistic.
    #[must_use]
    pub fn next(&self) -> String {
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        // Check for overflow - if we've wrapped, the session is invalid
        if n == u32::MAX {
            panic!("tag counter overflow: generated {} tags in this session", n);
        }
        format!("{}{:04}", self.prefix, n)
    }

    /// Returns the current counter value without incrementing.
    #[must_use]
    pub fn current(&self) -> u32 {
        self.counter.load(Ordering::Relaxed)
    }

    /// Resets the counter to zero.
    pub fn reset(&self) {
        self.counter.store(0, Ordering::Relaxed);
    }
}

impl Default for TagGenerator {
    fn default() -> Self {
        Self::new('A')
    }
}

impl Clone for TagGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: AtomicU32::new(self.counter.load(Ordering::Relaxed)),
            prefix: self.prefix,
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_string_new,
    clippy::needless_collect,
    clippy::unreadable_literal,
    clippy::used_underscore_items,
    clippy::similar_names
)]
mod tests {
    use super::*;

    #[test]
    fn test_tag_generation() {
        let generator = TagGenerator::default();
        assert_eq!(generator.next(), "A0000");
        assert_eq!(generator.next(), "A0001");
        assert_eq!(generator.next(), "A0002");
    }

    #[test]
    fn test_custom_prefix() {
        let generator = TagGenerator::new('T');
        assert_eq!(generator.next(), "T0000");
        assert_eq!(generator.next(), "T0001");
    }

    #[test]
    fn test_reset() {
        let generator = TagGenerator::default();
        let _ = generator.next();
        let _ = generator.next();
        generator.reset();
        assert_eq!(generator.next(), "A0000");
    }

    #[test]
    fn test_current() {
        let generator = TagGenerator::default();
        assert_eq!(generator.current(), 0);
        let _ = generator.next();
        assert_eq!(generator.current(), 1);
    }

    #[test]
    fn test_uniqueness() {
        let generator = TagGenerator::default();
        let mut seen = std::collections::HashSet::new();

        // Generate 10000 tags and ensure all are unique
        for _ in 0..10000 {
            let tag = generator.next();
            assert!(seen.insert(tag), "duplicate tag generated");
        }
    }

    #[test]
    fn test_format() {
        let generator = TagGenerator::new('X');
        assert_eq!(generator.next(), "X0000");
        assert_eq!(generator.next(), "X0001");
        // Verify padding is correct
        for _ in 2..100 {
            let _ = generator.next();
        }
        assert_eq!(generator.next(), "X0100");
    }

    #[test]
    #[should_panic(expected = "tag counter overflow")]
    fn test_overflow_detection() {
        let generator = TagGenerator::default();
        // Set counter to near max
        generator.counter.store(u32::MAX, Ordering::Relaxed);
        // This should panic
        let _ = generator.next();
    }
}
