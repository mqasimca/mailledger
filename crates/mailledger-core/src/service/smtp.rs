//! SMTP service for sending emails.
//!
//! Provides high-level email sending operations using the SMTP library.

use crate::Security;
use crate::account::Account;

/// Errors that can occur during SMTP operations.
#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    /// Connection failed.
    #[error("Connection failed: {0}")]
    Connection(String),

    /// Authentication failed.
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Send failed.
    #[error("Send failed: {0}")]
    Send(String),

    /// Invalid address.
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Security mode not supported.
    #[error("Security mode not supported: {0}")]
    UnsupportedSecurity(String),
}

/// An email message to send.
#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    /// Sender address.
    pub from: String,
    /// Recipient addresses.
    pub to: Vec<String>,
    /// CC addresses.
    pub cc: Vec<String>,
    /// BCC addresses.
    pub bcc: Vec<String>,
    /// Subject line.
    pub subject: String,
    /// Plain text body.
    pub body: String,
}

impl OutgoingMessage {
    /// Creates a new outgoing message.
    #[must_use]
    pub fn new(
        from: impl Into<String>,
        subject: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            subject: subject.into(),
            body: body.into(),
        }
    }

    /// Adds a recipient.
    #[must_use]
    pub fn to(mut self, recipient: impl Into<String>) -> Self {
        self.to.push(recipient.into());
        self
    }

    /// Adds a CC recipient.
    #[must_use]
    pub fn cc(mut self, recipient: impl Into<String>) -> Self {
        self.cc.push(recipient.into());
        self
    }

    /// Adds a BCC recipient.
    #[must_use]
    pub fn bcc(mut self, recipient: impl Into<String>) -> Self {
        self.bcc.push(recipient.into());
        self
    }

    /// Builds the RFC 5322 formatted message.
    fn to_rfc5322(&self) -> String {
        use std::fmt::Write;

        let mut message = String::new();

        // Headers
        let _ = writeln!(message, "From: {}\r", self.from);

        if !self.to.is_empty() {
            let _ = writeln!(message, "To: {}\r", self.to.join(", "));
        }

        if !self.cc.is_empty() {
            let _ = writeln!(message, "Cc: {}\r", self.cc.join(", "));
        }

        let _ = writeln!(message, "Subject: {}\r", self.subject);
        message.push_str("MIME-Version: 1.0\r\n");
        message.push_str("Content-Type: text/plain; charset=utf-8\r\n");
        message.push_str("Content-Transfer-Encoding: 8bit\r\n");

        // Empty line between headers and body
        message.push_str("\r\n");

        // Body
        message.push_str(&self.body);

        message
    }

    /// Returns all recipients (to, cc, bcc).
    fn all_recipients(&self) -> Vec<&str> {
        let mut recipients: Vec<&str> = Vec::new();
        for addr in &self.to {
            recipients.push(addr);
        }
        for addr in &self.cc {
            recipients.push(addr);
        }
        for addr in &self.bcc {
            recipients.push(addr);
        }
        recipients
    }
}

/// Send an email using the account's SMTP settings.
///
/// # Errors
///
/// Returns an error if connection, authentication, or sending fails.
pub async fn send_email(account: &Account, message: OutgoingMessage) -> Result<(), SmtpError> {
    use mailledger_smtp::connection::{connect, connect_tls};
    use mailledger_smtp::{Address, Client};

    // Validate recipients
    if message.to.is_empty() {
        return Err(SmtpError::InvalidAddress("No recipients specified".into()));
    }

    let all_recipients = message.all_recipients();
    if all_recipients.is_empty() {
        return Err(SmtpError::InvalidAddress("No recipients specified".into()));
    }

    // Connect based on security mode
    let stream = match account.smtp.security {
        Security::Tls => connect_tls(&account.smtp.host, account.smtp.port)
            .await
            .map_err(|e| SmtpError::Connection(e.to_string()))?,
        Security::StartTls | Security::None => connect(&account.smtp.host, account.smtp.port)
            .await
            .map_err(|e| SmtpError::Connection(e.to_string()))?,
    };

    // Create client
    let client = Client::from_stream(stream)
        .await
        .map_err(|e| SmtpError::Connection(e.to_string()))?;

    // Send EHLO
    let client = client
        .ehlo("localhost")
        .await
        .map_err(|e| SmtpError::Connection(e.to_string()))?;

    // Upgrade to TLS if using STARTTLS
    let client = if account.smtp.security == Security::StartTls {
        client
            .starttls(&account.smtp.host)
            .await
            .map_err(|e| SmtpError::Connection(e.to_string()))?
    } else {
        client
    };

    // Authenticate
    let client = client
        .auth_plain(&account.smtp.username, &account.smtp.password)
        .await
        .map_err(|e| SmtpError::Authentication(e.to_string()))?;

    // Start mail transaction
    let from_addr =
        Address::new(&message.from).map_err(|e| SmtpError::InvalidAddress(e.to_string()))?;

    let client = client
        .mail_from(from_addr)
        .await
        .map_err(|e| SmtpError::Send(e.to_string()))?;

    // Add first recipient
    let first_recipient =
        Address::new(all_recipients[0]).map_err(|e| SmtpError::InvalidAddress(e.to_string()))?;

    let mut client = client
        .rcpt_to(first_recipient)
        .await
        .map_err(|e| SmtpError::Send(e.to_string()))?;

    // Add remaining recipients
    for recipient in all_recipients.iter().skip(1) {
        let addr =
            Address::new(*recipient).map_err(|e| SmtpError::InvalidAddress(e.to_string()))?;
        client = client
            .rcpt_to(addr)
            .await
            .map_err(|e| SmtpError::Send(e.to_string()))?;
    }

    // Send message data
    let client = client
        .data()
        .await
        .map_err(|e| SmtpError::Send(e.to_string()))?;

    let rfc5322_message = message.to_rfc5322();
    let client = client
        .send_message(rfc5322_message.as_bytes())
        .await
        .map_err(|e| SmtpError::Send(e.to_string()))?;

    // Quit
    client
        .quit()
        .await
        .map_err(|e| SmtpError::Send(e.to_string()))?;

    Ok(())
}
