//! Time abstraction for testability.
//!
//! This module provides a `Clock` trait that abstracts over time operations,
//! enabling deterministic testing of time-dependent behavior.
//!
//! # Example
//!
//! ```
//! use mailledger_imap::time::{Clock, SystemClock};
//! use std::time::{Duration, Instant};
//!
//! let clock = SystemClock;
//! let now = clock.now();
//! // ... do some work ...
//! let elapsed = clock.now().duration_since(now);
//! ```

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Abstraction over time for testability.
///
/// In production, use [`SystemClock`] which delegates to `std::time::Instant`.
/// In tests, use [`MockClock`] to control time deterministically.
pub trait Clock: Send + Sync {
    /// Returns the current instant.
    fn now(&self) -> Instant;

    /// Returns the elapsed time since the given instant.
    fn elapsed(&self, since: Instant) -> Duration {
        self.now().duration_since(since)
    }

    /// Checks if a duration has elapsed since the given instant.
    fn has_elapsed(&self, since: Instant, duration: Duration) -> bool {
        self.elapsed(since) >= duration
    }
}

/// System clock that uses real time.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A mock clock for testing time-dependent code.
///
/// The clock starts at a base instant and can be advanced manually.
/// This is useful for testing timeouts, IDLE behavior, etc.
///
/// # Example
///
/// ```
/// use mailledger_imap::time::{Clock, MockClock};
/// use std::time::Duration;
///
/// let clock = MockClock::new();
/// let start = clock.now();
///
/// // Advance time by 5 seconds
/// clock.advance(Duration::from_secs(5));
///
/// assert_eq!(clock.elapsed(start), Duration::from_secs(5));
/// ```
#[derive(Debug)]
pub struct MockClock {
    /// Base instant (when the clock was created).
    base: Instant,
    /// Offset from base in nanoseconds.
    offset_nanos: AtomicU64,
}

impl Default for MockClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MockClock {
    /// Creates a new mock clock starting at the current time.
    #[must_use]
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset_nanos: AtomicU64::new(0),
        }
    }

    /// Creates a mock clock that can be shared across threads.
    #[must_use]
    pub fn shared() -> Arc<Self> {
        Arc::new(Self::new())
    }

    /// Advances the clock by the given duration.
    ///
    /// # Note
    /// Durations longer than ~584 years will be truncated. This is acceptable
    /// for a mock clock used in testing.
    #[allow(clippy::cast_possible_truncation)]
    pub fn advance(&self, duration: Duration) {
        let nanos = duration.as_nanos() as u64;
        self.offset_nanos.fetch_add(nanos, Ordering::SeqCst);
    }

    /// Sets the clock to a specific offset from the base.
    ///
    /// # Note
    /// Durations longer than ~584 years will be truncated. This is acceptable
    /// for a mock clock used in testing.
    #[allow(clippy::cast_possible_truncation)]
    pub fn set_offset(&self, offset: Duration) {
        let nanos = offset.as_nanos() as u64;
        self.offset_nanos.store(nanos, Ordering::SeqCst);
    }

    /// Resets the clock to the base time.
    pub fn reset(&self) {
        self.offset_nanos.store(0, Ordering::SeqCst);
    }

    /// Returns the current offset from the base time.
    #[must_use]
    pub fn offset(&self) -> Duration {
        Duration::from_nanos(self.offset_nanos.load(Ordering::SeqCst))
    }
}

impl Clock for MockClock {
    fn now(&self) -> Instant {
        self.base + self.offset()
    }
}

impl Clock for Arc<MockClock> {
    fn now(&self) -> Instant {
        self.as_ref().now()
    }
}

/// A boxed clock for dynamic dispatch.
pub type BoxClock = Box<dyn Clock>;

impl Clock for BoxClock {
    fn now(&self) -> Instant {
        self.as_ref().now()
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
    fn test_system_clock() {
        let clock = SystemClock;
        let before = Instant::now();
        let from_clock = clock.now();
        let after = Instant::now();

        // Clock should return a time between before and after
        assert!(from_clock >= before);
        assert!(from_clock <= after);
    }

    #[test]
    fn test_mock_clock_advance() {
        let clock = MockClock::new();
        let start = clock.now();

        clock.advance(Duration::from_secs(10));
        assert_eq!(clock.elapsed(start), Duration::from_secs(10));

        clock.advance(Duration::from_secs(5));
        assert_eq!(clock.elapsed(start), Duration::from_secs(15));
    }

    #[test]
    fn test_mock_clock_set_offset() {
        let clock = MockClock::new();
        let start = clock.now();

        clock.set_offset(Duration::from_secs(100));
        assert_eq!(clock.elapsed(start), Duration::from_secs(100));

        clock.set_offset(Duration::from_secs(50));
        assert_eq!(clock.elapsed(start), Duration::from_secs(50));
    }

    #[test]
    fn test_mock_clock_reset() {
        let clock = MockClock::new();
        let start = clock.now();

        clock.advance(Duration::from_secs(30));
        assert_eq!(clock.elapsed(start), Duration::from_secs(30));

        clock.reset();
        // After reset, we're back at offset 0, but start was captured when offset was 0
        // So elapsed should be 0
        assert_eq!(clock.offset(), Duration::ZERO);
    }

    #[test]
    fn test_mock_clock_has_elapsed() {
        let clock = MockClock::new();
        let start = clock.now();

        assert!(!clock.has_elapsed(start, Duration::from_secs(5)));

        clock.advance(Duration::from_secs(5));
        assert!(clock.has_elapsed(start, Duration::from_secs(5)));
        assert!(!clock.has_elapsed(start, Duration::from_secs(6)));

        clock.advance(Duration::from_secs(1));
        assert!(clock.has_elapsed(start, Duration::from_secs(6)));
    }

    #[test]
    fn test_shared_mock_clock() {
        let clock = MockClock::shared();
        let clock2 = Arc::clone(&clock);

        let start = clock.now();
        clock2.advance(Duration::from_secs(10));

        assert_eq!(clock.elapsed(start), Duration::from_secs(10));
        assert_eq!(clock2.elapsed(start), Duration::from_secs(10));
    }
}
