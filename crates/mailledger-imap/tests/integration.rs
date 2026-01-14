//! Integration tests for the IMAP client.
//!
//! These tests use a mock stream to simulate IMAP server responses
//! without requiring a real server connection.

use std::io::{self, Cursor};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use mailledger_imap::{Client, FetchItems, ResponseParser, SequenceSet};

/// Mock stream that returns predefined responses.
struct MockStream {
    /// Responses to return (in order).
    responses: Cursor<Vec<u8>>,
    /// Captured commands sent by the client.
    sent: Vec<u8>,
}

impl MockStream {
    fn new(responses: &[u8]) -> Self {
        Self {
            responses: Cursor::new(responses.to_vec()),
            sent: Vec::new(),
        }
    }

    #[allow(dead_code)]
    fn sent_data(&self) -> &[u8] {
        &self.sent
    }
}

impl AsyncRead for MockStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let data = self.responses.get_ref();
        let pos = self.responses.position() as usize;

        if pos >= data.len() {
            return Poll::Ready(Ok(()));
        }

        let remaining = &data[pos..];
        let to_read = remaining.len().min(buf.remaining());
        buf.put_slice(&remaining[..to_read]);
        self.responses.set_position((pos + to_read) as u64);

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for MockStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.sent.extend_from_slice(buf);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

#[test]
fn test_parser_capability() {
    let response = b"* CAPABILITY IMAP4rev1 IDLE NAMESPACE\r\n";
    let parsed = ResponseParser::parse(response).unwrap();

    match parsed {
        mailledger_imap::Response::Untagged(mailledger_imap::UntaggedResponse::Capability(
            caps,
        )) => {
            assert!(
                caps.iter()
                    .any(|c| matches!(c, mailledger_imap::Capability::Imap4Rev1))
            );
            assert!(
                caps.iter()
                    .any(|c| matches!(c, mailledger_imap::Capability::Idle))
            );
        }
        _ => panic!("Expected capability response"),
    }
}

#[test]
fn test_parser_exists() {
    let response = b"* 23 EXISTS\r\n";
    let parsed = ResponseParser::parse(response).unwrap();

    match parsed {
        mailledger_imap::Response::Untagged(mailledger_imap::UntaggedResponse::Exists(n)) => {
            assert_eq!(n, 23);
        }
        _ => panic!("Expected EXISTS response"),
    }
}

#[test]
fn test_parser_fetch_response() {
    let response = b"* 12 FETCH (FLAGS (\\Seen) UID 100)\r\n";
    let parsed = ResponseParser::parse(response).unwrap();

    match parsed {
        mailledger_imap::Response::Untagged(mailledger_imap::UntaggedResponse::Fetch {
            seq,
            items,
        }) => {
            assert_eq!(seq.get(), 12);
            assert!(!items.is_empty());
        }
        _ => panic!("Expected FETCH response"),
    }
}

#[test]
fn test_parser_list_response() {
    let response = b"* LIST (\\HasNoChildren) \"/\" \"INBOX\"\r\n";
    let parsed = ResponseParser::parse(response).unwrap();

    match parsed {
        mailledger_imap::Response::Untagged(mailledger_imap::UntaggedResponse::List(list)) => {
            assert_eq!(list.mailbox.as_str(), "INBOX");
            assert_eq!(list.delimiter, Some('/'));
        }
        _ => panic!("Expected LIST response"),
    }
}

#[test]
fn test_parser_tagged_ok() {
    let response = b"A001 OK LOGIN completed\r\n";
    let parsed = ResponseParser::parse(response).unwrap();

    match parsed {
        mailledger_imap::Response::Tagged {
            tag, status, text, ..
        } => {
            assert_eq!(tag.as_str(), "A001");
            assert!(status.is_ok());
            assert!(text.contains("LOGIN"));
        }
        _ => panic!("Expected tagged response"),
    }
}

#[test]
fn test_sequence_set_display() {
    let seq = SequenceSet::range(1, 10).unwrap();
    assert_eq!(seq.to_string(), "1:10");

    let seq = SequenceSet::All;
    assert_eq!(seq.to_string(), "*");
}

#[test]
fn test_fetch_items_display() {
    // FetchItems provides different fetch macros
    let _fast = FetchItems::Fast;
    let _full = FetchItems::Full;
    let _all = FetchItems::All;
}

#[tokio::test]
async fn test_client_greeting() {
    // Simulate server greeting
    let greeting = b"* OK IMAP4rev1 Service Ready\r\n\
                    * CAPABILITY IMAP4rev1 IDLE\r\n\
                    A001 OK CAPABILITY completed\r\n";

    let stream = MockStream::new(greeting);
    let client = Client::from_stream(stream).await;

    // Should successfully parse the greeting
    assert!(client.is_ok());
}

#[test]
fn test_server_quirks_detection() {
    use mailledger_imap::{Capability, ServerType};

    // Gmail detection
    let caps = vec![Capability::Unknown("X-GM-EXT-1".to_string())];
    assert_eq!(ServerType::detect(&caps, None), ServerType::Gmail);

    // Dovecot detection from greeting
    let caps = vec![Capability::Imap4Rev1];
    assert_eq!(
        ServerType::detect(&caps, Some("* OK Dovecot ready.")),
        ServerType::Dovecot
    );
}

#[test]
fn test_server_quirks_idle_timeout() {
    use mailledger_imap::{ServerQuirks, ServerType};

    let gmail_quirks = ServerQuirks::for_server(ServerType::Gmail, &[]);
    assert_eq!(gmail_quirks.idle_timeout_secs, 600); // 10 minutes

    let dovecot_quirks = ServerQuirks::for_server(ServerType::Dovecot, &[]);
    assert_eq!(dovecot_quirks.idle_timeout_secs, 1740); // 29 minutes
}

#[test]
fn test_mailbox_normalization() {
    use mailledger_imap::{ServerQuirks, ServerType};

    let quirks = ServerQuirks::for_server(ServerType::Unknown, &[]);

    // INBOX should be normalized to uppercase
    assert_eq!(quirks.normalize_mailbox("inbox"), "INBOX");
    assert_eq!(quirks.normalize_mailbox("INBOX"), "INBOX");
    assert_eq!(quirks.normalize_mailbox("InBoX"), "INBOX");

    // Other mailboxes should not be changed
    assert_eq!(quirks.normalize_mailbox("Sent"), "Sent");
}

#[test]
fn test_flags_parsing() {
    use mailledger_imap::Flag;

    assert_eq!(Flag::parse("\\Seen"), Flag::Seen);
    assert_eq!(Flag::parse("\\Flagged"), Flag::Flagged);
    assert_eq!(Flag::parse("\\Deleted"), Flag::Deleted);
    assert_eq!(Flag::parse("\\Draft"), Flag::Draft);
    assert_eq!(Flag::parse("\\Answered"), Flag::Answered);
    assert_eq!(Flag::parse("\\Recent"), Flag::Recent);

    // Custom keyword flag
    match Flag::parse("$Important") {
        Flag::Keyword(s) => assert_eq!(s, "$Important"),
        _ => panic!("Expected keyword flag"),
    }
}

#[test]
fn test_capability_parsing() {
    use mailledger_imap::Capability;

    assert!(matches!(
        Capability::parse("IMAP4rev1"),
        Capability::Imap4Rev1
    ));
    assert!(matches!(
        Capability::parse("IMAP4rev2"),
        Capability::Imap4Rev2
    ));
    assert!(matches!(Capability::parse("IDLE"), Capability::Idle));
    assert!(matches!(Capability::parse("MOVE"), Capability::Move));

    // Auth mechanism
    match Capability::parse("AUTH=PLAIN") {
        Capability::Auth(mech) => assert_eq!(mech, "PLAIN"),
        _ => panic!("Expected AUTH capability"),
    }
}
