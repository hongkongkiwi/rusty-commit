# AGENTS.md - Rusty Commit (rco) - AI Agent Reference

**Purpose**: This document serves as the comprehensive reference for AI agents (Claude, Gemini, Qwen, etc.) working on the Rusty Commit codebase. It consolidates project knowledge, architecture, coding standards, and workflows.

**Project**: Rusty Commit (`rco`) - AI-powered commit message generator written in Rust
**Repository**: https://github.com/hongkongkiwi/rusty-commit
**Language**: Rust (Edition 2021)
**License**: MIT

---

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Binary Name** | `rco` (also `rusty-commit`) |
| **Primary Purpose** | Generate conventional commits and GitMoji messages using 16+ AI providers |
| **Key Value Props** | Fast (native Rust), local-first options (Ollama), secure keychain storage, MCP server for editors |
| **Code Size** | ~9,363 lines across 54 files |
| **Architecture** | Trait-based provider system, async/await with tokio, modular CLI with clap |

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Directory Structure](#directory-structure)
4. [Key Technologies](#key-technologies)
5. [Provider System](#provider-system)
6. [Authentication & Security](#authentication--security)
7. [Configuration Management](#configuration-management)
8. [Coding Standards](#coding-standards)
9. [Testing](#testing)
10. [Build & Development](#build--development)
11. [CI/CD Pipeline](#cicd-pipeline)
12. [Common Tasks](#common-tasks)
13. [Troubleshooting](#troubleshooting)

---

## Project Overview

### What is Rusty Commit?

A blazing-fast, AI-powered commit message generator that:
- Analyzes git diffs to generate high-quality commit messages
- Supports 16+ AI providers (OpenAI, Anthropic/Claude, OpenRouter, Groq, DeepSeek, GitHub Copilot, Ollama, Fireworks, Moonshot/Kimi, Alibaba DashScope/Qwen, and more)
- Generates conventional commits and GitMoji-formatted messages
- Provides secure credential storage with optional keychain integration
- Offers MCP (Model Context Protocol) server for editor integrations
- Includes Git hooks and GitHub Actions integration

### Core Features

1. **AI-Powered Commit Generation**: Analyzes git diffs and generates conventional commit messages
2. **Multiple Commit Formats**: Conventional commits, GitMoji (with full specification support)
3. **Multi-Language Support**: Can generate commit messages in various languages
4. **Interactive Mode**: Select files to stage, choose between generated variants, edit before committing
5. **Clipboard Mode**: Copy generated messages to clipboard instead of committing
6. **Git Hooks Integration**: Automatic commit message generation via prepare-commit-msg hook
7. **OAuth Authentication**: Secure OAuth flow for Claude, OpenAI Codex, GitHub Copilot, GitLab, Vercel, Codex
8. **Multi-Account Support**: Manage multiple AI provider accounts
9. **MCP Server**: STDIO-based MCP server for editor integration
10. **PR Description Generation**: Generate pull request descriptions using AI

### Advanced Features

- **Token Management**: Automatic token counting and diff chunking for large changes
- **Custom Hooks**: Pre-gen, pre-commit, and post-commit hooks with timeout control
- **File Exclusion**: .rcoignore support to exclude files from AI analysis
- **Commitlint Integration**: Automatic loading and application of commitlint rules
- **Model Selection**: Interactive model selection and listing
- **Update System**: Self-updating via multiple installation methods

---

## Architecture

### Design Patterns

1. **Trait-Based Provider System**: `AIProvider` trait allows easy addition of new providers
2. **Async/Await**: Full async implementation with tokio
3. **Error Handling**: Comprehensive error handling with anyhow and thiserror
4. **Configuration Layering**: Global -> per-repo -> environment variable priority
5. **Feature Flags**: Conditional compilation for different providers

### Key Components

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│                    (clap - cli.rs)                           │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                      Command Layer                           │
│         (commands/ - commit, auth, config, mcp, etc.)       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                   Provider Layer                             │
│              (providers/ - AIProvider trait)                 │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                   Utility Layer                              │
│         (utils/ - token, hooks, retry, git operations)      │
└─────────────────────────────────────────────────────────────┘
```

---

## Directory Structure

```
rusty-commit/
├── src/
│   ├── main.rs                    # Entry point, command routing
│   ├── lib.rs                     # Library exports
│   ├── cli.rs                     # CLI argument parsing with clap
│   ├── config.rs                  # Configuration management (600+ lines)
│   ├── git.rs                     # Git operations (git2-rs wrapper)
│   ├── update.rs                  # Self-update mechanism
│   ├── auth/                      # Authentication system
│   │   ├── mod.rs                 # Multi-account auth support
│   │   ├── oauth.rs               # OAuth client implementation
│   │   ├── token_storage.rs       # Secure token storage
│   │   ├── gitlab_oauth.rs        # GitLab OAuth
│   │   ├── vercel_oauth.rs        # Vercel OAuth
│   │   └── codex_oauth.rs         # OpenAI Codex OAuth
│   ├── commands/                  # CLI command implementations
│   │   ├── commit.rs              # Main commit generation (600+ lines)
│   │   ├── auth.rs                # Authentication commands
│   │   ├── config.rs              # Configuration commands
│   │   ├── mcp.rs                 # MCP server implementation
│   │   ├── githook.rs             # Git hooks
│   │   ├── pr.rs                  # PR generation
│   │   └── model.rs               # Model selection
│   ├── providers/                 # AI provider implementations
│   │   ├── mod.rs                 # Provider trait and factory
│   │   ├── openai.rs              # OpenAI-compatible providers
│   │   ├── anthropic.rs           # Anthropic/Claude
│   │   ├── ollama.rs              # Local Ollama
│   │   ├── gemini.rs              # Google Gemini
│   │   ├── azure.rs               # Azure OpenAI
│   │   ├── perplexity.rs          # Perplexity AI
│   │   └── xai.rs                 # XAI/Grok
│   ├── config/                    # Configuration subsystems
│   │   ├── format.rs              # Config file formats
│   │   ├── secure_storage.rs      # Keychain integration
│   │   ├── accounts.rs            # Multi-account management
│   │   └── migrations.rs          # Config migrations
│   └── utils/                     # Utilities
│       ├── token.rs               # Token counting (tiktoken)
│       ├── hooks.rs               # Hook execution
│       ├── retry.rs               # Retry logic with backoff
│       └── version.rs             # Version checking
├── tests/                         # Integration tests (10 test files)
├── docs/                          # Documentation
│   ├── VERIFICATION.md            # Release verification
│   ├── SIGNING.md                 # Signing procedures
│   └── SECURITY.md                # Security guidelines
├── .github/workflows/             # CI/CD
│   ├── ci.yml                     # Continuous integration
│   └── release.yml                # Release automation
├── justfile                       # Development commands (Just)
├── action.yml                     # GitHub Action configuration
└── install.sh                     # Universal installation script
```

---

## Key Technologies

### Core Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| **tokio** | 1.35 | Async runtime |
| **clap** | 4.5 | CLI argument parsing |
| **git2** | 0.20 | Git operations (libgit2 bindings) |
| **reqwest** | 0.12 | HTTP client |
| **serde** | 1.0 | Serialization |
| **async-openai** | 0.29 | OpenAI SDK |
| **keyring** | 3.6 | System keychain integration (optional) |
| **dialoguer** | 0.11 | Interactive prompts |
| **rmcp** | 0.11.0 | MCP protocol SDK |

### Build Features

- **Default Features**: openai, anthropic, ollama, gemini, azure, perplexity, xai
- **Optional Features**: secure-storage (keychain integration), docs

### Platform Support

- **Linux**: x86_64, aarch64, armv7, riscv64 (gnu/musl)
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Windows**: x86_64, i686

---

## Provider System

### AIProvider Trait

All providers implement the `AIProvider` trait defined in `src/providers/mod.rs`:

```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
    ) -> Result<String>;
}
```

### Supported Providers

| Provider | Config Value | Model Example | Notes |
|----------|--------------|---------------|-------|
| Anthropic | `anthropic` | `claude-3-5-haiku-20241022` | OAuth supported |
| OpenAI | `openai` | `gpt-4o-mini` | OpenAI-compatible |
| OpenRouter | `openrouter` | `openai/gpt-4o-mini` | Aggregator |
| Groq | `groq` | `llama-3.1-70b-versatile` | Fast inference |
| DeepSeek | `deepseek` | `deepseek-chat` | |
| GitHub Copilot | `github-copilot` | `gpt-4o` | OAuth supported |
| Ollama | `ollama` | `mistral` | Local, custom URL |
| Gemini | `gemini` | `gemini-pro` | Google |
| Azure | `azure` | `<deployment-name>` | Azure OpenAI |
| Perplexity | `perplexity` | `llama-3.1-sonar-small-128k-online` | |
| XAI | `xai` | `grok-beta` | |
| Fireworks | `fireworks` | `accounts/fireworks/models/...` | |
| Moonshot | `moonshot` | `kimi-k2` | Kimi |
| DashScope | `dashscope` | `qwen3-coder-32b-instruct` | Alibaba Qwen |
| Together | `together` | `meta-llama/Meta-Llama-3.1-70B...` | |
| DeepInfra | `deepinfra` | `meta-llama/Meta-Llama-3-70B...` | |
| Mistral | `mistral` | `mistral-small-latest` | |

### Adding a New Provider

1. Create a new file in `src/providers/` (e.g., `newprovider.rs`)
2. Implement the `AIProvider` trait
3. Add the provider to the factory function in `src/providers/mod.rs`
4. Add the provider name to `src/cli.rs` if needed
5. Add documentation to README.md
6. Add tests in `tests/` directory

---

## Authentication & Security

### Authentication Methods

1. **API Key**: Direct configuration via `RCO_API_KEY`
2. **OAuth 2.0 with PKCE**: Secure OAuth flow for Claude, OpenAI Codex, GitHub Copilot, GitLab, Vercel
3. **Multi-Account Support**: Manage multiple accounts per provider

### Secure Storage (optional feature)

When `secure-storage` feature is enabled:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring / KWallet / KeePassXC)
- **Windows**: Credential Manager
- Automatic fallback to config file if unavailable

### OAuth Flow

1. `rco auth login` - Initiates OAuth flow
2. Local HTTP server listens on callback port
3. System browser opens for user authentication
4. Provider redirects to localhost with auth code
5. Token is exchanged and stored securely

### Token Storage

- Tokens stored in `~/.config/rustycommit/` by default
- Secure storage via system keychain when available
- Multi-account support in `~/.config/rustycommit/accounts.toml`

---

## Configuration Management

### Configuration Locations

1. **Global**: `~/.config/rustycommit/config.{toml,json}`
2. **Per-repo**: `.rustycommit.toml` or `.rco.toml`
3. **Environment Variables**: `RCO_*` prefix

### Priority Order

Per-repo config > Global config > Environment variables > Defaults

### Common Configuration Keys

| Key | What it does | Example |
|---|---|---|
| `RCO_AI_PROVIDER` | Which AI backend to use | `anthropic`, `openai`, `ollama` |
| `RCO_MODEL` | Model name for the provider | `claude-3-5-haiku-20241022`, `gpt-4o-mini` |
| `RCO_API_KEY` | API key if required | `sk-...`, `gsk_...` |
| `RCO_API_URL` | Custom endpoint | `http://localhost:11434` |
| `RCO_COMMIT_TYPE` | Commit format | `conventional` or `gitmoji` |
| `RCO_EMOJI` | Emojis in messages | `true` / `false` |
| `RCO_LANGUAGE` | Output language | `en`, `es`, `fr` |

### Configuration Commands

```bash
rco config status                          # secure storage status
rco config set RCO_AI_PROVIDER=anthropic
rco config get RCO_AI_PROVIDER
rco config reset --all
```

---

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

### Commit Message Convention

Follow Conventional Commits:

```
<type>(<scope>): <description>

[optional body]

[optional footer]
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

---

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

---

## Build & Development

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

### Just Commands

The project uses `just` as a command runner. Common commands:

```bash
just build              # Build the project
just test               # Run tests
just fmt                # Format code
just clippy             # Run clippy
just all                # Run fmt, clippy, and test
```

---

## CI/CD Pipeline

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

---

## Common Tasks

### Adding a New AI Provider

1. Create implementation in `src/providers/newprovider.rs`
2. Implement `AIProvider` trait
3. Add to provider factory in `src/providers/mod.rs`
4. Update CLI arguments if needed
5. Add tests in `tests/`
6. Update README.md with provider documentation

### Adding a New Configuration Option

1. Add config key to `src/config.rs`
2. Add CLI argument to `src/cli.rs`
3. Add tests for new option
4. Update README.md

### Adding a New Command

1. Create command file in `src/commands/newcommand.rs`
2. Add command to CLI in `src/cli.rs`
3. Add command routing in `src/main.rs`
4. Add tests in `tests/`

### Debugging OAuth Flow

1. Check local server logs for callback
2. Verify browser redirects to correct port
3. Check token storage in `~/.config/rustycommit/`
4. Enable debug logging: `RUST_LOG=debug rco auth login`

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| 401 / Invalid API key | Re-authenticate (`rco auth login`) or set valid `RCO_API_KEY` |
| Rate-limited (429) | Wait briefly; try lighter model or another provider |
| Secure storage unavailable | Falls back to file storage; check `rco config status` |
| Hooks not running | Ensure `.git/hooks/prepare-commit-msg` exists and is executable |
| Windows PATH issues | Add install dir (e.g., `%USERPROFILE%\.cargo\bin`) to PATH |
| Corporate proxy | Set `HTTP_PROXY`/`HTTPS_PROXY` environment variables |

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

---

## Additional Resources

### Documentation Files

- `README.md` - User guide with installation, configuration, examples
- `README_OAUTH.md` - OAuth authentication flow documentation
- `CONTRIBUTING.md` - Development setup, coding standards, PR process
- `INSTALL.md` - Installation instructions
- `docs/VERIFICATION.md` - Release verification guide
- `docs/SIGNING.md` - Package signing procedures
- `docs/SECURITY.md` - Security considerations
- `action.yml` - GitHub Action with all inputs/outputs documented

### External Links

- **Repository**: https://github.com/hongkongkiwi/rusty-commit
- **Crates.io**: https://crates.io/crates/rusty-commit
- **Documentation**: https://docs.rs/rusty-commit
- **CI/CD**: https://github.com/hongkongkiwi/rusty-commit/actions

### Related Projects

- Inspired by [OpenCommit](https://github.com/di-sukharev/opencommit)

---

## License

MIT License - See [LICENSE](LICENSE) file for details.
