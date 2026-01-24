//! Message threading model.

use super::{MessageId, MessageSummary};
use std::collections::HashMap;

/// View mode for the message list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Show messages in a flat list (default).
    #[default]
    Flat,
    /// Show messages grouped by thread.
    Threaded,
}

/// A thread of related messages.
#[derive(Debug, Clone)]
#[allow(dead_code)] // is_expanded tracked externally in main.rs
pub struct Thread {
    /// Thread identifier (usually the root Message-ID).
    pub id: String,
    /// Messages in this thread, ordered by date.
    pub messages: Vec<MessageId>,
    /// Subject of the thread (from the first message).
    pub subject: String,
    /// Participants (unique senders).
    pub participants: Vec<String>,
    /// Date of the most recent message.
    pub latest_date: String,
    /// Number of unread messages in the thread.
    pub unread_count: usize,
    /// Whether the thread is expanded in the UI.
    pub is_expanded: bool,
}

impl Thread {
    /// Creates a new thread from a root message.
    #[must_use]
    pub fn new(root_message: &MessageSummary) -> Self {
        let thread_id = root_message
            .thread_id
            .clone()
            .or_else(|| root_message.message_id.clone())
            .unwrap_or_else(|| format!("thread-{}", root_message.id.0));

        Self {
            id: thread_id,
            messages: vec![root_message.id],
            subject: root_message.subject.clone(),
            participants: vec![root_message.from_name.clone()],
            latest_date: root_message.date.clone(),
            unread_count: usize::from(!root_message.is_read),
            is_expanded: false,
        }
    }

    /// Adds a message to the thread.
    pub fn add_message(&mut self, message: &MessageSummary) {
        if !self.messages.contains(&message.id) {
            self.messages.push(message.id);

            // Update participants
            if !self.participants.contains(&message.from_name) {
                self.participants.push(message.from_name.clone());
            }

            // Update unread count
            if !message.is_read {
                self.unread_count += 1;
            }

            // Update latest date (assuming messages are processed chronologically)
            self.latest_date.clone_from(&message.date);
        }
    }

    /// Returns the number of messages in the thread.
    #[must_use]
    pub const fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Returns a display string for participants (e.g., "Alice, Bob, +2 others").
    #[must_use]
    pub fn participants_display(&self) -> String {
        match self.participants.len() {
            0 => String::new(),
            1 => self.participants[0].clone(),
            2 => format!("{}, {}", self.participants[0], self.participants[1]),
            n => format!(
                "{}, {} +{} others",
                self.participants[0],
                self.participants[1],
                n - 2
            ),
        }
    }
}

/// Groups messages into threads.
#[must_use]
pub fn group_into_threads(messages: &[MessageSummary]) -> Vec<Thread> {
    let mut threads: HashMap<String, Thread> = HashMap::new();

    for message in messages {
        // Determine thread ID
        let thread_id = message
            .thread_id
            .clone()
            .or_else(|| message.in_reply_to.clone())
            .or_else(|| message.message_id.clone())
            .unwrap_or_else(|| format!("thread-{}", message.id.0));

        // Check if this message is a reply to an existing thread
        let existing_thread_id = message
            .in_reply_to
            .as_ref()
            .and_then(|reply_to| {
                // Find thread containing the message we're replying to
                threads
                    .iter()
                    .find(|(_, t)| {
                        messages.iter().any(|m| {
                            m.message_id.as_ref() == Some(reply_to) && t.messages.contains(&m.id)
                        })
                    })
                    .map(|(id, _)| id.clone())
            })
            .unwrap_or_else(|| thread_id.clone());

        if let Some(thread) = threads.get_mut(&existing_thread_id) {
            thread.add_message(message);
        } else {
            let thread = Thread::new(message);
            threads.insert(thread.id.clone(), thread);
        }
    }

    // Convert to vector and sort by latest date (most recent first)
    let mut thread_list: Vec<Thread> = threads.into_values().collect();
    thread_list.sort_by(|a, b| b.latest_date.cmp(&a.latest_date));
    thread_list
}
