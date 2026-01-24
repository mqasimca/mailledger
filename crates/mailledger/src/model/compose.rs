//! Compose message model.

/// Which address field is currently being autocompleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AutocompleteField {
    /// To field.
    #[default]
    To,
    /// Cc field.
    Cc,
    /// Bcc field.
    Bcc,
}

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
    /// Message body (kept for backward compatibility, actual body is in `text_editor::Content`).
    #[allow(dead_code)]
    pub body: String,
    /// Whether we're currently sending.
    pub is_sending: bool,
    /// Error message from send attempt.
    pub send_error: Option<String>,
    /// Success message after sending.
    pub send_success: bool,
    /// Contact suggestions for autocomplete.
    pub suggestions: Vec<mailledger_core::Contact>,
    /// Which field is currently being autocompleted.
    pub active_autocomplete: Option<AutocompleteField>,
    /// Index of currently selected suggestion (for keyboard navigation).
    pub selected_suggestion: usize,
}

impl ComposeState {
    /// Creates a new empty compose state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets suggestions and shows autocomplete for a field.
    pub fn show_suggestions(
        &mut self,
        suggestions: Vec<mailledger_core::Contact>,
        field: AutocompleteField,
    ) {
        self.suggestions = suggestions;
        self.active_autocomplete = Some(field);
        self.selected_suggestion = 0;
    }

    /// Clears suggestions and hides autocomplete.
    pub fn clear_suggestions(&mut self) {
        self.suggestions.clear();
        self.active_autocomplete = None;
        self.selected_suggestion = 0;
    }

    /// Selects a suggestion and appends it to the appropriate field.
    pub fn apply_suggestion(&mut self, index: usize) {
        if let Some(contact) = self.suggestions.get(index) {
            let email_to_add = contact.display();

            if let Some(field) = self.active_autocomplete {
                let target = match field {
                    AutocompleteField::To => &mut self.to,
                    AutocompleteField::Cc => &mut self.cc,
                    AutocompleteField::Bcc => &mut self.bcc,
                };

                // Find and replace the current incomplete entry
                // Split by comma, replace last entry with the contact, rejoin
                let mut parts: Vec<&str> = target.split(',').collect();
                if let Some(last) = parts.last_mut() {
                    *last = "";
                }
                let mut new_value = parts.join(",").trim_end_matches(',').to_string();
                if !new_value.is_empty() {
                    new_value.push_str(", ");
                }
                new_value.push_str(&email_to_add);

                *target = new_value;
            }

            self.clear_suggestions();
        }
    }

    /// Moves selection up in suggestions list.
    #[allow(dead_code)] // Will be used for arrow key navigation
    pub fn select_previous(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_suggestion = self
                .selected_suggestion
                .checked_sub(1)
                .unwrap_or(self.suggestions.len() - 1);
        }
    }

    /// Moves selection down in suggestions list.
    #[allow(dead_code)] // Will be used for arrow key navigation
    pub const fn select_next(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_suggestion = (self.selected_suggestion + 1) % self.suggestions.len();
        }
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

    /// Converts to `OutgoingMessage` for sending (uses self.body).
    #[must_use]
    #[allow(dead_code)] // Kept for backward compatibility
    pub fn to_outgoing(&self, from: &str) -> mailledger_core::OutgoingMessage {
        self.to_outgoing_with_body(from, &self.body)
    }

    /// Converts to `OutgoingMessage` for sending with explicit body.
    #[must_use]
    pub fn to_outgoing_with_body(
        &self,
        from: &str,
        body: &str,
    ) -> mailledger_core::OutgoingMessage {
        let mut msg = mailledger_core::OutgoingMessage::new(from, &self.subject, body);

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
