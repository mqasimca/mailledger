# /fix - Auto-fix Code Issues

Automatically fix common code issues:

1. Format all code with rustfmt:
```bash
cargo fmt --all
```

2. Apply clippy suggestions where possible:
```bash
cargo clippy --workspace --fix --allow-dirty
```

3. Re-run checks to verify:
```bash
cargo clippy --workspace -- -D warnings
```

Report what was fixed and any remaining issues that need manual attention.
