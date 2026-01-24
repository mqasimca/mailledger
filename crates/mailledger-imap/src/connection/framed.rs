//! Framed I/O for IMAP protocol.
//!
//! IMAP uses CRLF-terminated lines with support for literals.
//! This module provides buffered reading and writing with proper
//! handling of the IMAP framing.

#![allow(clippy::missing_errors_doc)]

use std::io;

use bytes::BytesMut;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::Result;

/// Default buffer size for reading.
const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Maximum line length to prevent memory exhaustion.
const MAX_LINE_LENGTH: usize = 1024 * 1024; // 1 MB

/// Maximum literal size to prevent memory exhaustion.
const MAX_LITERAL_SIZE: usize = 100 * 1024 * 1024; // 100 MB

/// Framed connection for IMAP protocol.
///
/// Handles line-based reading with literal support and buffered writing.
pub struct FramedStream<S> {
    reader: BufReader<S>,
    write_buffer: BytesMut,
}

impl<S> FramedStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Creates a new framed stream.
    pub fn new(stream: S) -> Self {
        Self {
            reader: BufReader::with_capacity(DEFAULT_BUFFER_SIZE, stream),
            write_buffer: BytesMut::with_capacity(DEFAULT_BUFFER_SIZE),
        }
    }

    /// Reads a complete IMAP response line, handling literals.
    ///
    /// IMAP responses can contain literals in the format `{n}\r\n<n bytes>`.
    /// This method reads the entire response including any embedded literals.
    pub async fn read_response(&mut self) -> Result<Vec<u8>> {
        let mut response = Vec::new();

        loop {
            // Read until CRLF
            let line = self.read_line().await?;

            // Append the line to the response
            response.extend_from_slice(&line);

            // Check for literal at end of line: {123} or {123+}
            if let Some(literal_len) = parse_literal_length(&line) {
                // Validate literal size to prevent DoS via memory exhaustion
                if literal_len > MAX_LITERAL_SIZE {
                    return Err(crate::Error::Protocol(format!(
                        "literal too large: {literal_len} bytes (max {MAX_LITERAL_SIZE})"
                    )));
                }
                // Read the literal data
                let mut literal = vec![0u8; literal_len];
                self.reader.read_exact(&mut literal).await?;
                response.extend_from_slice(&literal);
                // Continue reading (there might be more after the literal)
            } else {
                // No literal, this is the end of the response
                break;
            }
        }

        Ok(response)
    }

    /// Reads a single CRLF-terminated line.
    async fn read_line(&mut self) -> Result<Vec<u8>> {
        let mut line = Vec::new();

        loop {
            let buf = self.reader.fill_buf().await?;
            if buf.is_empty() {
                return Err(crate::Error::Io(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "connection closed",
                )));
            }

            // Look for CRLF
            if let Some(pos) = find_crlf(buf) {
                line.extend_from_slice(&buf[..pos + 2]);
                self.reader.consume(pos + 2);
                break;
            }

            // No CRLF found, consume all and continue
            let len = buf.len();
            line.extend_from_slice(buf);
            self.reader.consume(len);

            // Check for maximum line length
            if line.len() > MAX_LINE_LENGTH {
                return Err(crate::Error::Protocol("line too long".to_string()));
            }
        }

        Ok(line)
    }

    /// Writes a command to the stream.
    pub async fn write_command(&mut self, data: &[u8]) -> Result<()> {
        self.write_buffer.clear();
        self.write_buffer.extend_from_slice(data);

        let stream = self.reader.get_mut();
        stream.write_all(&self.write_buffer).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Writes raw data to the stream (for literals).
    pub async fn write_raw(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.reader.get_mut();
        stream.write_all(data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Gets a reference to the underlying stream.
    pub fn get_ref(&self) -> &S {
        self.reader.get_ref()
    }

    /// Gets a mutable reference to the underlying stream.
    pub fn get_mut(&mut self) -> &mut S {
        self.reader.get_mut()
    }

    /// Consumes the framed stream and returns the inner stream.
    ///
    /// Note: Any buffered data will be lost.
    pub fn into_inner(self) -> S {
        self.reader.into_inner()
    }
}

/// Finds the position of CRLF in a buffer.
fn find_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\r\n")
}

/// Parses a literal length from the end of a line.
///
/// Matches patterns like `{123}\r\n` or `{123+}\r\n` (non-synchronizing).
fn parse_literal_length(line: &[u8]) -> Option<usize> {
    // Line must end with CRLF
    if !line.ends_with(b"\r\n") {
        return None;
    }

    let line = &line[..line.len() - 2]; // Strip CRLF

    // Find the opening brace
    let open = line.iter().rposition(|&b| b == b'{')?;

    // Must end with }
    if !line.ends_with(b"}") && !line.ends_with(b"+}") {
        return None;
    }

    // Extract the number
    let num_start = open + 1;
    let num_end = if line.ends_with(b"+}") {
        line.len() - 2
    } else {
        line.len() - 1
    };

    let num_str = std::str::from_utf8(&line[num_start..num_end]).ok()?;
    num_str.parse().ok()
}

/// A response reader that accumulates responses until a tagged response.
pub struct ResponseAccumulator {
    tag: String,
    responses: Vec<Vec<u8>>,
}

impl ResponseAccumulator {
    /// Creates a new response accumulator for the given tag.
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            responses: Vec::new(),
        }
    }

    /// Reads responses until a tagged response matching our tag is found.
    pub async fn read_until_tagged<S>(
        &mut self,
        framed: &mut FramedStream<S>,
    ) -> Result<Vec<Vec<u8>>>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        loop {
            let response = framed.read_response().await?;

            // Check if this is a tagged response matching our tag
            let is_tagged = response
                .get(..self.tag.len())
                .is_some_and(|prefix| prefix == self.tag.as_bytes())
                && response.get(self.tag.len()).is_some_and(|&b| b == b' ');

            self.responses.push(response);

            if is_tagged {
                break;
            }
        }

        Ok(std::mem::take(&mut self.responses))
    }

    /// Returns the collected responses.
    #[must_use]
    pub fn responses(&self) -> &[Vec<u8>] {
        &self.responses
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_string_new,
    clippy::needless_collect,
    clippy::unreadable_literal,
    clippy::used_underscore_items,
    clippy::similar_names
)]
mod tests {
    use super::*;

    #[test]
    fn test_find_crlf() {
        assert_eq!(find_crlf(b"hello\r\n"), Some(5));
        assert_eq!(find_crlf(b"\r\n"), Some(0));
        assert_eq!(find_crlf(b"no newline"), None);
        assert_eq!(find_crlf(b"just\n"), None);
        assert_eq!(find_crlf(b"just\r"), None);
    }

    #[test]
    fn test_parse_literal_length() {
        assert_eq!(parse_literal_length(b"BODY {123}\r\n"), Some(123));
        assert_eq!(parse_literal_length(b"BODY {123+}\r\n"), Some(123));
        assert_eq!(parse_literal_length(b"{0}\r\n"), Some(0));
        assert_eq!(parse_literal_length(b"{999999}\r\n"), Some(999_999));
        assert_eq!(parse_literal_length(b"no literal\r\n"), None);
        assert_eq!(parse_literal_length(b"incomplete {123"), None);
        assert_eq!(parse_literal_length(b"wrong {abc}\r\n"), None);
    }

    #[tokio::test]
    async fn test_framed_read_simple_line() {
        use tokio_test::io::Builder;

        let mock = Builder::new().read(b"* OK ready\r\n").build();
        let mut framed = FramedStream::new(mock);

        let response = framed.read_response().await.unwrap();
        assert_eq!(response, b"* OK ready\r\n");
    }

    #[tokio::test]
    async fn test_framed_read_with_literal() {
        use tokio_test::io::Builder;

        let mock = Builder::new()
            .read(b"* 1 FETCH (BODY {5}\r\n")
            .read(b"hello)\r\n")
            .build();
        let mut framed = FramedStream::new(mock);

        let response = framed.read_response().await.unwrap();
        assert_eq!(response, b"* 1 FETCH (BODY {5}\r\nhello)\r\n");
    }

    #[tokio::test]
    async fn test_framed_write_command() {
        use tokio_test::io::Builder;

        let mock = Builder::new().write(b"A001 LOGIN user pass\r\n").build();
        let mut framed = FramedStream::new(mock);

        framed
            .write_command(b"A001 LOGIN user pass\r\n")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_response_accumulator() {
        use tokio_test::io::Builder;

        let mock = Builder::new()
            .read(b"* CAPABILITY IMAP4rev2\r\n")
            .read(b"* OK IMAP ready\r\n")
            .read(b"A001 OK Success\r\n")
            .build();

        let mut framed = FramedStream::new(mock);
        let mut accumulator = ResponseAccumulator::new("A001");

        let responses = accumulator.read_until_tagged(&mut framed).await.unwrap();

        assert_eq!(responses.len(), 3);
        assert_eq!(responses[0], b"* CAPABILITY IMAP4rev2\r\n");
        assert_eq!(responses[1], b"* OK IMAP ready\r\n");
        assert_eq!(responses[2], b"A001 OK Success\r\n");
    }

    #[tokio::test]
    async fn test_literal_size_validation() {
        use tokio_test::io::Builder;

        // Test that excessively large literals are rejected
        let literal_size = MAX_LITERAL_SIZE + 1;
        let header = format!("* 1 FETCH (BODY {{{literal_size}}}\r\n");

        let mock = Builder::new().read(header.as_bytes()).build();
        let mut framed = FramedStream::new(mock);

        let result = framed.read_response().await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("literal too large")
        );
    }

    #[tokio::test]
    async fn test_literal_max_size_allowed() {
        use tokio_test::io::Builder;

        // Test that the maximum allowed literal size works
        let literal_size = 1000; // Small literal for testing
        let header = format!("* 1 FETCH (BODY {{{literal_size}}}\r\n");
        let literal_data = vec![b'X'; literal_size];
        let trailer = b")\r\n";

        let mock = Builder::new()
            .read(header.as_bytes())
            .read(&literal_data)
            .read(trailer)
            .build();
        let mut framed = FramedStream::new(mock);

        let result = framed.read_response().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_line_length_limit() {
        use tokio_test::io::Builder;

        // Create a line longer than MAX_LINE_LENGTH
        let long_line = "A".repeat(MAX_LINE_LENGTH + 100);
        let mock = Builder::new().read(long_line.as_bytes()).build();
        let mut framed = FramedStream::new(mock);

        let result = framed.read_response().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("line too long"));
    }
}
