# Testing Patterns

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_response() {
        let input = b"* OK Server ready\r\n";
        let result = parse_response(input);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Response::Greeting);
    }

    #[test]
    fn parse_invalid_response() {
        let input = b"garbage";
        let result = parse_response(input);

        assert!(matches!(result, Err(Error::Parse { .. })));
    }
}
```

## Async Tests

```rust
#[tokio::test]
async fn connect_to_server() {
    let client = ImapClient::connect("localhost", 993).await;
    assert!(client.is_ok());
}

#[tokio::test]
async fn login_with_valid_credentials() {
    let client = ImapClient::connect("localhost", 993).await.unwrap();
    let session = client.login("user", "pass").await;
    assert!(session.is_ok());
}
```

## Property-Based Tests (proptest)

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn parser_never_panics(data: Vec<u8>) {
        // Should not panic on any input
        let _ = parse_response(&data);
    }

    #[test]
    fn roundtrip_command(cmd in any_command()) {
        let serialized = cmd.serialize();
        let parsed = Command::parse(&serialized).unwrap();
        prop_assert_eq!(cmd, parsed);
    }
}

fn any_command() -> impl Strategy<Value = Command> {
    prop_oneof![
        Just(Command::Noop),
        Just(Command::Capability),
        any::<String>().prop_map(|s| Command::Login {
            user: s.clone(),
            pass: s
        }),
    ]
}
```

## Test Fixtures

```rust
#[cfg(test)]
mod tests {
    fn sample_envelope() -> Envelope {
        Envelope {
            subject: Some("Test".into()),
            from: vec![Address::new("test@example.com")],
            ..Default::default()
        }
    }

    #[test]
    fn envelope_display() {
        let env = sample_envelope();
        assert!(env.to_string().contains("Test"));
    }
}
```

## Integration Tests

Place in `tests/` directory:

```rust
// tests/imap_integration.rs
use mailledger_imap::ImapClient;

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn full_imap_flow() {
    let client = ImapClient::connect("imap.test.com", 993)
        .await
        .expect("connect");

    let session = client.login("test", "pass")
        .await
        .expect("login");

    let folders = session.list("", "*")
        .await
        .expect("list");

    assert!(folders.iter().any(|f| f.name == "INBOX"));
}
```
