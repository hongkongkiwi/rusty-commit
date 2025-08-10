# Rusty Commit (rco)

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
[![CodeRabbit](https://img.shields.io/badge/CodeRabbit-AI%20Review-blue)](https://github.com/hongkongkiwi/rusty-commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/rustycommit.svg)](https://crates.io/crates/rustycommit)
[![Documentation](https://docs.rs/rustycommit/badge.svg)](https://docs.rs/rustycommit)

🚀 **Blazing-fast commit messages powered by AI and written in Rust** 🦀

**Rusty Commit** (`rco`) is a high-performance, feature-rich commit message generator written in Rust. Inspired by the excellent work of [@di-sukharev](https://github.com/di-sukharev) and the original [OpenCommit](https://github.com/di-sukharev/opencommit) project, this Rust implementation brings enhanced performance, memory safety, and additional powerful features while maintaining full compatibility with OpenCommit configurations.

## 🙏 Attribution

This project is heavily inspired by and maintains compatibility with [OpenCommit](https://github.com/di-sukharev/opencommit) by [@di-sukharev](https://github.com/di-sukharev). We're grateful for their pioneering work in AI-powered commit message generation. Rusty Commit aims to complement the original by providing a Rust-native alternative with enhanced performance and additional features.

## 🌟 Features

**Performance & Reliability**
- ⚡ **Blazing fast** - Native Rust performance with minimal startup time
- 🦀 **Memory safe** - Rust's ownership system prevents crashes and memory leaks
- 📦 **Single binary** - No runtime dependencies, just drop and run

**AI Provider Support**
- 🤖 **16+ AI providers**: OpenAI, Anthropic/Claude, OpenRouter, Groq, DeepSeek, Mistral, AWS Bedrock, GitHub Copilot, and more
- 🔐 **OAuth authentication** - Direct Claude Pro/Max integration (no API key needed)
- 🛡️ **Secure credential storage** - System keychain integration

**Configuration & Flexibility**
- 📄 **Multiple config formats** - TOML, JSON, and legacy .env support
- 🌍 **Per-repo configs** - Different settings for different projects
- 🔄 **Full OpenCommit compatibility** - Seamless migration from original
- 🎨 **Format support** - Conventional commits, GitMoji, custom templates

**Developer Experience**
- 🪝 **Git hooks integration** - Auto-generate on commit
- 🌐 **Multi-language** - Generate commits in multiple languages
- 🎯 **Context-aware** - Smart diff analysis for better messages
- 📊 **Interactive menus** - Beautiful CLI with provider selection

## Installation

### From source

```bash
# Basic installation (stores API keys in config file)
cargo install --path .

# With secure storage support (stores API keys in system keychain)
cargo install --path . --features secure-storage
```

### From crates.io (when published)

```bash
# Basic installation
cargo install rustycommit

# With secure storage support
cargo install rustycommit --features secure-storage
```

### Platform Support

Rusty Commit supports secure credential storage on all major platforms:

#### macOS
- **Storage**: macOS Keychain (login keychain)
- **Requirements**: No additional setup needed
- **Access**: Managed by macOS security framework

#### Linux
- **Storage**: Secret Service API
- **Supported backends**:
  - GNOME Keyring (GNOME desktop)
  - KWallet (KDE desktop)
  - KeePassXC with Secret Service integration
- **Requirements**: Desktop environment with Secret Service support
- **Fallback**: File-based storage for headless/minimal systems

#### Windows
- **Storage**: Windows Credential Manager
- **Requirements**: Windows 7 or later
- **Access**: Integrated with Windows user account

#### Other Platforms
- **FreeBSD/OpenBSD**: Secret Service if available
- **Docker/Containers**: Falls back to file storage
- **Headless systems**: Falls back to file storage
- **All platforms**: Automatic fallback to `~/.config/rustycommit/` if keychain unavailable

## Quick Start

1. **Install Rusty Commit:**
   ```bash
   cargo install rustycommit
   ```

2. **Set up your API key:**
   ```bash
   rco config set RCO_API_KEY=your_api_key_here
   ```

   Or use OAuth for Claude:
   ```bash
   rco auth login
   ```

3. **Generate a commit:**
   ```bash
   git add .
   rco
   ```

## Configuration

Rusty Commit supports multiple configuration formats and locations:

### Configuration Files

- **Global config**: `~/.config/rustycommit/config.toml` (or `.json`)
- **Repo config**: `.rustycommit.toml` or `.rco.toml` (or `.json`) in your repo root
- **Legacy support**: Reads existing `~/.opencommit` files for compatibility

### Configure using the `config` command:

```bash
# Set API key (stored securely if keychain is available)
rco config set RCO_API_KEY=sk-...

# Check secure storage status
rco config status

# Set AI provider (supports 16+ providers)
rco config set RCO_AI_PROVIDER=anthropic

# Set model
rco config set RCO_MODEL=claude-3-5-haiku-20241022

# Enable emojis (can be set per-repo)
rco config set RCO_EMOJI=true

# Set commit type (conventional, gitmoji)
rco config set RCO_COMMIT_TYPE=conventional

# View a configuration value
rco config get RCO_AI_PROVIDER

# Reset configuration
rco config reset --all
```

### Secure Storage

Rusty Commit can store your API keys securely in your system's keychain:

- API keys are encrypted and protected by your system
- Keys are stored separately from the configuration file
- Automatic fallback to file storage if keychain is unavailable

To check if secure storage is available:
```bash
rco config status
```

## Using with Ollama (Local AI)

1. Install and start Ollama
2. Pull a model: `ollama run mistral`
3. Configure Rusty Commit:
   ```bash
   rco config set RCO_AI_PROVIDER=ollama
   rco config set RCO_MODEL=mistral
   ```

For remote Ollama instances:
```bash
rco config set RCO_API_URL=http://192.168.1.10:11434
```

## Provider-Specific Setup

### OpenRouter (Access 200+ Models)
```bash
rco auth login  # Select OpenRouter
# Or manually:
rco config set RCO_AI_PROVIDER=openrouter
rco config set RCO_API_KEY=sk-or-...
rco config set RCO_MODEL=openai/gpt-4o-mini
```

### AWS Bedrock (New 2025 API Keys)
```bash
rco auth login  # Select AWS Bedrock
# Or manually with new API key method:
export AWS_BEARER_TOKEN_BEDROCK=your_api_key
rco config set RCO_AI_PROVIDER=amazon-bedrock
rco config set RCO_MODEL=us.anthropic.claude-3-5-haiku-20241022-v1:0
```

### Groq (Ultra-Fast Inference)
```bash
rco auth login  # Select Groq
# Or manually:
rco config set RCO_AI_PROVIDER=groq
rco config set RCO_API_KEY=gsk_...
rco config set RCO_MODEL=llama-3.1-70b-versatile
```

### DeepSeek (Cost-Effective)
```bash
rco auth login  # Select DeepSeek
# Or manually:
rco config set RCO_AI_PROVIDER=deepseek
rco config set RCO_API_KEY=sk-...
rco config set RCO_MODEL=deepseek-chat
```

### GitHub Copilot (Free for Subscribers)
```bash
gh auth login  # First authenticate with GitHub CLI
rco auth login  # Then select GitHub Copilot
```

## Git Hooks

Install the prepare-commit-msg hook to automatically generate commit messages:

```bash
# Install hook
rco hook set

# Uninstall hook
rco hook unset
```

## GitHub Action

Use Rusty Commit in your GitHub workflows to automatically generate commit messages:

### Basic Usage

```yaml
name: 'Auto Commit with Rusty Commit'
on:
  push:
    branches-ignore: [main, master]

jobs:
  rustycommit:
    name: Generate AI Commit Messages
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
          token: ${{ secrets.GITHUB_TOKEN }}

      - uses: hongkongkiwi/rusty-commit@main
        with:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RCO_API_KEY: ${{ secrets.RCO_API_KEY }}
          RCO_AI_PROVIDER: 'anthropic'
          RCO_MODEL: 'claude-3-5-haiku-20241022'
          RCO_COMMIT_TYPE: 'conventional'
          confirm: 'true'
          push: 'true'
```

### Available Action Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `GITHUB_TOKEN` | GitHub token for commits | **Required** |
| `RCO_API_KEY` | AI provider API key | - |
| `RCO_AI_PROVIDER` | AI provider (openai, anthropic, openrouter, groq, github-copilot, etc.) | `openai` |
| `RCO_MODEL` | AI model to use | `gpt-4o-mini` |
| `RCO_COMMIT_TYPE` | Commit format (conventional, gitmoji) | `conventional` |
| `RCO_EMOJI` | Enable emojis in commit messages | `false` |
| `RCO_LANGUAGE` | Language for commit messages (en, es, fr, de, etc.) | `en` |
| `RCO_DESCRIPTION_MAX_LENGTH` | Maximum commit description length | `100` |
| `RCO_TOKENS_MAX_INPUT` | Maximum input tokens for AI provider | `4096` |
| `RCO_TOKENS_MAX_OUTPUT` | Maximum output tokens for AI provider | `500` |
| `RCO_API_URL` | Custom API endpoint URL (for Ollama, custom providers) | - |
| `RCO_SECURE_STORAGE` | Use secure credential storage (keychain) | `true` |
| `full-gitmoji` | Use full GitMoji specification | `false` |
| `context` | Additional context for the commit message | - |
| `exclude` | Files to exclude from diff analysis (glob patterns) | - |
| `diff-unified` | Number of lines of context in diff | `3` |
| `one-line-commit` | Generate single-line commit messages | `false` |
| `confirm` | Skip confirmation prompt (auto-commit) | `true` |
| `push` | Push commits after creation | `false` |


### Multi-Provider Example

```yaml
# Try GitHub Copilot first, fallback to other providers
- uses: hongkongkiwi/opencommit-rust@main
  continue-on-error: true
  with:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    RCO_AI_PROVIDER: 'github-copilot'

- uses: hongkongkiwi/opencommit-rust@main
  if: steps.copilot.outcome == 'failure'
  with:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    RCO_API_KEY: ${{ secrets.RCO_API_KEY }}
    RCO_AI_PROVIDER: 'groq'
    RCO_MODEL: 'llama-3.1-70b-versatile'
```

Check the [examples directory](.github/workflows/) for more advanced workflows including PR automation, scheduled commits, and multi-provider setups.

## Command Line Options

```bash
# Use full GitMoji specification
rco --fgm

# Add context to the commit message
rco --context "Fixed the bug in authentication"

# Skip confirmation prompt
rco --yes
```

## Supported AI Providers

Rusty Commit supports **16+ AI providers** with easy authentication setup:

### 🚀 **Recommended Providers**
- **Anthropic Claude** - Claude Pro/Max OAuth + API keys (Claude 3.5 Haiku/Sonnet)
- **GitHub Copilot** - Free for subscribers, powered by GPT-4o
- **OpenAI** - GPT-4o, GPT-4o-mini, and other OpenAI models
- **OpenRouter** - Access 200+ models from multiple providers in one API

### ⚡ **High-Performance Providers**
- **Groq** - Ultra-fast inference with Llama models
- **DeepSeek** - Cost-effective reasoning and coding models
- **Together AI** - Optimized open-source model hosting

### ☁️ **Cloud Platform Providers**
- **AWS Bedrock** - Amazon's managed AI service (new 2025 API keys supported)
- **Azure OpenAI** - Microsoft's hosted OpenAI models
- **Google Gemini** - Gemini Pro and Vertex AI models
- **GitHub Models** - GitHub's hosted AI models

### 🤗 **Open Source & Research**
- **Hugging Face** - Inference API for open models
- **Ollama** - Local models (Llama, Mistral, CodeLlama, etc.)
- **DeepInfra** - Hosted open-source models
- **Mistral AI** - Mistral's native API

### 🔧 **Custom Providers**
- **Any OpenAI-compatible API** - Custom endpoints and providers

## Configuration Options

**Rusty Commit Variables** (Recommended):

| Key | Description | Default |
|-----|-------------|---------|
| `RCO_API_KEY` | API key for your AI provider | - |
| `RCO_API_URL` | Custom API endpoint | Provider default |
| `RCO_AI_PROVIDER` | AI provider to use | `openai` |
| `RCO_MODEL` | Model to use | Provider default |
| `RCO_TOKENS_MAX_INPUT` | Maximum input tokens | `4096` |
| `RCO_TOKENS_MAX_OUTPUT` | Maximum output tokens | `500` |
| `RCO_COMMIT_TYPE` | Commit format style | `conventional` |
| `RCO_EMOJI` | Enable emoji in commits | `false` |
| `RCO_LANGUAGE` | Language for commit messages | `en` |
| `RCO_DESCRIPTION_MAX_LENGTH` | Maximum description length | `100` |
| `RCO_SECURE_STORAGE` | Use secure credential storage | `true` |

**Example Configuration:**
```bash
rco config set RCO_API_KEY=your_api_key_here
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_MODEL=claude-3-5-haiku-20241022
```


## Development

### Using Just (Task Runner)

We use `just` as our command runner for common development tasks:

```bash
# Install just
cargo install just

# See all available commands
just

# Common workflows
just test           # Run tests
just lint           # Run clippy
just fmt            # Format code
just check          # Run all checks (format, lint, test)
just ci             # Run full CI locally

# Building
just build          # Build debug binary
just build-release  # Build release binary
just install        # Install locally

# Maintenance
just outdated       # Check for outdated dependencies
just audit          # Security audit
just update         # Update dependencies

# Release management
just release-patch  # Release patch version (1.0.0 -> 1.0.1)
just release-minor  # Release minor version (1.0.0 -> 1.1.0)  
just release-major  # Release major version (1.0.0 -> 2.0.0)
```

### Manual Commands

```bash
# Build the project
cargo build

# Build with all features (including secure storage)
cargo build --all-features

# Run tests
cargo test

# Run all tests including integration tests
cargo test --all

# Run tests with all features
cargo test --all-features

# Run with debug output
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Run clippy
cargo clippy --all-features -- -D warnings

# Run security audit
cargo audit
```

### Releasing

We provide a convenient release script that handles version bumping, tagging, and pushing:

```bash
# Release a patch version (1.0.0 -> 1.0.1)
./release.sh patch
# Or using just:
just release-patch

# Release a minor version (1.0.0 -> 1.1.0)
./release.sh minor
# Or using just:
just release-minor

# Release a major version (1.0.0 -> 2.0.0)
./release.sh major
# Or using just:
just release-major

# Release a specific version
./release.sh 1.2.3
# Or using just:
just release 1.2.3
```

The release script will:
1. Run tests and clippy to ensure quality
2. Update version in Cargo.toml
3. Commit the version bump
4. Create an annotated git tag
5. Push changes and tag to GitHub
6. Trigger GitHub Actions to:
   - Create a GitHub release
   - Build binaries for all platforms
   - Publish to crates.io

## Testing

Rusty Commit has comprehensive test coverage:

- **Unit Tests**: Core functionality and configuration
- **Integration Tests**: CLI commands and provider configurations
- **Authentication Tests**: OAuth flows and token management
- **Provider Tests**: All 16+ AI providers with specific configurations
- **Cross-platform**: Tests run on Linux, macOS, and Windows

```bash
# Run specific test suites
cargo test --test auth_test
cargo test --test providers_comprehensive_test
cargo test --test oauth_test
cargo test --test cli_integration_test
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure tests pass (`cargo test --all-features`)
6. Run clippy (`cargo clippy --all-features -- -D warnings`)
7. Format code (`cargo fmt`)
8. Commit your changes (`git commit -m 'feat: add amazing feature'`)
9. Push to the branch (`git push origin feat/amazing-feature`)
10. Open a Pull Request

### Security

Security is a top priority. If you discover a security vulnerability, please:

1. **Do NOT** open a public issue
2. Email security concerns to the maintainers
3. Include detailed steps to reproduce
4. Allow time for assessment and patching

### Code Style

- Follow Rust conventions and idioms
- Use `cargo fmt` for consistent formatting
- Address all `cargo clippy` warnings
- Add documentation for public APIs
- Include tests for new features

## License

MIT

## Credits

This is a Rust reimplementation of the original [OpenCommit](https://github.com/di-sukharev/opencommit) project by [@di-sukharev](https://github.com/di-sukharev).
