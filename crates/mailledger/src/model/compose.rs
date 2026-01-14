//! Compose message model.

/// State for the compose message form.
#[derive(Debug, Clone, Default)]
pub struct ComposeState {
    /// Recipient addresses (To).
    pub to: String,
    /// CC addresses.
    pub cc: String,
    /// BCC addresses.
    pub bcc: String,
    /// Subject line.
    pub subject: String,
    /// Message body.
    pub body: String,
    /// Whether we're currently sending.
    pub is_sending: bool,
    /// Error message from send attempt.
    pub send_error: Option<String>,
    /// Success message after sending.
    pub send_success: bool,
}

impl ComposeState {
    /// Creates a new empty compose state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a compose state for a reply.
    #[must_use]
    pub fn reply(to: &str, subject: &str, quoted_body: &str) -> Self {
        let subject = if subject.to_lowercase().starts_with("re:") {
            subject.to_string()
        } else {
            format!("Re: {subject}")
        };

        let body = format!("\n\n> {}", quoted_body.replace('\n', "\n> "));

        Self {
            to: to.to_string(),
            subject,
            body,
            ..Default::default()
        }
    }

    /// Creates a compose state for forwarding.
    #[must_use]
    pub fn forward(subject: &str, original_body: &str, original_from: &str) -> Self {
        let subject = if subject.to_lowercase().starts_with("fwd:") {
            subject.to_string()
        } else {
            format!("Fwd: {subject}")
        };

        let body = format!(
            "\n\n---------- Forwarded message ----------\nFrom: {original_from}\n\n{original_body}"
        );

        Self {
            subject,
            body,
            ..Default::default()
        }
    }

    /// Validates the compose form.
    #[must_use]
    pub fn validate(&self) -> Option<String> {
        if self.to.trim().is_empty() {
            return Some("Please enter at least one recipient".to_string());
        }

        // Basic email validation for each recipient
        for recipient in self.to.split(',') {
            let recipient = recipient.trim();
            if !recipient.is_empty() && !recipient.contains('@') {
                return Some(format!("Invalid email address: {recipient}"));
            }
        }

        if self.subject.trim().is_empty() {
            return Some("Please enter a subject".to_string());
        }

        None
    }

    /// Converts to `OutgoingMessage` for sending.
    #[must_use]
    pub fn to_outgoing(&self, from: &str) -> mailledger_core::OutgoingMessage {
        let mut msg = mailledger_core::OutgoingMessage::new(from, &self.subject, &self.body);

        // Parse recipients
        for recipient in self.to.split(',') {
            let recipient = recipient.trim();
            if !recipient.is_empty() {
                msg.to.push(recipient.to_string());
            }
        }

        for recipient in self.cc.split(',') {
            let recipient = recipient.trim();
            if !recipient.is_empty() {
                msg.cc.push(recipient.to_string());
            }
        }

        for recipient in self.bcc.split(',') {
            let recipient = recipient.trim();
            if !recipient.is_empty() {
                msg.bcc.push(recipient.to_string());
            }
        }

        msg
    }
}
