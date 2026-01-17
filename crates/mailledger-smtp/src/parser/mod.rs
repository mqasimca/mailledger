//! SMTP response parser.

use crate::error::{Error, Result};
use crate::types::{Reply, ReplyCode};

/// Parses an SMTP reply from response lines.
///
/// SMTP replies can be single-line or multi-line:
/// - Single: `250 OK\r\n`
/// - Multi: `250-First line\r\n250-Second line\r\n250 Last line\r\n`
///
/// # Errors
///
/// Returns an error if the reply is malformed.
pub fn parse_reply(lines: &[String]) -> Result<Reply> {
    if lines.is_empty() {
        return Err(Error::Protocol("Empty reply".into()));
    }

    // Parse code from first line
    let first = &lines[0];
    if first.len() < 3 {
        return Err(Error::Protocol(format!("Reply too short: {first}")));
    }

    let code_str = &first[0..3];
    let code = code_str
        .parse::<u16>()
        .map_err(|_| Error::Protocol(format!("Invalid reply code: {code_str}")))?;

    let reply_code = ReplyCode::new(code);

    // Extract message from all lines
    let mut message = Vec::new();
    for line in lines {
        if line.len() > 4 {
            // Skip code and separator (e.g., "250-" or "250 ")
            message.push(line[4..].to_string());
        } else if line.len() == 3 {
            // Just code, no message
            message.push(String::new());
        } else {
            return Err(Error::Protocol(format!("Malformed reply line: {line}")));
        }
    }

    Ok(Reply::new(reply_code, message))
}

/// Checks if a line is the last line of a multi-line reply.
///
/// Multi-line replies use `-` separator for continuation and ` ` for the last line.
#[must_use]
pub fn is_last_reply_line(line: &str) -> bool {
    line.len() >= 4 && line.as_bytes()[3] == b' '
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::redundant_clone, clippy::manual_string_new, clippy::needless_collect, clippy::unreadable_literal, clippy::used_underscore_items, clippy::similar_names)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_line_reply() {
        let lines = vec!["250 OK".to_string()];
        let reply = parse_reply(&lines).unwrap();
        assert_eq!(reply.code.as_u16(), 250);
        assert_eq!(reply.message, vec!["OK"]);
        assert!(reply.is_success());
    }

    #[test]
    fn test_parse_multi_line_reply() {
        let lines = vec![
            "250-First line".to_string(),
            "250-Second line".to_string(),
            "250 Last line".to_string(),
        ];
        let reply = parse_reply(&lines).unwrap();
        assert_eq!(reply.code.as_u16(), 250);
        assert_eq!(
            reply.message,
            vec!["First line", "Second line", "Last line"]
        );
    }

    #[test]
    fn test_parse_greeting() {
        let lines = vec!["220 smtp.example.com ESMTP ready".to_string()];
        let reply = parse_reply(&lines).unwrap();
        assert_eq!(reply.code.as_u16(), 220);
        assert_eq!(reply.message, vec!["smtp.example.com ESMTP ready"]);
    }

    #[test]
    fn test_is_last_reply_line() {
        assert!(is_last_reply_line("250 OK"));
        assert!(!is_last_reply_line("250-Continuing"));
        assert!(!is_last_reply_line("250"));
    }

    #[test]
    fn test_parse_error_empty() {
        assert!(parse_reply(&[]).is_err());
    }

    #[test]
    fn test_parse_error_too_short() {
        let lines = vec!["25".to_string()];
        assert!(parse_reply(&lines).is_err());
    }

    #[test]
    fn test_parse_error_invalid_code() {
        let lines = vec!["ABC OK".to_string()];
        assert!(parse_reply(&lines).is_err());
    }
}
