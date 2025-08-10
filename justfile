# List available commands
default:
    @just --list

# Run tests
test:
    cargo test --all-features

# Run tests with output
test-verbose:
    cargo test --all-features -- --nocapture

# Run clippy linter
lint:
    cargo clippy --all-features -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

# Build debug binary
build:
    cargo build --all-features

# Build release binary
build-release:
    cargo build --release --all-features

# Run debug binary
run *ARGS:
    cargo run -- {{ARGS}}

# Run release binary
run-release *ARGS:
    cargo run --release -- {{ARGS}}

# Install locally
install:
    cargo install --path .

# Install with secure storage
install-secure:
    cargo install --path . --features secure-storage

# Clean build artifacts
clean:
    cargo clean

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Security audit
audit:
    cargo audit

# Generate documentation
docs:
    cargo doc --no-deps --all-features --open

# Run benchmarks (if any)
bench:
    cargo bench

# Release patch version (1.0.0 -> 1.0.1)
release-patch:
    ./release.sh patch

# Release minor version (1.0.0 -> 1.1.0)
release-minor:
    ./release.sh minor

# Release major version (1.0.0 -> 2.0.0)
release-major:
    ./release.sh major

# Release specific version
release VERSION:
    ./release.sh {{VERSION}}

# Dry run of publish to crates.io
publish-dry:
    cargo publish --dry-run

# Publish to crates.io (requires authentication)
publish:
    cargo publish

# Run pre-commit hooks manually
pre-commit:
    cargo fmt --all
    cargo clippy --all-features -- -D warnings

# Setup development environment
setup:
    rustup component add rustfmt clippy
    cargo install cargo-outdated cargo-audit
    pre-commit install

# Run CI locally (mimics GitHub Actions)
ci: check test
    @echo "✅ All CI checks passed!"

# Watch for changes and run tests
watch:
    cargo watch -x test

# Watch for changes and run checks
watch-check:
    cargo watch -x check -x test

# Show project stats
stats:
    @echo "Lines of code:"
    @tokei src tests --exclude target
    @echo "\nDependencies:"
    @cargo tree --depth 1 | wc -l | xargs echo "Direct:"
    @cargo tree | wc -l | xargs echo "Total:"

# Test a specific provider interactively
test-provider PROVIDER:
    RCO_AI_PROVIDER={{PROVIDER}} cargo run -- config setup

# Commit using rco (dogfooding)
commit:
    cargo build --release
    ./target/release/rco commit

# Run with debug logging
debug *ARGS:
    RUST_LOG=debug cargo run -- {{ARGS}}

# Run with trace logging
trace *ARGS:
    RUST_LOG=trace cargo run -- {{ARGS}}
