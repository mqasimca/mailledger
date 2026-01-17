//! SMTP reply types.

/// SMTP reply from server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    /// Reply code (e.g., 250).
    pub code: ReplyCode,
    /// Reply message lines.
    pub message: Vec<String>,
}

impl Reply {
    /// Creates a new reply.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec is not const-compatible
    pub fn new(code: ReplyCode, message: Vec<String>) -> Self {
        Self { code, message }
    }

    /// Returns true if this is a success reply (2xx).
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.code.is_success()
    }

    /// Returns true if this is a transient error (4xx).
    #[must_use]
    pub const fn is_transient_error(&self) -> bool {
        self.code.is_transient()
    }

    /// Returns true if this is a permanent error (5xx).
    #[must_use]
    pub const fn is_permanent_error(&self) -> bool {
        self.code.is_permanent()
    }

    /// Returns the full message as a single string.
    #[must_use]
    pub fn message_text(&self) -> String {
        self.message.join("\n")
    }
}

/// SMTP reply code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReplyCode(u16);

impl ReplyCode {
    /// Creates a new reply code.
    #[must_use]
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// Returns the numeric code.
    #[must_use]
    pub const fn as_u16(self) -> u16 {
        self.0
    }

    /// Returns true if this is a success code (2xx).
    #[must_use]
    pub const fn is_success(self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    /// Returns true if this is a transient error (4xx).
    #[must_use]
    pub const fn is_transient(self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    /// Returns true if this is a permanent error (5xx).
    #[must_use]
    pub const fn is_permanent(self) -> bool {
        self.0 >= 500 && self.0 < 600
    }

    /// Returns true if this is an intermediate reply (3xx).
    #[must_use]
    pub const fn is_intermediate(self) -> bool {
        self.0 >= 300 && self.0 < 400
    }
}

impl std::fmt::Display for ReplyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Common reply codes
impl ReplyCode {
    /// 220 Service ready
    pub const SERVICE_READY: Self = Self(220);
    /// 221 Service closing transmission channel
    pub const CLOSING: Self = Self(221);
    /// 250 Requested mail action okay, completed
    pub const OK: Self = Self(250);
    /// 251 User not local; will forward
    pub const FORWARD: Self = Self(251);
    /// 334 Continue with authentication
    pub const AUTH_CONTINUE: Self = Self(334);
    /// 354 Start mail input
    pub const START_DATA: Self = Self(354);
    /// 421 Service not available, closing transmission channel
    pub const SERVICE_UNAVAILABLE: Self = Self(421);
    /// 450 Mailbox unavailable (busy)
    pub const MAILBOX_BUSY: Self = Self(450);
    /// 451 Local error in processing
    pub const LOCAL_ERROR: Self = Self(451);
    /// 452 Insufficient system storage
    pub const INSUFFICIENT_STORAGE: Self = Self(452);
    /// 500 Syntax error, command unrecognized
    pub const SYNTAX_ERROR: Self = Self(500);
    /// 501 Syntax error in parameters or arguments
    pub const PARAMETER_ERROR: Self = Self(501);
    /// 502 Command not implemented
    pub const NOT_IMPLEMENTED: Self = Self(502);
    /// 503 Bad sequence of commands
    pub const BAD_SEQUENCE: Self = Self(503);
    /// 504 Command parameter not implemented
    pub const PARAMETER_NOT_IMPLEMENTED: Self = Self(504);
    /// 535 Authentication credentials invalid
    pub const AUTH_FAILED: Self = Self(535);
    /// 550 Mailbox unavailable (not found, access denied)
    pub const MAILBOX_UNAVAILABLE: Self = Self(550);
    /// 551 User not local
    pub const USER_NOT_LOCAL: Self = Self(551);
    /// 552 Exceeded storage allocation
    pub const EXCEEDED_STORAGE: Self = Self(552);
    /// 553 Mailbox name not allowed
    pub const MAILBOX_NAME_INVALID: Self = Self(553);
    /// 554 Transaction failed
    pub const TRANSACTION_FAILED: Self = Self(554);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    mod reply_code_tests {
        use super::*;

        #[test]
        fn success_codes() {
            assert!(ReplyCode::OK.is_success());
            assert!(ReplyCode::SERVICE_READY.is_success());
            assert!(ReplyCode::CLOSING.is_success());
            assert!(ReplyCode::FORWARD.is_success());
        }

        #[test]
        fn not_success_codes() {
            assert!(!ReplyCode::OK.is_transient());
            assert!(!ReplyCode::OK.is_permanent());
            assert!(!ReplyCode::OK.is_intermediate());
        }

        #[test]
        fn intermediate_codes() {
            assert!(ReplyCode::AUTH_CONTINUE.is_intermediate());
            assert!(ReplyCode::START_DATA.is_intermediate());
        }

        #[test]
        fn transient_errors() {
            assert!(ReplyCode::MAILBOX_BUSY.is_transient());
            assert!(ReplyCode::SERVICE_UNAVAILABLE.is_transient());
            assert!(ReplyCode::LOCAL_ERROR.is_transient());
            assert!(ReplyCode::INSUFFICIENT_STORAGE.is_transient());
        }

        #[test]
        fn permanent_errors() {
            assert!(ReplyCode::MAILBOX_UNAVAILABLE.is_permanent());
            assert!(ReplyCode::SYNTAX_ERROR.is_permanent());
            assert!(ReplyCode::PARAMETER_ERROR.is_permanent());
            assert!(ReplyCode::NOT_IMPLEMENTED.is_permanent());
            assert!(ReplyCode::BAD_SEQUENCE.is_permanent());
            assert!(ReplyCode::AUTH_FAILED.is_permanent());
        }

        #[test]
        fn as_u16() {
            assert_eq!(ReplyCode::OK.as_u16(), 250);
            assert_eq!(ReplyCode::SERVICE_READY.as_u16(), 220);
            assert_eq!(ReplyCode::AUTH_FAILED.as_u16(), 535);
        }

        #[test]
        fn new() {
            let code = ReplyCode::new(200);
            assert!(code.is_success());
            assert_eq!(code.as_u16(), 200);
        }

        #[test]
        fn display() {
            assert_eq!(format!("{}", ReplyCode::OK), "250");
            assert_eq!(format!("{}", ReplyCode::SYNTAX_ERROR), "500");
        }

        #[test]
        fn ordering() {
            assert!(ReplyCode::OK < ReplyCode::MAILBOX_BUSY);
            assert!(ReplyCode::MAILBOX_BUSY < ReplyCode::MAILBOX_UNAVAILABLE);
        }
    }

    mod reply_tests {
        use super::*;

        #[test]
        fn new() {
            let reply = Reply::new(ReplyCode::OK, vec!["OK".to_string()]);
            assert_eq!(reply.code, ReplyCode::OK);
            assert_eq!(reply.message.len(), 1);
        }

        #[test]
        fn is_success() {
            let reply = Reply::new(ReplyCode::OK, vec!["OK".to_string()]);
            assert!(reply.is_success());
            assert!(!reply.is_transient_error());
            assert!(!reply.is_permanent_error());
        }

        #[test]
        fn is_transient_error() {
            let reply = Reply::new(ReplyCode::MAILBOX_BUSY, vec!["Busy".to_string()]);
            assert!(!reply.is_success());
            assert!(reply.is_transient_error());
            assert!(!reply.is_permanent_error());
        }

        #[test]
        fn is_permanent_error() {
            let reply = Reply::new(
                ReplyCode::MAILBOX_UNAVAILABLE,
                vec!["Not found".to_string()],
            );
            assert!(!reply.is_success());
            assert!(!reply.is_transient_error());
            assert!(reply.is_permanent_error());
        }

        #[test]
        fn message_text_single_line() {
            let reply = Reply::new(ReplyCode::OK, vec!["Message sent".to_string()]);
            assert_eq!(reply.message_text(), "Message sent");
        }

        #[test]
        fn message_text_multiple_lines() {
            let reply = Reply::new(
                ReplyCode::SERVICE_READY,
                vec![
                    "smtp.example.com ESMTP".to_string(),
                    "Ready to serve".to_string(),
                ],
            );
            assert_eq!(
                reply.message_text(),
                "smtp.example.com ESMTP\nReady to serve"
            );
        }

        #[test]
        fn message_text_empty() {
            let reply = Reply::new(ReplyCode::OK, vec![]);
            assert_eq!(reply.message_text(), "");
        }
    }
}
