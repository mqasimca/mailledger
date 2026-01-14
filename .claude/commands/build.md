# /build - Build Project

Build the project with options:

## Debug build (default)
```bash
cargo build --workspace
```

## Release build
```bash
cargo build --workspace --release
```

## Check only (faster, no codegen)
```bash
cargo check --workspace
```

Report build status, any errors or warnings, and binary sizes for release builds.
