# Contributing to Rusty Commit

Thank you for your interest in contributing to Rusty Commit! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [CI/CD Pipeline](#cicd-pipeline)
- [CodeRabbit AI Review](#coderabbit-ai-review)

## Code of Conduct

Please be respectful and constructive in all interactions. We aim to maintain a welcoming and inclusive community.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/rusty-commit.git`
3. Add upstream remote: `git remote add upstream https://github.com/hongkongkiwi/rusty-commit.git`
4. Create a feature branch: `git checkout -b feature/your-feature-name`

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git 2.23+
- cargo-edit (optional): `cargo install cargo-edit`
- cargo-watch (optional): `cargo install cargo-watch`

### Building

```bash
# Standard build
cargo build

# Build with all features
cargo build --all-features

# Release build
cargo build --release

# Build with secure storage
cargo build --features secure-storage
```

### Running

```bash
# Run directly
cargo run -- [arguments]

# Or build and run
./target/debug/oco [arguments]
```

## Pull Request Process

### 1. Before Creating a PR

- Ensure your code follows Rust conventions
- Run `cargo fmt` to format your code
- Run `cargo clippy` to check for common issues
- Add tests for new functionality
- Update documentation as needed

### 2. PR Title Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/). Your PR title should be formatted as:

```
<type>(<scope>): <description>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Test additions or modifications
- `chore`: Maintenance tasks
- `perf`: Performance improvements
- `ci`: CI/CD changes
- `build`: Build system changes

Examples:
- `feat(auth): add OAuth authentication support`
- `fix(config): resolve path issue on Windows`
- `docs: update installation instructions`

### 3. PR Description

Include:
- What changes you've made
- Why you've made them
- Any breaking changes
- Related issues (use `Fixes #123` to auto-close issues)

### 4. Review Process

- PRs are automatically reviewed by CodeRabbit AI
- Human maintainers will review after CI passes
- Address feedback promptly
- Keep PRs focused and reasonably sized

## Coding Standards

### Rust Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting (configuration in `rustfmt.toml`)
- Use `clippy` for linting
- Prefer `Result<T, E>` over panicking
- Document public APIs with doc comments
- Use meaningful variable and function names

### Error Handling

```rust
// Good
fn process_data(input: &str) -> Result<String> {
    let parsed = parse_input(input)?;
    Ok(format!("Processed: {}", parsed))
}

// Avoid
fn process_data(input: &str) -> String {
    let parsed = parse_input(input).unwrap();  // Don't unwrap in production code
    format!("Processed: {}", parsed)
}
```

### Documentation

- Add doc comments to all public items
- Include examples in doc comments where helpful
- Keep comments up-to-date with code changes

```rust
/// Generates a commit message using the specified AI provider.
///
/// # Arguments
///
/// * `diff` - The git diff to generate a message for
/// * `context` - Optional additional context
///
/// # Example
///
/// ```
/// let message = generate_commit_message("diff", None).await?;
/// ```
pub async fn generate_commit_message(diff: &str, context: Option<&str>) -> Result<String> {
    // Implementation
}
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Run doc tests
cargo test --doc
```

### Writing Tests

- Place unit tests in the same file as the code
- Place integration tests in `tests/` directory
- Use descriptive test names
- Test edge cases and error conditions

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_works() {
        let result = my_function("input");
        assert_eq!(result, expected_value);
    }

    #[test]
    fn test_handles_error() {
        let result = my_function("");
        assert!(result.is_err());
    }
}
```

## CI/CD Pipeline

Our CI pipeline runs on every PR and includes:

### Automated Checks

1. **Linting** (`cargo fmt --check`, `cargo clippy`)
2. **Security Audit** (`cargo audit`)
3. **Tests** (multiple OS and Rust versions)
4. **Documentation** (`cargo doc`)
5. **Code Coverage** (via cargo-tarpaulin)
6. **PR Title Validation** (conventional commits)

### Build Matrix

- **Operating Systems**: Ubuntu, macOS, Windows
- **Rust Versions**: stable, beta, nightly (Linux only)
- **Features**: with and without `secure-storage`

## CodeRabbit AI Review

### What is CodeRabbit?

CodeRabbit is an AI-powered code review tool that automatically reviews every PR. It provides:

- Code quality suggestions
- Security vulnerability detection
- Performance optimization tips
- Best practice recommendations
- Documentation improvements

### How It Works

1. **Automatic Trigger**: CodeRabbit runs on every PR automatically
2. **Inline Comments**: Provides specific feedback on code lines
3. **Summary**: Generates a high-level review summary
4. **Interactive**: You can reply to CodeRabbit comments for clarification

### Configuration

CodeRabbit is configured via `.coderabbit.yaml`. Current settings include:

- **Language-specific checks**: Rust best practices, clippy, rustfmt
- **Security scanning**: Dependency vulnerabilities, hardcoded secrets
- **Performance analysis**: Algorithm complexity, common issues
- **Documentation**: Missing docs, outdated comments

### Interacting with CodeRabbit

- **@coderabbitai review** - Request a re-review
- **@coderabbitai ignore** - Ignore a specific suggestion
- **@coderabbitai help** - Get help with CodeRabbit commands

### Best Practices for CodeRabbit Reviews

1. **Address All Comments**: Even if you disagree, explain why
2. **Ask for Clarification**: If a suggestion is unclear, ask CodeRabbit
3. **Learn from Feedback**: CodeRabbit often catches subtle issues
4. **Don't Blindly Accept**: Evaluate suggestions critically

## Common Issues and Solutions

### Build Failures

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for outdated dependencies
cargo outdated
```

### Test Failures

```bash
# Run tests with more verbose output
cargo test -- --test-threads=1 --nocapture

# Run only failing test
cargo test specific_test_name
```

### Clippy Warnings

```bash
# Auto-fix some clippy warnings
cargo clippy --fix

# See all clippy warnings
cargo clippy -- -W clippy::all
```

## Getting Help

- **Issues**: Open an issue for bugs or feature requests
- **Discussions**: Use GitHub Discussions for questions
- **Discord/Slack**: [If applicable, add community links]

## License

By contributing, you agree that your contributions will be licensed under the MIT License.