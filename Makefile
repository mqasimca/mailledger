# MailLedger Makefile
# Common development tasks

.PHONY: all build release test check clippy fmt clean run doc help

# Default target
all: check

# Build in debug mode
build:
	cargo build --workspace

# Build in release mode
release:
	cargo build --workspace --release

# Run all tests
test:
	cargo test --workspace

# Run all checks (fmt, clippy, test)
check: fmt-check clippy test

# Run clippy linter
clippy:
	cargo clippy --workspace

# Format code
fmt:
	cargo fmt --all

# Check formatting without modifying
fmt-check:
	cargo fmt --all -- --check

# Clean build artifacts
clean:
	cargo clean

# Run the application
run:
	RUST_LOG=info cargo run -p mailledger

# Run with debug logging
run-debug:
	RUST_LOG=debug cargo run -p mailledger

# Generate documentation
doc:
	cargo doc --workspace --no-deps --open

# Watch for changes and run tests
watch-test:
	cargo watch -x 'test --workspace'

# Watch for changes and check
watch-check:
	cargo watch -x 'clippy --workspace'

# Install development dependencies
setup:
	@echo "Installing cargo-watch..."
	cargo install cargo-watch || true
	@echo "Setup complete!"

# Show help
help:
	@echo "MailLedger Development Tasks"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build       Build in debug mode"
	@echo "  release     Build in release mode"
	@echo "  test        Run all tests"
	@echo "  check       Run fmt-check, clippy, and test"
	@echo "  clippy      Run clippy linter"
	@echo "  fmt         Format code"
	@echo "  fmt-check   Check formatting without modifying"
	@echo "  clean       Clean build artifacts"
	@echo "  run         Run the application (info logging)"
	@echo "  run-debug   Run with debug logging"
	@echo "  doc         Generate and open documentation"
	@echo "  watch-test  Watch for changes and run tests"
	@echo "  watch-check Watch for changes and run clippy"
	@echo "  setup       Install development dependencies"
	@echo "  help        Show this help message"
