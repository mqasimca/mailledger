//! Type-state SMTP client.

use super::{ServerInfo, SmtpStream};
use crate::command::Command;
use crate::error::{Error, Result};
use crate::parser::{is_last_reply_line, parse_reply};
use crate::types::{Address, AuthMechanism, Extension, Reply, ReplyCode};
use base64::Engine;
use std::collections::HashSet;
use std::marker::PhantomData;

/// Type-state marker for connected state.
#[derive(Debug)]
pub struct Connected;

/// Type-state marker for authenticated state.
#[derive(Debug)]
pub struct Authenticated;

/// Type-state marker for mail transaction started.
#[derive(Debug)]
pub struct MailTransaction;

/// Type-state marker for recipient added.
#[derive(Debug)]
pub struct RecipientAdded;

/// Type-state marker for data mode.
#[derive(Debug)]
pub struct Data;

/// SMTP client with type-state pattern.
#[derive(Debug)]
pub struct Client<State> {
    stream: SmtpStream,
    server_info: ServerInfo,
    _state: PhantomData<State>,
}

/// Connection trait for all states.
pub trait SmtpConnection {
    /// Returns the server information.
    fn server_info(&self) -> &ServerInfo;
}

impl<S> SmtpConnection for Client<S> {
    fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }
}

impl Client<Connected> {
    /// Creates a client from a stream and reads the server greeting.
    ///
    /// # Errors
    ///
    /// Returns an error if reading the greeting fails or if the server returns an error.
    pub async fn from_stream(mut stream: SmtpStream) -> Result<Self> {
        let greeting = Self::read_reply(&mut stream).await?;
        if !greeting.is_success() {
            return Err(Error::smtp_error(
                greeting.code.as_u16(),
                greeting.message_text(),
            ));
        }

        // Extract hostname from greeting (first word after code)
        let hostname = greeting
            .message
            .first()
            .and_then(|msg| msg.split_whitespace().next())
            .unwrap_or("unknown")
            .to_string();

        Ok(Self {
            stream,
            server_info: ServerInfo {
                hostname,
                extensions: HashSet::new(),
            },
            _state: PhantomData,
        })
    }

    /// Sends EHLO and discovers server capabilities.
    ///
    /// # Errors
    ///
    /// Returns an error if the EHLO command fails.
    pub async fn ehlo(mut self, client_hostname: &str) -> Result<Self> {
        let cmd = Command::Ehlo {
            hostname: client_hostname.to_string(),
        };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        // Parse extensions from EHLO response (skip first line which is greeting)
        let mut extensions = HashSet::new();
        for line in reply.message.iter().skip(1) {
            extensions.insert(Extension::parse(line));
        }

        self.server_info.extensions = extensions;
        Ok(self)
    }

    /// Upgrades the connection to TLS using STARTTLS.
    ///
    /// # Errors
    ///
    /// Returns an error if STARTTLS is not supported or if the upgrade fails.
    pub async fn starttls(mut self, hostname: &str) -> Result<Self> {
        if !self.server_info.supports_starttls() {
            return Err(Error::NotSupported("STARTTLS".into()));
        }

        let cmd = Command::StartTls;
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        // Upgrade stream to TLS
        self.stream = self.stream.upgrade_to_tls(hostname).await?;

        // Send EHLO again after STARTTLS
        let cmd = Command::Ehlo {
            hostname: hostname.to_string(),
        };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        // Re-parse extensions
        let mut extensions = HashSet::new();
        for line in reply.message.iter().skip(1) {
            extensions.insert(Extension::parse(line));
        }
        self.server_info.extensions = extensions;

        Ok(self)
    }

    /// Authenticates using PLAIN mechanism.
    ///
    /// # Errors
    ///
    /// Returns an error if authentication fails.
    pub async fn auth_plain(
        mut self,
        username: &str,
        password: &str,
    ) -> Result<Client<Authenticated>> {
        // Build PLAIN response: \0username\0password
        let credentials = format!("\0{username}\0{password}");
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());

        let cmd = Command::Auth {
            mechanism: AuthMechanism::Plain,
            initial_response: Some(encoded),
        };

        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }

    /// Starts a mail transaction without authentication (if server allows).
    ///
    /// # Errors
    ///
    /// Returns an error if the MAIL FROM command fails.
    pub async fn mail_from(mut self, from: Address) -> Result<Client<MailTransaction>> {
        let cmd = Command::MailFrom {
            from,
            body: None,
            size: None,
        };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }
}

impl Client<Authenticated> {
    /// Starts a mail transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the MAIL FROM command fails.
    pub async fn mail_from(mut self, from: Address) -> Result<Client<MailTransaction>> {
        let cmd = Command::MailFrom {
            from,
            body: None,
            size: None,
        };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }
}

impl Client<MailTransaction> {
    /// Adds a recipient to the transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the RCPT TO command fails.
    pub async fn rcpt_to(mut self, to: Address) -> Result<Client<RecipientAdded>> {
        let cmd = Command::RcptTo { to };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }

    /// Resets the transaction and returns to authenticated/connected state.
    ///
    /// # Errors
    ///
    /// Returns an error if the RSET command fails.
    pub async fn reset(mut self) -> Result<Client<Connected>> {
        let cmd = Command::Rset;
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }
}

impl Client<RecipientAdded> {
    /// Adds another recipient to the transaction.
    ///
    /// # Errors
    ///
    /// Returns an error if the RCPT TO command fails.
    pub async fn rcpt_to(mut self, to: Address) -> Result<Self> {
        let cmd = Command::RcptTo { to };
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(self)
    }

    /// Begins sending message data.
    ///
    /// # Errors
    ///
    /// Returns an error if the DATA command fails.
    pub async fn data(mut self) -> Result<Client<Data>> {
        let cmd = Command::Data;
        let reply = self.send_command(cmd).await?;

        if reply.code != ReplyCode::START_DATA {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }

    /// Resets the transaction and returns to connected state.
    ///
    /// # Errors
    ///
    /// Returns an error if the RSET command fails.
    pub async fn reset(mut self) -> Result<Client<Connected>> {
        let cmd = Command::Rset;
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }
}

impl Client<Data> {
    /// Sends the message content and completes the transaction.
    ///
    /// Message should be RFC 5322 formatted. Line endings will be normalized to CRLF.
    /// The terminating "." line will be added automatically.
    ///
    /// # Errors
    ///
    /// Returns an error if sending the message fails or server rejects it.
    pub async fn send_message(mut self, message: &[u8]) -> Result<Client<Connected>> {
        // Send message with proper line ending normalization
        // and byte-stuffing (leading dots)
        for line in message.split(|&b| b == b'\n') {
            let line = if !line.is_empty() && line[line.len() - 1] == b'\r' {
                &line[..line.len() - 1]
            } else {
                line
            };

            // Byte-stuff lines starting with '.'
            if !line.is_empty() && line[0] == b'.' {
                self.stream.write_all(b".").await?;
            }

            self.stream.write_all(line).await?;
            self.stream.write_all(b"\r\n").await?;
        }

        // Send terminating sequence
        self.stream.write_all(b".\r\n").await?;

        // Read server response
        let reply = Self::read_reply(&mut self.stream).await?;

        if !reply.is_success() {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(Client {
            stream: self.stream,
            server_info: self.server_info,
            _state: PhantomData,
        })
    }
}

// Common implementation for all states
impl<S> Client<S> {
    async fn send_command(&mut self, cmd: Command) -> Result<Reply> {
        let data = cmd.serialize();
        self.stream.write_all(&data).await?;
        Self::read_reply(&mut self.stream).await
    }

    async fn read_reply(stream: &mut SmtpStream) -> Result<Reply> {
        let mut lines = Vec::new();
        loop {
            let line = stream.read_line().await?;
            if line.is_empty() {
                continue;
            }

            let is_last = is_last_reply_line(&line);
            lines.push(line);

            if is_last {
                break;
            }
        }

        parse_reply(&lines)
    }

    /// Sends QUIT and closes the connection (available in any state).
    ///
    /// # Errors
    ///
    /// Returns an error if the QUIT command fails.
    pub async fn quit(mut self) -> Result<()> {
        let cmd = Command::Quit;
        let reply = self.send_command(cmd).await?;

        if !reply.is_success() && reply.code != ReplyCode::CLOSING {
            return Err(Error::smtp_error(reply.code.as_u16(), reply.message_text()));
        }

        Ok(())
    }
}
