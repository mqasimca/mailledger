//! # mailledger-mime
//!
//! MIME message parsing and generation library for email.
//!
//! ## Features
//!
//! - **Message parsing**: Parse MIME messages with multipart support
//! - **Message generation**: Build MIME messages with attachments
//! - **Encoding/Decoding**: Base64, Quoted-Printable, RFC 2047 header encoding
//! - **Content types**: Full MIME content type support
//! - **Multipart**: Mixed, alternative, related message types
//!
//! ## Quick Start
//!
//! ### Parsing MIME Messages
//!
//! ```ignore
//! use mailledger_mime::Message;
//!
//! let raw_message = "From: sender@example.com\r\n\
//!                    To: recipient@example.com\r\n\
//!                    Subject: Test\r\n\
//!                    Content-Type: text/plain\r\n\
//!                    \r\n\
//!                    Hello, World!";
//!
//! let message = Message::parse(raw_message)?;
//! println!("Subject: {}", message.subject().unwrap_or("(no subject)"));
//! println!("Body: {}", message.body_text()?);
//! ```
//!
//! ### Building MIME Messages
//!
//! ```ignore
//! use mailledger_mime::{MessageBuilder, ContentType};
//!
//! let message = MessageBuilder::new()
//!     .from("sender@example.com")
//!     .to("recipient@example.com")
//!     .subject("Test Message")
//!     .text_body("Hello, World!")
//!     .build()?;
//!
//! println!("{}", message.to_string());
//! ```
//!
//! ### Working with Attachments
//!
//! ```ignore
//! use mailledger_mime::{MessageBuilder, Attachment};
//!
//! let attachment = Attachment::from_file("document.pdf")?;
//!
//! let message = MessageBuilder::new()
//!     .from("sender@example.com")
//!     .to("recipient@example.com")
//!     .subject("Document")
//!     .text_body("Please find the attached document.")
//!     .attach(attachment)
//!     .build()?;
//! ```
//!
//! ### Multipart Messages
//!
//! ```ignore
//! use mailledger_mime::MessageBuilder;
//!
//! let message = MessageBuilder::new()
//!     .from("sender@example.com")
//!     .to("recipient@example.com")
//!     .subject("Test")
//!     .text_body("Plain text version")
//!     .html_body("<html><body><h1>HTML version</h1></body></html>")
//!     .build()?; // Creates multipart/alternative
//! ```
//!
//! ### Encoding/Decoding
//!
//! ```ignore
//! use mailledger_mime::encoding::{encode_base64, decode_base64, encode_quoted_printable};
//!
//! // Base64
//! let encoded = encode_base64(b"Hello, World!");
//! let decoded = decode_base64(&encoded)?;
//!
//! // Quoted-Printable
//! let encoded = encode_quoted_printable("Héllo, Wørld!");
//!
//! // RFC 2047 header encoding
//! use mailledger_mime::encoding::encode_rfc2047;
//! let encoded = encode_rfc2047("Héllo", "utf-8")?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod content_type;
mod error;
mod header;
mod message;

pub mod encoding;

pub use content_type::ContentType;
pub use error::{Error, Result};
pub use header::Headers;
pub use message::{Message, Part, TransferEncoding};
