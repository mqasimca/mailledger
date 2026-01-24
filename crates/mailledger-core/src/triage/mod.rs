//! Email Triage System - HEY-inspired sender screening and inbox organization.
//!
//! This module provides:
//! - **Screener**: New senders are held for approval before reaching inbox
//! - **Sender Status**: Track whether a sender is approved, blocked, or pending
//! - **Inbox Categories**: Route emails to Imbox (important), Feed (newsletters), or Paper Trail (receipts)
//!
//! # Design Philosophy
//!
//! Instead of dealing with email pile-ups, `MailLedger` puts you in control:
//! 1. First-time senders go to The Screener for your approval
//! 2. Approved senders can be routed to different categories based on type
//! 3. Blocked senders never bother you again
//!
//! # Example
//!
//! ```ignore
//! use mailledger_core::triage::{TriageRepository, SenderDecision, InboxCategory};
//!
//! // Check if sender is known
//! let status = repo.get_sender_status(account_id, "newsletter@company.com").await?;
//!
//! match status {
//!     None => {
//!         // New sender - show in Screener for user decision
//!     }
//!     Some(sender) if sender.decision == SenderDecision::Approved => {
//!         // Show in their designated category
//!     }
//!     Some(sender) if sender.decision == SenderDecision::Blocked => {
//!         // Hide this email
//!     }
//!     _ => {}
//! }
//! ```

mod model;
mod repository;

pub use model::{InboxCategory, ScreenedSender, SenderDecision};
pub use repository::TriageRepository;
