//! Triage system data models.

use crate::AccountId;

/// The user's decision about a sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SenderDecision {
    /// Sender hasn't been screened yet - waiting for user decision.
    #[default]
    Pending,
    /// Sender is approved - their emails will appear in the designated category.
    Approved,
    /// Sender is blocked - their emails will be hidden.
    Blocked,
}

impl SenderDecision {
    /// Parse from database string representation.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "approved" => Self::Approved,
            "blocked" => Self::Blocked,
            _ => Self::Pending,
        }
    }

    /// Convert to database string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Blocked => "blocked",
        }
    }
}

/// Category for organizing approved emails.
///
/// Inspired by HEY's Imbox/Feed/Paper Trail concept:
/// - **Imbox**: Important emails from real people you care about
/// - **Feed**: Newsletters and subscriptions you read when you have time
/// - **Paper Trail**: Receipts, confirmations, and transactional emails
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InboxCategory {
    /// Important emails that deserve immediate attention.
    /// These appear front and center in your inbox.
    #[default]
    Imbox,
    /// Newsletters and content you read at your leisure.
    /// Like a personal news feed - scroll through when you have time.
    Feed,
    /// Receipts, confirmations, shipping notifications, etc.
    /// Important to keep but not to see until you need them.
    PaperTrail,
}

impl InboxCategory {
    /// Parse from database string representation.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "feed" => Self::Feed,
            "paper_trail" | "papertrial" => Self::PaperTrail,
            _ => Self::Imbox,
        }
    }

    /// Convert to database string representation.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Imbox => "imbox",
            Self::Feed => "feed",
            Self::PaperTrail => "paper_trail",
        }
    }

    /// Human-readable display name.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Imbox => "Imbox",
            Self::Feed => "The Feed",
            Self::PaperTrail => "Paper Trail",
        }
    }

    /// Description of what belongs in this category.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Imbox => "Important emails from people you care about",
            Self::Feed => "Newsletters and subscriptions to read later",
            Self::PaperTrail => "Receipts, confirmations, and transactional emails",
        }
    }

    /// Icon name for this category.
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Imbox => "inbox",
            Self::Feed => "newspaper",
            Self::PaperTrail => "receipt",
        }
    }
}

/// A sender that has been screened by the user.
#[derive(Debug, Clone)]
pub struct ScreenedSender {
    /// Unique identifier.
    pub id: Option<i64>,
    /// Account this sender belongs to.
    pub account_id: AccountId,
    /// Email address of the sender (normalized to lowercase).
    pub email: String,
    /// Display name of the sender (if known).
    pub display_name: Option<String>,
    /// User's decision about this sender.
    pub decision: SenderDecision,
    /// Category for approved senders.
    pub category: InboxCategory,
    /// Optional note about this sender.
    pub note: Option<String>,
    /// Number of emails received from this sender.
    pub email_count: u32,
    /// Timestamp of first email from this sender.
    pub first_seen: Option<String>,
    /// Timestamp of most recent email from this sender.
    pub last_seen: Option<String>,
}

impl ScreenedSender {
    /// Create a new pending sender entry.
    #[must_use]
    pub fn new_pending(account_id: AccountId, email: &str, display_name: Option<&str>) -> Self {
        Self {
            id: None,
            account_id,
            email: email.to_lowercase(),
            display_name: display_name.map(ToString::to_string),
            decision: SenderDecision::Pending,
            category: InboxCategory::Imbox,
            note: None,
            email_count: 1,
            first_seen: None,
            last_seen: None,
        }
    }

    /// Check if this sender is approved.
    #[must_use]
    pub const fn is_approved(&self) -> bool {
        matches!(self.decision, SenderDecision::Approved)
    }

    /// Check if this sender is blocked.
    #[must_use]
    pub const fn is_blocked(&self) -> bool {
        matches!(self.decision, SenderDecision::Blocked)
    }

    /// Check if this sender is pending screening.
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        matches!(self.decision, SenderDecision::Pending)
    }

    /// Approve this sender and assign to a category.
    pub const fn approve(&mut self, category: InboxCategory) {
        self.decision = SenderDecision::Approved;
        self.category = category;
    }

    /// Block this sender.
    pub const fn block(&mut self) {
        self.decision = SenderDecision::Blocked;
    }
}

impl std::str::FromStr for SenderDecision {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl std::str::FromStr for InboxCategory {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sender_decision_roundtrip() {
        for decision in [
            SenderDecision::Pending,
            SenderDecision::Approved,
            SenderDecision::Blocked,
        ] {
            assert_eq!(SenderDecision::parse(decision.as_str()), decision);
        }
    }

    #[test]
    fn test_inbox_category_roundtrip() {
        for category in [
            InboxCategory::Imbox,
            InboxCategory::Feed,
            InboxCategory::PaperTrail,
        ] {
            assert_eq!(InboxCategory::parse(category.as_str()), category);
        }
    }

    #[test]
    fn test_screened_sender_new_pending() {
        let sender =
            ScreenedSender::new_pending(AccountId::new(1), "Test@Example.com", Some("Test User"));

        assert!(sender.is_pending());
        assert!(!sender.is_approved());
        assert!(!sender.is_blocked());
        assert_eq!(sender.email, "test@example.com"); // Normalized to lowercase
        assert_eq!(sender.display_name, Some("Test User".to_string()));
    }

    #[test]
    fn test_screened_sender_approve() {
        let mut sender = ScreenedSender::new_pending(AccountId::new(1), "news@example.com", None);

        sender.approve(InboxCategory::Feed);

        assert!(sender.is_approved());
        assert_eq!(sender.category, InboxCategory::Feed);
    }

    #[test]
    fn test_screened_sender_block() {
        let mut sender = ScreenedSender::new_pending(AccountId::new(1), "spam@example.com", None);

        sender.block();

        assert!(sender.is_blocked());
    }
}
