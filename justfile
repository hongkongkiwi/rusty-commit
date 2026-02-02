# Rusty Commit - Justfile
# Comprehensive command reference for development, testing, and releasing

# ============================================
# BUILD COMMANDS
# ============================================

# Build debug binary
build:
    cargo build

# Build release binary
build-release:
    cargo build --release

# Build with specific features
build-features FEATURES="":
    #!/usr/bin/env bash
    if [ -z "$FEATURES" ]; then
        cargo build --all-features
    else
        cargo build --features $FEATURES
    fi

# Build for a specific target
build-target TARGET="":
    cargo build --target {{TARGET}}

# Build release for a specific target
build-release-target TARGET="":
    cargo build --release --target {{TARGET}}

# ============================================
# DEVELOPMENT COMMANDS
# ============================================

# Start rusty-commit in stdio mode (for Claude Code etc)
dev:
    cargo run --release -- stdio

# Start with custom config
dev-config CONFIG="":
    cargo run --release -- stdio --config {{CONFIG}}

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cargo watch -x "run --release -- stdio"

# ============================================
# TEST COMMANDS
# ============================================

# Run all tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run specific test
test-name NAME="":
    cargo test {{NAME}}

# Run doc tests
test-doc:
    cargo test --doc

# Run tests with all features
test-all-features:
    cargo test --all-features

# ============================================
# CODE QUALITY
# ============================================

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Format code
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy --all-targets -- -D warnings

# Run clippy with fixes (auto-corrects)
clippy-fix:
    cargo clippy --all-targets --fix --allow-dirty

# Generate documentation
doc:
    cargo doc --no-deps --all-features
    @echo "Documentation generated in target/doc/"

# Check documentation links
doc-check-links:
    cargo doc --no-deps --all-features --document-private-items

# Run all checks (fmt, clippy, doc, test)
check: fmt-check clippy doc test
    @echo "All checks passed!"

# ============================================
# GIT HOOKS (cargo-husky)
# ============================================

# Install git hooks via cargo-husky
hooks-install:
    @if grep -q 'cargo-husky' Cargo.toml 2>/dev/null; then \
        cargo husky install; \
        echo "Git hooks installed"; \
    else \
        echo "cargo-husky not configured. Add to Cargo.toml or install manually:"; \
        echo "  cargo install cargo-husky"; \
        echo "  cargo husky install"; \
    fi

# Run cargo-husky hooks manually
hooks-run:
    @if command -v cargo &> /dev/null; then \
        cargo husky run; \
    else \
        echo "Cargo not found"; \
    fi

# ============================================
# SECURITY
# ============================================

# Run cargo audit for vulnerabilities
audit:
    cargo audit

# Update cargo lock and audit
audit-update:
    cargo update && cargo audit

# ============================================
# RELEASE COMMANDS (using cargo-release)
# ============================================

# Show current version
version:
    @grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'

# Release using cargo-release (recommended)
# Usage: just release-cargo [patch|minor|major]
# This will: update version, update CHANGELOG.md, commit, tag, and push
# Skips publishing (we use GitHub Actions for releases)
release-cargo TYPE="patch":
    #!/usr/bin/env bash
    set -e
    cargo release {{TYPE}} --no-publish --no-verify --no-confirm --execute

# Dry-run release to see what would happen
release-dry-run:
    cargo release patch --no-publish --no-verify --no-confirm

# Show next version (patch, minor, or major)
next-patch:
    @echo "$(($(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\)"/\1*\100+\2*\10+\3/')) + 1)" | bc | \
    awk -F. '{OFS="."; print $$1, $$2, $$3+1}'

next-minor:
    @echo "$(($(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\)"/\1*\100+\2*\10+\3/')) + 10)" | bc | \
    awk -F. '{OFS="."; print $$1, $$2+1, 0}'

next-major:
    @echo "$(($(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\([0-9]*\)\.\([0-9]*\)\.\([0-9]*\)"/\1*\100+\2*\10+\3/')) + 100)" | bc | \
    awk -F. '{OFS="."; print $$1+1, $$2, $$3}'

# Bump version and create git tag
# Usage: just release [patch|minor|major]
release TYPE="patch":
    #!/usr/bin/env bash
    set -e

    TYPE="${TYPE:-patch}"
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

    # Calculate new version
    if [ "$TYPE" = "patch" ]; then
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{OFS="."; $$3++; print}')
    elif [ "$TYPE" = "minor" ]; then
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{OFS="."; $$2++; $$3=0; print}')
    elif [ "$TYPE" = "major" ]; then
        NEW_VERSION=$(echo $CURRENT_VERSION | awk -F. '{OFS="."; $$1++; $$2=0; $$3=0; print}')
    else
        echo "Invalid type: $TYPE. Use patch, minor, or major."
        exit 1
    fi

    echo "Current version: $CURRENT_VERSION"
    echo "New version: $NEW_VERSION"

    # Update version in Cargo.toml (handle both macOS and Linux)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    else
        sed -i "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
    fi

    # Commit version bump
    git add Cargo.toml
    git commit -m "Bump version to v$NEW_VERSION"

    # Create and push tag
    git tag "v$NEW_VERSION"
    git push origin main
    git push origin "v$NEW_VERSION"

    echo ""
    echo "Release v$NEW_VERSION created and pushed!"
    echo "GitHub Actions will now build and publish the release."
    echo "Monitor at: https://github.com/hongkongkiwi/rusty-commit/actions"

# Draft release notes (generates template)
release-notes:
    #!/usr/bin/env bash
    echo "## What's Changed"
    echo ""
    echo "### Added"
    echo ""
    echo "### Changed"
    echo ""
    echo "### Fixed"
    echo ""
    echo "### Removed"
    echo ""
    echo "### Security"
    echo ""
    echo "**Full Changelog**: https://github.com/hongkongkiwi/rusty-commit/compare/$(git describe --tags --abbrev=0 2>/dev/null || echo "previous")...v$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"

# Local dry-run release build (builds all platforms locally)
release-local:
    #!/usr/bin/env bash
    echo "Building release artifacts locally..."
    cargo build --release
    echo ""
    echo "Binary built at: target/release/rusty-commit"
    echo ""
    echo "To create a full release:"
    echo "  1. Update version in Cargo.toml"
    echo "  2. Commit changes"
    echo "  3. Run: git tag v<version>"
    echo "  4. Run: git push origin main --tags"
    echo ""
    echo "GitHub Actions will automatically build and publish."

# ============================================
# CLEANUP COMMANDS
# ============================================

# Clean build artifacts
clean:
    cargo clean

# Clean everything including target
clean-all:
    cargo clean
    rm -rf target/

# Remove lock file and clean
clean-lock:
    rm -f Cargo.lock
    cargo clean

# ============================================
# INSTALL COMMANDS
# ============================================

# Install from source
install:
    cargo install --path .

# Install specific version from GitHub
install-github VERSION="":
    #!/usr/bin/env bash
    if [ -z "$VERSION" ]; then
        VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    fi
    echo "Installing rusty-commit v$VERSION from source..."
    cargo install --git https://github.com/hongkongkiwi/rusty-commit --tag "v$VERSION" --locked

# ============================================
# DEPENDENCY COMMANDS
# ============================================

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Tree of dependencies
deps-tree:
    cargo tree

# Show why a dependency is used
deps-why DEP="":
    cargo why {{DEP}}

# ============================================
# HELP
# ============================================

# Show all available commands
help:
    @echo "Rusty Commit - Available Commands"
    @echo ""
    @echo "Build:"
    @echo "  build           - Build debug binary"
    @echo "  build-release   - Build release binary"
    @echo "  build-features  - Build with specific features"
    @echo "  build-target    - Build for specific target"
    @echo ""
    @echo "Development:"
    @echo "  dev             - Start MCP server in stdio mode"
    @echo "  dev-config      - Start with custom config"
    @echo "  watch           - Watch and rebuild (requires cargo-watch)"
    @echo ""
    @echo "Testing:"
    @echo "  test            - Run all tests"
    @echo "  test-verbose    - Run tests with output"
    @echo "  test-name       - Run specific test"
    @echo "  test-doc        - Run doc tests"
    @echo ""
    @echo "Code Quality:"
    @echo "  fmt             - Format code"
    @echo "  fmt-check       - Check formatting"
    @echo "  clippy          - Run clippy lints"
    @echo "  clippy-fix      - Auto-fix clippy issues"
    @echo "  doc             - Generate documentation"
    @echo "  check           - Run all checks"
    @echo ""
    @echo "Security:"
    @echo "  audit           - Check for vulnerabilities"
    @echo "  audit-update    - Update and audit"
    @echo ""
    @echo "Release:"
    @echo "  version         - Show current version"
    @echo "  next-patch      - Show next patch version"
    @echo "  next-minor      - Show next minor version"
    @echo "  next-major      - Show next major version"
    @echo "  release-cargo   - Release using cargo-release (recommended)"
    @echo "  release-dry-run - Dry-run cargo-release"
    @echo "  release [type]  - Legacy manual release"
    @echo "  release-notes   - Generate release notes template"
    @echo "  release-local   - Local dry-run build"
    @echo ""
    @echo "Cleanup:"
    @echo "  clean           - Clean build artifacts"
    @echo "  clean-all       - Clean everything"
    @echo "  clean-lock      - Remove lock file and clean"
    @echo ""
    @echo "Install:"
    @echo "  install         - Install from source"
    @echo "  install-github  - Install from GitHub"
    @echo ""
    @echo "Dependencies:"
    @echo "  update          - Update dependencies"
    @echo "  outdated        - Show outdated dependencies"
    @echo "  deps-tree       - Tree of dependencies"
    @echo ""
    @echo "  help            - Show this help"
