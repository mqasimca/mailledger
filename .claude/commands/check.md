# /check - Run All Quality Checks

Run the full Rust quality check suite:

```bash
cargo fmt --all --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

Report any issues found and suggest fixes.
