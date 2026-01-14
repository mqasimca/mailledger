//! SMTP command builder.

use crate::types::{Address, AuthMechanism};

/// SMTP command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// HELO - Simple greeting
    Helo {
        /// Client hostname
        hostname: String,
    },
    /// EHLO - Extended greeting
    Ehlo {
        /// Client hostname
        hostname: String,
    },
    /// STARTTLS - Upgrade to TLS
    StartTls,
    /// AUTH - Begin authentication
    Auth {
        /// Authentication mechanism
        mechanism: AuthMechanism,
        /// Initial response (optional, for SASL-IR)
        initial_response: Option<String>,
    },
    /// MAIL FROM - Start mail transaction
    MailFrom {
        /// Sender address
        from: Address,
        /// BODY parameter (7BIT, 8BITMIME)
        body: Option<String>,
        /// SIZE parameter
        size: Option<usize>,
    },
    /// RCPT TO - Add recipient
    RcptTo {
        /// Recipient address
        to: Address,
    },
    /// DATA - Begin message data
    Data,
    /// RSET - Reset transaction
    Rset,
    /// VRFY - Verify address
    Vrfy {
        /// Address to verify
        address: String,
    },
    /// NOOP - No operation
    Noop,
    /// QUIT - Close connection
    Quit,
}

impl Command {
    /// Serializes the command to bytes.
    #[must_use]
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Self::Helo { hostname } => {
                buf.extend_from_slice(b"HELO ");
                buf.extend_from_slice(hostname.as_bytes());
            }
            Self::Ehlo { hostname } => {
                buf.extend_from_slice(b"EHLO ");
                buf.extend_from_slice(hostname.as_bytes());
            }
            Self::StartTls => {
                buf.extend_from_slice(b"STARTTLS");
            }
            Self::Auth {
                mechanism,
                initial_response,
            } => {
                buf.extend_from_slice(b"AUTH ");
                buf.extend_from_slice(mechanism.as_str().as_bytes());
                if let Some(resp) = initial_response {
                    buf.push(b' ');
                    buf.extend_from_slice(resp.as_bytes());
                }
            }
            Self::MailFrom { from, body, size } => {
                buf.extend_from_slice(b"MAIL FROM:<");
                buf.extend_from_slice(from.as_str().as_bytes());
                buf.push(b'>');
                if let Some(body_type) = body {
                    buf.extend_from_slice(b" BODY=");
                    buf.extend_from_slice(body_type.as_bytes());
                }
                if let Some(msg_size) = size {
                    buf.extend_from_slice(format!(" SIZE={msg_size}").as_bytes());
                }
            }
            Self::RcptTo { to } => {
                buf.extend_from_slice(b"RCPT TO:<");
                buf.extend_from_slice(to.as_str().as_bytes());
                buf.push(b'>');
            }
            Self::Data => {
                buf.extend_from_slice(b"DATA");
            }
            Self::Rset => {
                buf.extend_from_slice(b"RSET");
            }
            Self::Vrfy { address } => {
                buf.extend_from_slice(b"VRFY ");
                buf.extend_from_slice(address.as_bytes());
            }
            Self::Noop => {
                buf.extend_from_slice(b"NOOP");
            }
            Self::Quit => {
                buf.extend_from_slice(b"QUIT");
            }
        }

        buf.extend_from_slice(b"\r\n");
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helo_command() {
        let cmd = Command::Helo {
            hostname: "client.example.com".to_string(),
        };
        assert_eq!(cmd.serialize(), b"HELO client.example.com\r\n");
    }

    #[test]
    fn test_ehlo_command() {
        let cmd = Command::Ehlo {
            hostname: "client.example.com".to_string(),
        };
        assert_eq!(cmd.serialize(), b"EHLO client.example.com\r\n");
    }

    #[test]
    fn test_starttls_command() {
        let cmd = Command::StartTls;
        assert_eq!(cmd.serialize(), b"STARTTLS\r\n");
    }

    #[test]
    fn test_auth_plain() {
        let cmd = Command::Auth {
            mechanism: AuthMechanism::Plain,
            initial_response: Some("AHVzZXIAcGFzcw==".to_string()),
        };
        assert_eq!(cmd.serialize(), b"AUTH PLAIN AHVzZXIAcGFzcw==\r\n");
    }

    #[test]
    fn test_mail_from_simple() {
        let cmd = Command::MailFrom {
            from: Address::new("sender@example.com").unwrap(),
            body: None,
            size: None,
        };
        assert_eq!(cmd.serialize(), b"MAIL FROM:<sender@example.com>\r\n");
    }

    #[test]
    fn test_mail_from_with_params() {
        let cmd = Command::MailFrom {
            from: Address::new("sender@example.com").unwrap(),
            body: Some("8BITMIME".to_string()),
            size: Some(12345),
        };
        assert_eq!(
            cmd.serialize(),
            b"MAIL FROM:<sender@example.com> BODY=8BITMIME SIZE=12345\r\n"
        );
    }

    #[test]
    fn test_rcpt_to_command() {
        let cmd = Command::RcptTo {
            to: Address::new("recipient@example.com").unwrap(),
        };
        assert_eq!(cmd.serialize(), b"RCPT TO:<recipient@example.com>\r\n");
    }

    #[test]
    fn test_data_command() {
        let cmd = Command::Data;
        assert_eq!(cmd.serialize(), b"DATA\r\n");
    }

    #[test]
    fn test_rset_command() {
        let cmd = Command::Rset;
        assert_eq!(cmd.serialize(), b"RSET\r\n");
    }

    #[test]
    fn test_quit_command() {
        let cmd = Command::Quit;
        assert_eq!(cmd.serialize(), b"QUIT\r\n");
    }

    #[test]
    fn test_noop_command() {
        let cmd = Command::Noop;
        assert_eq!(cmd.serialize(), b"NOOP\r\n");
    }
}
