# mailledger-mime

MIME message parsing and generation library for email.

## Features

- **Message parsing**: Parse MIME messages with multipart support
- **Message generation**: Build MIME messages with attachments
- **Encoding/Decoding**: Base64, Quoted-Printable, RFC 2047 header encoding
- **Content types**: Full MIME content type support
- **Multipart**: Mixed, alternative, related message types

## Quick Start

### Parsing MIME Messages

```rust
use mailledger_mime::Message;

let raw_message = "From: sender@example.com\r\n\
                   To: recipient@example.com\r\n\
                   Subject: Test\r\n\
                   Content-Type: text/plain\r\n\
                   \r\n\
                   Hello, World!";

let message = Message::parse(raw_message)?;
println!("Subject: {}", message.subject().unwrap_or("(no subject)"));
println!("Body: {}", message.body_text()?);
```

### Encoding/Decoding

```rust
use mailledger_mime::encoding::{encode_base64, decode_base64};

// Base64
let encoded = encode_base64(b"Hello, World!");
let decoded = decode_base64(&encoded)?;

// Quoted-Printable
use mailledger_mime::encoding::encode_quoted_printable;
let encoded = encode_quoted_printable("Héllo, Wørld!");
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
