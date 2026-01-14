# /test - Run Tests

Run the test suite with various options:

## Quick test (default)
```bash
cargo test --workspace
```

## Verbose output
```bash
cargo test --workspace -- --nocapture
```

## Run specific test
If a test name is provided, run only that test:
```bash
cargo test --workspace <test_name>
```

## With coverage (if cargo-tarpaulin is installed)
```bash
cargo tarpaulin --workspace --out Html
```

Report test results including any failures with their error messages.
