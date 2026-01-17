//! SMTP extension types.

/// SMTP extensions discovered from EHLO response.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Extension {
    /// STARTTLS - TLS upgrade
    StartTls,
    /// AUTH - Authentication
    Auth(Vec<AuthMechanism>),
    /// SIZE - Maximum message size
    Size(Option<usize>),
    /// 8BITMIME - 8-bit MIME transport
    EightBitMime,
    /// PIPELINING - Command pipelining
    Pipelining,
    /// CHUNKING - Chunked message transfer
    Chunking,
    /// SMTPUTF8 - UTF-8 email addresses
    SmtpUtf8,
    /// DSN - Delivery status notifications
    Dsn,
    /// BINARYMIME - Binary MIME
    BinaryMime,
    /// Unknown extension
    Unknown(String),
}

impl Extension {
    /// Parses an extension line from EHLO response.
    #[must_use]
    pub fn parse(line: &str) -> Self {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return Self::Unknown(line.to_string());
        }

        let keyword = parts[0].to_uppercase();
        match keyword.as_str() {
            "STARTTLS" => Self::StartTls,
            "AUTH" => {
                let mechanisms = parts[1..]
                    .iter()
                    .filter_map(|m| AuthMechanism::parse(m))
                    .collect();
                Self::Auth(mechanisms)
            }
            "SIZE" => {
                let size = parts.get(1).and_then(|s| s.parse().ok());
                Self::Size(size)
            }
            "8BITMIME" => Self::EightBitMime,
            "PIPELINING" => Self::Pipelining,
            "CHUNKING" => Self::Chunking,
            "SMTPUTF8" => Self::SmtpUtf8,
            "DSN" => Self::Dsn,
            "BINARYMIME" => Self::BinaryMime,
            _ => Self::Unknown(line.to_string()),
        }
    }
}

/// SASL authentication mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthMechanism {
    /// PLAIN - plaintext authentication
    Plain,
    /// LOGIN - legacy plaintext
    Login,
    /// CRAM-MD5 - challenge-response
    CramMd5,
    /// `XOAUTH2` - `OAuth2` (Google/Microsoft)
    XOAuth2,
    /// `OAUTHBEARER` - RFC 7628 `OAuth2`
    OAuthBearer,
}

impl AuthMechanism {
    /// Parses an authentication mechanism name.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PLAIN" => Some(Self::Plain),
            "LOGIN" => Some(Self::Login),
            "CRAM-MD5" => Some(Self::CramMd5),
            "XOAUTH2" => Some(Self::XOAuth2),
            "OAUTHBEARER" => Some(Self::OAuthBearer),
            _ => None,
        }
    }

    /// Returns the mechanism name as a string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "PLAIN",
            Self::Login => "LOGIN",
            Self::CramMd5 => "CRAM-MD5",
            Self::XOAuth2 => "XOAUTH2",
            Self::OAuthBearer => "OAUTHBEARER",
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    mod extension_parse_tests {
        use super::*;

        #[test]
        fn parse_starttls() {
            assert_eq!(Extension::parse("STARTTLS"), Extension::StartTls);
        }

        #[test]
        fn parse_starttls_lowercase() {
            assert_eq!(Extension::parse("starttls"), Extension::StartTls);
        }

        #[test]
        fn parse_auth_plain() {
            let ext = Extension::parse("AUTH PLAIN LOGIN");
            if let Extension::Auth(mechs) = ext {
                assert_eq!(mechs.len(), 2);
                assert!(mechs.contains(&AuthMechanism::Plain));
                assert!(mechs.contains(&AuthMechanism::Login));
            } else {
                panic!("Expected Auth variant");
            }
        }

        #[test]
        fn parse_auth_xoauth2() {
            let ext = Extension::parse("AUTH XOAUTH2 OAUTHBEARER");
            if let Extension::Auth(mechs) = ext {
                assert!(mechs.contains(&AuthMechanism::XOAuth2));
                assert!(mechs.contains(&AuthMechanism::OAuthBearer));
            } else {
                panic!("Expected Auth variant");
            }
        }

        #[test]
        fn parse_size_with_value() {
            let ext = Extension::parse("SIZE 52428800");
            if let Extension::Size(size) = ext {
                assert_eq!(size, Some(52_428_800));
            } else {
                panic!("Expected Size variant");
            }
        }

        #[test]
        fn parse_size_without_value() {
            let ext = Extension::parse("SIZE");
            if let Extension::Size(size) = ext {
                assert_eq!(size, None);
            } else {
                panic!("Expected Size variant");
            }
        }

        #[test]
        fn parse_8bitmime() {
            assert_eq!(Extension::parse("8BITMIME"), Extension::EightBitMime);
        }

        #[test]
        fn parse_pipelining() {
            assert_eq!(Extension::parse("PIPELINING"), Extension::Pipelining);
        }

        #[test]
        fn parse_chunking() {
            assert_eq!(Extension::parse("CHUNKING"), Extension::Chunking);
        }

        #[test]
        fn parse_smtputf8() {
            assert_eq!(Extension::parse("SMTPUTF8"), Extension::SmtpUtf8);
        }

        #[test]
        fn parse_dsn() {
            assert_eq!(Extension::parse("DSN"), Extension::Dsn);
        }

        #[test]
        fn parse_binarymime() {
            assert_eq!(Extension::parse("BINARYMIME"), Extension::BinaryMime);
        }

        #[test]
        fn parse_unknown() {
            let ext = Extension::parse("SOMECUSTOMEXT");
            if let Extension::Unknown(s) = ext {
                assert_eq!(s, "SOMECUSTOMEXT");
            } else {
                panic!("Expected Unknown variant");
            }
        }

        #[test]
        fn parse_empty() {
            let ext = Extension::parse("");
            assert!(matches!(ext, Extension::Unknown(_)));
        }
    }

    mod auth_mechanism_tests {
        use super::*;

        #[test]
        fn parse_plain() {
            assert_eq!(AuthMechanism::parse("PLAIN"), Some(AuthMechanism::Plain));
            assert_eq!(AuthMechanism::parse("plain"), Some(AuthMechanism::Plain));
        }

        #[test]
        fn parse_login() {
            assert_eq!(AuthMechanism::parse("LOGIN"), Some(AuthMechanism::Login));
        }

        #[test]
        fn parse_cram_md5() {
            assert_eq!(
                AuthMechanism::parse("CRAM-MD5"),
                Some(AuthMechanism::CramMd5)
            );
        }

        #[test]
        fn parse_xoauth2() {
            assert_eq!(
                AuthMechanism::parse("XOAUTH2"),
                Some(AuthMechanism::XOAuth2)
            );
        }

        #[test]
        fn parse_oauthbearer() {
            assert_eq!(
                AuthMechanism::parse("OAUTHBEARER"),
                Some(AuthMechanism::OAuthBearer)
            );
        }

        #[test]
        fn parse_unknown() {
            assert_eq!(AuthMechanism::parse("UNKNOWN"), None);
        }

        #[test]
        fn as_str() {
            assert_eq!(AuthMechanism::Plain.as_str(), "PLAIN");
            assert_eq!(AuthMechanism::Login.as_str(), "LOGIN");
            assert_eq!(AuthMechanism::CramMd5.as_str(), "CRAM-MD5");
            assert_eq!(AuthMechanism::XOAuth2.as_str(), "XOAUTH2");
            assert_eq!(AuthMechanism::OAuthBearer.as_str(), "OAUTHBEARER");
        }
    }
}
