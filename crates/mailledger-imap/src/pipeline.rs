//! Command pipelining for improved IMAP performance.
//!
// Allow missing_const_for_fn since many can't be const due to Vec/VecDeque operations.
#![allow(clippy::missing_const_for_fn)]
//!
//! IMAP allows sending multiple commands before receiving responses
//! (RFC 9051 Section 5.5). This can significantly reduce round-trip
//! latency, especially over high-latency connections.
//!
//! ## When to Pipeline
//!
//! Pipelining is safe when commands:
//! - Don't depend on each other's results
//! - Won't cause ambiguous parsing
//! - Are unlikely to fail
//!
//! ## When NOT to Pipeline
//!
//! Don't pipeline when:
//! - Commands depend on previous results (e.g., LOGIN then SELECT)
//! - The server doesn't advertise LITERAL+ (for commands with literals)
//! - Commands might fail and affect subsequent commands
//!
//! # Example
//!
//! ```ignore
//! use mailledger_imap::pipeline::{Pipeline, PipelineConfig};
//!
//! let config = PipelineConfig::new()
//!     .max_depth(4)
//!     .timeout(Duration::from_secs(30));
//!
//! let mut pipeline = Pipeline::new(config);
//!
//! // Queue multiple NOOP commands
//! pipeline.queue(Command::Noop);
//! pipeline.queue(Command::Noop);
//! pipeline.queue(Command::Noop);
//!
//! // Send all at once
//! let transmits = pipeline.flush();
//! ```

use std::collections::VecDeque;
use std::time::Duration;

use crate::command::Command;
use crate::types::Tag;

/// Default maximum pipeline depth.
pub const DEFAULT_MAX_DEPTH: usize = 4;

/// Maximum allowed pipeline depth.
pub const MAX_PIPELINE_DEPTH: usize = 16;

/// Default pipeline timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Configuration for command pipelining.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Maximum number of commands to pipeline.
    pub max_depth: usize,

    /// Timeout for the entire pipeline.
    pub timeout: Duration,

    /// Whether to enable pipelining at all.
    pub enabled: bool,

    /// Whether to pipeline commands with literals (requires LITERAL+).
    pub allow_literals: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            timeout: DEFAULT_TIMEOUT,
            enabled: true,
            allow_literals: false,
        }
    }
}

impl PipelineConfig {
    /// Creates a new pipeline configuration with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum pipeline depth.
    #[must_use]
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth.min(MAX_PIPELINE_DEPTH);
        self
    }

    /// Sets the pipeline timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enables or disables pipelining.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Allows pipelining commands with literals (requires LITERAL+).
    #[must_use]
    pub fn allow_literals(mut self, allow: bool) -> Self {
        self.allow_literals = allow;
        self
    }
}

/// A command queued for pipelining.
#[derive(Debug, Clone)]
pub struct QueuedCommand {
    /// The command to send.
    pub command: Command,
    /// Tag assigned to this command.
    pub tag: Tag,
    /// Whether this command can be pipelined.
    pub pipelineable: bool,
}

impl QueuedCommand {
    /// Creates a new queued command.
    #[must_use]
    pub fn new(command: Command, tag: Tag) -> Self {
        let pipelineable = command.is_pipelineable();
        Self {
            command,
            tag,
            pipelineable,
        }
    }
}

/// Command pipeline manager.
///
/// Manages queuing and flushing of pipelined IMAP commands.
#[derive(Debug)]
pub struct Pipeline {
    config: PipelineConfig,
    queue: VecDeque<QueuedCommand>,
    in_flight: VecDeque<Tag>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::default())
    }
}

impl Pipeline {
    /// Creates a new pipeline with the given configuration.
    #[must_use]
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            queue: VecDeque::new(),
            in_flight: VecDeque::new(),
        }
    }

    /// Returns the pipeline configuration.
    #[must_use]
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    /// Returns the number of commands currently queued.
    #[must_use]
    pub fn queued_count(&self) -> usize {
        self.queue.len()
    }

    /// Returns the number of commands currently in flight.
    #[must_use]
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Returns `true` if the pipeline has room for more commands.
    #[must_use]
    pub fn has_capacity(&self) -> bool {
        self.in_flight.len() + self.queue.len() < self.config.max_depth
    }

    /// Returns `true` if there are queued commands ready to send.
    #[must_use]
    pub fn has_pending(&self) -> bool {
        !self.queue.is_empty()
    }

    /// Returns `true` if pipelining is enabled and beneficial.
    #[must_use]
    pub fn should_pipeline(&self) -> bool {
        self.config.enabled && self.queue.len() >= 2
    }

    /// Queues a command for pipelining.
    ///
    /// Returns `true` if the command was queued, `false` if the pipeline is full.
    pub fn queue(&mut self, command: QueuedCommand) -> bool {
        if !self.has_capacity() {
            return false;
        }

        // Check if command can be pipelined
        if !self.config.enabled && !self.queue.is_empty() {
            return false;
        }

        // Check for literals if not allowed
        if !self.config.allow_literals && command.command.has_literal() {
            // Can only queue if this is the only command
            if !self.queue.is_empty() || !self.in_flight.is_empty() {
                return false;
            }
        }

        self.queue.push_back(command);
        true
    }

    /// Takes all queued commands, ready to be sent together.
    ///
    /// The commands' tags are moved to the in-flight list.
    pub fn flush(&mut self) -> Vec<QueuedCommand> {
        let commands: Vec<_> = self.queue.drain(..).collect();
        for cmd in &commands {
            self.in_flight.push_back(cmd.tag.clone());
        }
        commands
    }

    /// Marks a command as completed.
    ///
    /// Returns `true` if the tag was in-flight, `false` otherwise.
    pub fn complete(&mut self, tag: &Tag) -> bool {
        if let Some(pos) = self.in_flight.iter().position(|t| t == tag) {
            // Warn if responses arrive out of order
            if pos != 0 {
                tracing::warn!(
                    "received response for tag {:?} out of order (expected {:?})",
                    tag,
                    self.in_flight.front()
                );
            }
            self.in_flight.remove(pos);
            true
        } else {
            false
        }
    }

    /// Clears all queued and in-flight commands.
    pub fn clear(&mut self) {
        self.queue.clear();
        self.in_flight.clear();
    }

    /// Returns the next expected response tag, if any.
    #[must_use]
    pub fn next_expected(&self) -> Option<&Tag> {
        self.in_flight.front()
    }

    /// Checks if a tag is currently in flight.
    #[must_use]
    pub fn is_in_flight(&self, tag: &Tag) -> bool {
        self.in_flight.contains(tag)
    }
}

/// Classification of commands for pipelining safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineSafety {
    /// Command is safe to pipeline (no state changes, no dependencies).
    Safe,

    /// Command can be pipelined but may affect subsequent commands.
    Caution,

    /// Command should not be pipelined (state-changing or dependent).
    Unsafe,
}

impl Command {
    /// Returns whether this command can be safely pipelined.
    #[must_use]
    pub fn pipeline_safety(&self) -> PipelineSafety {
        match self {
            // Safe to pipeline - no state changes
            Self::Noop
            | Self::Capability
            | Self::List { .. }
            | Self::Namespace
            | Self::Status { .. } => PipelineSafety::Safe,

            // Can pipeline with caution
            Self::Fetch { .. } | Self::Search { .. } | Self::Copy { .. } | Self::Move { .. } => {
                PipelineSafety::Caution
            }

            // Should not pipeline - state changes
            Self::Login { .. }
            | Self::Logout
            | Self::Select { .. }
            | Self::Examine { .. }
            | Self::Close
            | Self::Unselect
            | Self::Expunge
            | Self::UidExpunge { .. }
            | Self::Store { .. }
            | Self::Append { .. }
            | Self::Idle
            | Self::Done
            | Self::Create { .. }
            | Self::Delete { .. }
            | Self::Rename { .. }
            | Self::Subscribe { .. }
            | Self::Unsubscribe { .. }
            | Self::StartTls
            | Self::Authenticate { .. }
            | Self::Id { .. }
            | Self::Enable { .. } => PipelineSafety::Unsafe,
        }
    }

    /// Returns `true` if this command can be pipelined.
    #[must_use]
    pub fn is_pipelineable(&self) -> bool {
        !matches!(self.pipeline_safety(), PipelineSafety::Unsafe)
    }

    /// Returns `true` if this command contains a literal.
    #[must_use]
    pub fn has_literal(&self) -> bool {
        matches!(self, Self::Append { .. })
    }
}

/// Batches commands into pipeline-safe groups.
///
/// Commands that are unsafe to pipeline will be in their own batch.
#[must_use]
pub fn batch_commands(commands: Vec<Command>) -> Vec<Vec<Command>> {
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();

    for cmd in commands {
        if cmd.is_pipelineable() {
            current_batch.push(cmd);
        } else {
            // Flush current batch if not empty
            if !current_batch.is_empty() {
                batches.push(std::mem::take(&mut current_batch));
            }
            // Add unsafe command as its own batch
            batches.push(vec![cmd]);
        }
    }

    // Flush remaining commands
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
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
    use crate::types::Mailbox;

    fn test_tag(s: &str) -> Tag {
        Tag::new(s)
    }

    #[test]
    fn test_pipeline_config_defaults() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_depth, DEFAULT_MAX_DEPTH);
        assert!(config.enabled);
        assert!(!config.allow_literals);
    }

    #[test]
    fn test_pipeline_config_builder() {
        let config = PipelineConfig::new()
            .max_depth(8)
            .timeout(Duration::from_secs(120))
            .enabled(false);

        assert_eq!(config.max_depth, 8);
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert!(!config.enabled);
    }

    #[test]
    fn test_pipeline_max_depth_clamping() {
        let config = PipelineConfig::new().max_depth(100);
        assert_eq!(config.max_depth, MAX_PIPELINE_DEPTH);
    }

    #[test]
    fn test_pipeline_queue() {
        let mut pipeline = Pipeline::default();

        let cmd = QueuedCommand::new(Command::Noop, test_tag("A001"));
        assert!(pipeline.queue(cmd));
        assert_eq!(pipeline.queued_count(), 1);
    }

    #[test]
    fn test_pipeline_flush() {
        let mut pipeline = Pipeline::default();

        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A001")));
        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A002")));

        let commands = pipeline.flush();
        assert_eq!(commands.len(), 2);
        assert_eq!(pipeline.queued_count(), 0);
        assert_eq!(pipeline.in_flight_count(), 2);
    }

    #[test]
    fn test_pipeline_complete() {
        let mut pipeline = Pipeline::default();

        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A001")));
        pipeline.flush();

        assert!(pipeline.complete(&test_tag("A001")));
        assert_eq!(pipeline.in_flight_count(), 0);

        // Completing again should return false
        assert!(!pipeline.complete(&test_tag("A001")));
    }

    #[test]
    fn test_pipeline_capacity() {
        let mut pipeline = Pipeline::new(PipelineConfig::new().max_depth(2));

        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A001")));
        assert!(pipeline.has_capacity());

        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A002")));
        assert!(!pipeline.has_capacity());
    }

    #[test]
    fn test_command_pipeline_safety() {
        assert_eq!(Command::Noop.pipeline_safety(), PipelineSafety::Safe);
        assert_eq!(Command::Capability.pipeline_safety(), PipelineSafety::Safe);
        assert_eq!(
            Command::Login {
                username: "".to_string(),
                password: "".to_string()
            }
            .pipeline_safety(),
            PipelineSafety::Unsafe
        );
        assert_eq!(
            Command::Select {
                mailbox: Mailbox::new("INBOX"),
                condstore: false
            }
            .pipeline_safety(),
            PipelineSafety::Unsafe
        );
    }

    #[test]
    fn test_batch_commands() {
        let commands = vec![
            Command::Noop,
            Command::Capability,
            Command::Login {
                username: "user".to_string(),
                password: "pass".to_string(),
            },
            Command::Noop,
            Command::Noop,
        ];

        let batches = batch_commands(commands);
        assert_eq!(batches.len(), 3);
        assert_eq!(batches[0].len(), 2); // Noop, Capability
        assert_eq!(batches[1].len(), 1); // Login
        assert_eq!(batches[2].len(), 2); // Noop, Noop
    }

    #[test]
    fn test_next_expected() {
        let mut pipeline = Pipeline::default();
        assert!(pipeline.next_expected().is_none());

        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A001")));
        pipeline.queue(QueuedCommand::new(Command::Noop, test_tag("A002")));
        pipeline.flush();

        assert_eq!(pipeline.next_expected(), Some(&test_tag("A001")));
        pipeline.complete(&test_tag("A001"));
        assert_eq!(pipeline.next_expected(), Some(&test_tag("A002")));
    }
}
