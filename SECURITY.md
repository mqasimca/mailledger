# Security Policy

## Reporting Security Vulnerabilities

If you discover a security vulnerability in MailLedger, please report it by creating a private security advisory on GitHub or emailing the maintainers directly. Please do not open public issues for security vulnerabilities.

## Security Fixes

This document tracks security vulnerabilities that have been identified and fixed in MailLedger.

### Path Traversal in Attachment Downloads (CWE-22)

**Status**: Fixed
**Severity**: HIGH (CVSS 8.2)
**Affected Component**: GUI application (`mailledger`)
**Fixed in**: Development (unreleased)

**Description**: The attachment download handler did not sanitize filenames received from email messages. A malicious email could include attachments with path traversal sequences (e.g., `../../etc/passwd` or `..\\..\\windows\\system32`) to write files outside the intended downloads directory.

**Fix**: Implemented `sanitize_filename()` function that:
- Removes path separators (`/`, `\`, null bytes)
- Strips leading dots (prevents hidden files)
- Removes control characters
- Limits filename length to 255 characters
- Preserves Unicode and file extensions
- Logs warnings when filenames are sanitized

**Location**: `crates/mailledger/src/main.rs:37-67`

### DoS via Unbounded Literal Size (CWE-400)

**Status**: Fixed
**Severity**: CRITICAL
**Affected Component**: IMAP client (`mailledger-imap`)
**Fixed in**: Development (unreleased)

**Description**: The IMAP parser did not validate the size of literal strings before attempting to read them into memory. A malicious IMAP server could specify an extremely large literal size (e.g., 10GB) causing memory exhaustion.

**Fix**: Added `MAX_LITERAL_SIZE` constant (100MB) and validation in `FramedConnection::read_response()` to reject literals exceeding this limit.

**Location**: `crates/mailledger-imap/src/connection/framed.rs:14-32`

### Silent UID 0 Discard (CWE-393)

**Status**: Fixed
**Severity**: CRITICAL
**Affected Component**: IMAP client (`mailledger-imap`)
**Fixed in**: Development (unreleased)

**Description**: The FETCH response parser silently discarded messages with UID 0, which is not a valid UID according to RFC 3501. This could allow a malicious server to hide messages from the user without any error indication.

**Fix**: Changed parser to return an explicit error when encountering UID 0, making the invalid state visible.

**Location**: `crates/mailledger-imap/src/parser/response/fetch.rs`

### UTF-8 Boundary Corruption in Quoted-Printable (CWE-838)

**Status**: Fixed
**Severity**: HIGH
**Affected Component**: MIME parser (`mailledger-mime`)
**Fixed in**: Development (unreleased)

**Description**: The quoted-printable decoder could truncate UTF-8 sequences when decoding non-ASCII text, causing data corruption or panic when converting to UTF-8.

**Fix**: Modified decoder to handle UTF-8 multibyte sequences correctly and validate UTF-8 integrity after decoding.

**Location**: `crates/mailledger-mime/src/encoding.rs`

### Integer Overflow in Batch Range Calculation (CWE-190)

**Status**: Fixed
**Severity**: HIGH
**Affected Component**: IMAP client (`mailledger-imap`)
**Fixed in**: Development (unreleased)

**Description**: Batch range calculations could overflow when processing large UID ranges, potentially causing incorrect message fetching or panic.

**Fix**: Replaced arithmetic operators with saturating variants (`saturating_sub`, `saturating_add`).

**Location**: `crates/mailledger-imap/src/fetch.rs`

### IMAP Search Injection (CWE-74)

**Status**: Fixed
**Severity**: HIGH
**Affected Component**: Core services (`mailledger-core`)
**Fixed in**: Development (unreleased)

**Description**: The IMAP SEARCH command construction did not properly escape user input, allowing potential injection of IMAP commands.

**Fix**: Improved input sanitization to escape special characters and quotes in search terms.

**Location**: `crates/mailledger-core/src/service/mail.rs`

### Tag Counter Overflow (CWE-190)

**Status**: Fixed
**Severity**: CRITICAL
**Affected Component**: IMAP client (`mailledger-imap`)
**Fixed in**: Development (unreleased)

**Description**: The IMAP tag generator could overflow after 4.2 billion operations, causing tag reuse and protocol confusion.

**Fix**: Added overflow detection with explicit panic when counter reaches maximum value.

**Location**: `crates/mailledger-imap/src/command/tag_generator.rs`

### Buffer Overflow in Literal Size Calculation (CWE-680)

**Status**: Fixed
**Severity**: CRITICAL
**Affected Component**: IMAP client (`mailledger-imap`)
**Fixed in**: Development (unreleased)

**Description**: Integer overflow in literal size calculation could cause buffer overflows when parsing IMAP responses.

**Fix**: Replaced arithmetic with checked operations (`checked_add`, `checked_mul`) that return errors on overflow.

**Location**: `crates/mailledger-imap/src/parser/lexer/mod.rs`

## Security Best Practices

When contributing to MailLedger, please follow these security guidelines:

1. **Input Validation**: Always validate and sanitize user input and data from external sources (IMAP/SMTP servers, email content)
2. **Resource Limits**: Impose reasonable limits on memory allocations and processing time
3. **Error Handling**: Never silently discard errors that could indicate malicious activity
4. **Integer Safety**: Use checked arithmetic operations for security-critical calculations
5. **Path Safety**: Sanitize all filesystem paths derived from external input
6. **Injection Prevention**: Properly escape all strings used in command construction

## Testing

All security fixes include comprehensive test coverage:
- Unit tests for individual functions
- Property-based tests for parsers and validators
- Integration tests for end-to-end scenarios
- Fuzzing (planned) for protocol parsers

Run security tests:
```bash
cargo test --workspace
cargo test -p mailledger-imap parser
cargo test -p mailledger sanitize_filename
```
