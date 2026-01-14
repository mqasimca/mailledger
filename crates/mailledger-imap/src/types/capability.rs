//! Server capabilities and response status.

/// Response status from a tagged response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Command completed successfully.
    Ok,
    /// Command failed (operational error).
    No,
    /// Command failed (protocol/syntax error).
    Bad,
    /// Server greeting (pre-authenticated).
    PreAuth,
    /// Server is closing connection.
    Bye,
}

impl Status {
    /// Returns true if this is a successful status.
    #[must_use]
    pub fn is_ok(self) -> bool {
        matches!(self, Self::Ok | Self::PreAuth)
    }
}

/// Server capability.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// `IMAP4rev1` (RFC 3501)
    Imap4Rev1,
    /// `IMAP4rev2` (RFC 9051)
    Imap4Rev2,
    /// IDLE command support (RFC 2177)
    Idle,
    /// NAMESPACE command support (RFC 2342)
    Namespace,
    /// UIDPLUS extension (RFC 4315)
    UidPlus,
    /// MOVE extension (RFC 6851)
    Move,
    /// LITERAL+ extension (RFC 7888)
    LiteralPlus,
    /// LITERAL- extension (RFC 7888)
    LiteralMinus,
    /// STARTTLS support
    StartTls,
    /// LOGIN disabled
    LoginDisabled,
    /// AUTH mechanism
    Auth(String),
    /// ENABLE command (RFC 5161)
    Enable,
    /// UTF8=ACCEPT (RFC 6855)
    Utf8Accept,
    /// CONDSTORE (RFC 7162)
    CondStore,
    /// QRESYNC (RFC 7162)
    QResync,
    /// Unstrict (RFC 9586)
    Unstrict,
    /// ID extension (RFC 2971)
    Id,
    /// SPECIAL-USE mailboxes (RFC 6154)
    SpecialUse,
    /// Unknown capability
    Unknown(String),
}

impl Capability {
    /// Parses a capability string.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        let upper = s.to_uppercase();
        match upper.as_str() {
            "IMAP4REV1" => Self::Imap4Rev1,
            "IMAP4REV2" => Self::Imap4Rev2,
            "IDLE" => Self::Idle,
            "NAMESPACE" => Self::Namespace,
            "UIDPLUS" => Self::UidPlus,
            "MOVE" => Self::Move,
            "LITERAL+" => Self::LiteralPlus,
            "LITERAL-" => Self::LiteralMinus,
            "STARTTLS" => Self::StartTls,
            "LOGINDISABLED" => Self::LoginDisabled,
            "ENABLE" => Self::Enable,
            "UTF8=ACCEPT" => Self::Utf8Accept,
            "CONDSTORE" => Self::CondStore,
            "QRESYNC" => Self::QResync,
            "UNSTRICT" => Self::Unstrict,
            "ID" => Self::Id,
            "SPECIAL-USE" => Self::SpecialUse,
            _ if upper.starts_with("AUTH=") => Self::Auth(s[5..].to_string()),
            _ => Self::Unknown(s.to_string()),
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Imap4Rev1 => write!(f, "IMAP4rev1"),
            Self::Imap4Rev2 => write!(f, "IMAP4rev2"),
            Self::Idle => write!(f, "IDLE"),
            Self::Namespace => write!(f, "NAMESPACE"),
            Self::UidPlus => write!(f, "UIDPLUS"),
            Self::Move => write!(f, "MOVE"),
            Self::LiteralPlus => write!(f, "LITERAL+"),
            Self::LiteralMinus => write!(f, "LITERAL-"),
            Self::StartTls => write!(f, "STARTTLS"),
            Self::LoginDisabled => write!(f, "LOGINDISABLED"),
            Self::Auth(mech) => write!(f, "AUTH={mech}"),
            Self::Enable => write!(f, "ENABLE"),
            Self::Utf8Accept => write!(f, "UTF8=ACCEPT"),
            Self::CondStore => write!(f, "CONDSTORE"),
            Self::QResync => write!(f, "QRESYNC"),
            Self::Unstrict => write!(f, "UNSTRICT"),
            Self::Id => write!(f, "ID"),
            Self::SpecialUse => write!(f, "SPECIAL-USE"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod status_tests {
        use super::*;

        #[test]
        fn is_ok_for_ok() {
            assert!(Status::Ok.is_ok());
        }

        #[test]
        fn is_ok_for_preauth() {
            assert!(Status::PreAuth.is_ok());
        }

        #[test]
        fn is_ok_false_for_no() {
            assert!(!Status::No.is_ok());
        }

        #[test]
        fn is_ok_false_for_bad() {
            assert!(!Status::Bad.is_ok());
        }

        #[test]
        fn is_ok_false_for_bye() {
            assert!(!Status::Bye.is_ok());
        }
    }

    mod capability_parse_tests {
        use super::*;

        #[test]
        fn parse_imap4rev1() {
            assert_eq!(Capability::parse("IMAP4REV1"), Capability::Imap4Rev1);
            assert_eq!(Capability::parse("imap4rev1"), Capability::Imap4Rev1);
        }

        #[test]
        fn parse_imap4rev2() {
            assert_eq!(Capability::parse("IMAP4REV2"), Capability::Imap4Rev2);
        }

        #[test]
        fn parse_idle() {
            assert_eq!(Capability::parse("IDLE"), Capability::Idle);
        }

        #[test]
        fn parse_namespace() {
            assert_eq!(Capability::parse("NAMESPACE"), Capability::Namespace);
        }

        #[test]
        fn parse_uidplus() {
            assert_eq!(Capability::parse("UIDPLUS"), Capability::UidPlus);
        }

        #[test]
        fn parse_move() {
            assert_eq!(Capability::parse("MOVE"), Capability::Move);
        }

        #[test]
        fn parse_literal_plus() {
            assert_eq!(Capability::parse("LITERAL+"), Capability::LiteralPlus);
        }

        #[test]
        fn parse_literal_minus() {
            assert_eq!(Capability::parse("LITERAL-"), Capability::LiteralMinus);
        }

        #[test]
        fn parse_starttls() {
            assert_eq!(Capability::parse("STARTTLS"), Capability::StartTls);
        }

        #[test]
        fn parse_logindisabled() {
            assert_eq!(
                Capability::parse("LOGINDISABLED"),
                Capability::LoginDisabled
            );
        }

        #[test]
        fn parse_auth() {
            let cap = Capability::parse("AUTH=PLAIN");
            assert_eq!(cap, Capability::Auth("PLAIN".to_string()));
        }

        #[test]
        fn parse_auth_xoauth2() {
            let cap = Capability::parse("AUTH=XOAUTH2");
            assert_eq!(cap, Capability::Auth("XOAUTH2".to_string()));
        }

        #[test]
        fn parse_enable() {
            assert_eq!(Capability::parse("ENABLE"), Capability::Enable);
        }

        #[test]
        fn parse_utf8_accept() {
            assert_eq!(Capability::parse("UTF8=ACCEPT"), Capability::Utf8Accept);
        }

        #[test]
        fn parse_condstore() {
            assert_eq!(Capability::parse("CONDSTORE"), Capability::CondStore);
        }

        #[test]
        fn parse_qresync() {
            assert_eq!(Capability::parse("QRESYNC"), Capability::QResync);
        }

        #[test]
        fn parse_unstrict() {
            assert_eq!(Capability::parse("UNSTRICT"), Capability::Unstrict);
        }

        #[test]
        fn parse_id() {
            assert_eq!(Capability::parse("ID"), Capability::Id);
        }

        #[test]
        fn parse_special_use() {
            assert_eq!(Capability::parse("SPECIAL-USE"), Capability::SpecialUse);
        }

        #[test]
        fn parse_unknown() {
            let cap = Capability::parse("XSOMETHING");
            assert_eq!(cap, Capability::Unknown("XSOMETHING".to_string()));
        }
    }

    mod capability_display_tests {
        use super::*;

        #[test]
        fn display_imap4rev1() {
            assert_eq!(format!("{}", Capability::Imap4Rev1), "IMAP4rev1");
        }

        #[test]
        fn display_imap4rev2() {
            assert_eq!(format!("{}", Capability::Imap4Rev2), "IMAP4rev2");
        }

        #[test]
        fn display_auth() {
            assert_eq!(
                format!("{}", Capability::Auth("PLAIN".to_string())),
                "AUTH=PLAIN"
            );
        }

        #[test]
        fn display_unknown() {
            assert_eq!(
                format!("{}", Capability::Unknown("CUSTOM".to_string())),
                "CUSTOM"
            );
        }
    }
}
