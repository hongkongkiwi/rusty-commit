# Rusty Commit (rco)

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rusty-commit.svg)](https://crates.io/crates/rusty-commit)
[![Documentation](https://docs.rs/rusty-commit/badge.svg)](https://docs.rs/rusty-commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

AI-powered commit message generator written in Rust. Generate conventional commits, GitMoji messages, and PR descriptions using 18+ AI providers.

**Fast. Local-first options. Secure. Editor integrations via MCP.**

### Why Rusty Commit
- **Speed**: Native Rust binary with instant startup
- **Choice**: Works with 18+ AI providers (OpenAI, Anthropic/Claude, OpenRouter, Groq, DeepSeek, GitHub Copilot, Ollama, Fireworks, Moonshot/Kimi, Alibaba DashScope/Qwen, Mistral, and more)
- **Secure**: Optional keychain storage via `--features secure-storage`
- **Flexible**: Conventional commits, GitMoji, templates, multi-language
- **Integrated**: Git hooks, GitHub Actions, MCP server for editors
- **Multi-account**: Switch between multiple providers/accounts seamlessly

## Contents
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Generate Commit Messages](#generate-commit-modes)
- [Interactive Mode](#interactive-mode)
- [PR Description Generation](#pr-description-generation)
- [Examples](#examples)
- [Configuration](#configuration)
- [Multi-Account Support](#multi-account-support)
- [Providers](#providers)
- [Git Hooks](#git-hooks)
- [MCP Server](#mcp-server)
- [Updates](#updates)
- [GitHub Action](#github-action)
- [Advanced Options](#advanced-options)
- [File Exclusion](#file-exclusion)
- [Troubleshooting](#troubleshooting)
- [Uninstall](#uninstall)
- [Compatibility](#compatibility)
- [Development](#development)
- [Security & Verification](#security--verification)
- [License](#license)
- [Credits](#credits)

## Installation

### One-liner (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

**Security-conscious users:** [Verify the install script](docs/INSTALL-SCRIPT-VERIFICATION.md) before running it.

The script auto-detects your platform and installs via Homebrew, .deb/.rpm, Cargo, or binary.
**All packages are cryptographically signed and verified automatically:**
- Cosign/Sigstore signatures (keyless, modern)
- GPG signatures (traditional)
- SHA256 checksums
- GitHub build attestations

### Homebrew (macOS/Linux)
```bash
brew tap hongkongkiwi/tap
brew install rusty-commit
```

### Cargo
```bash
cargo install rusty-commit                      # basic
cargo install rusty-commit --features secure-storage  # store API keys in system keychain
```

### Debian/Ubuntu
```bash
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit_amd64.deb
sudo dpkg -i rusty-commit_amd64.deb
```

### Fedora/RHEL
```bash
sudo dnf install https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit.x86_64.rpm
```

### Alpine Linux
```bash
# Direct .apk install (signed packages)
wget https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rusty-commit-x86_64.apk
sudo apk add --allow-untrusted rusty-commit-x86_64.apk

# Or via binary (all architectures):
# x86_64
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-musl-x86_64.tar.gz | tar xz
sudo mv rco /usr/local/bin/

# aarch64
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-musl-aarch64.tar.gz | tar xz
sudo mv rco /usr/local/bin/

# riscv64
curl -fsSL https://github.com/hongkongkiwi/rusty-commit/releases/latest/download/rustycommit-linux-musl-riscv64.tar.gz | tar xz
sudo mv rco /usr/local/bin/
```

### Windows (Scoop)
```bash
scoop install rusty-commit
```

## Quick Start

### Setup Wizard (Recommended for New Users)
```bash
rco setup          # Interactive wizard to configure everything
```

### Single Provider (Traditional)
```bash
# 1) Authenticate (Claude OAuth) or set an API key
rco auth login
# or
rco config set RCO_API_KEY=sk-...

# 2) Generate a commit
git add .
rco
```

### Multi-Account (Recommended for Multiple Providers)
```bash
# 1) Add your first account (interactive wizard)
rco config add-provider

# 2) Add more accounts for different providers/roles
rco config add-provider  # Add "work-openai", "personal-anthropic", etc.

# 3) List all accounts
rco config list-accounts

# 4) Switch between accounts
rco config use-account work-openai
rco config use-account personal-anthropic

# 5) Generate commits (uses active account automatically)
git add .
rco
```

## Generate Commit Modes

### Basic Generation
```bash
rco                         # Generate and commit
rco -y                      # Skip confirmation, commit automatically
rco -c "Fix login bug"      # Add extra context
```

### Multiple Variations
Generate up to 5 variations and choose the best one:
```bash
rco -g 3                    # Generate 3 variations to choose from
rco -g 5 -y                 # Generate 5, auto-select first (use -y carefully)
```

### GitMoji Format
```bash
rco --fgm                   # Use full GitMoji specification
rco -fgm -y                 # GitMoji + auto-commit
```

### Clipboard Mode
Copy generated message to clipboard instead of committing:
```bash
rco -C                      # Copy to clipboard
rco -C -y                   # Copy and skip confirmation
```

### Show Prompt (Debug)
```bash
rco --show-prompt           # Print the AI prompt and exit (no commit)
```

### Timing Information
```bash
rco --timing                # Show detailed timing information
```

### Thinking Tags Stripping
For reasoning models (like DeepSeek R1) that output `<thinking>` tags:
```bash
rco --strip-thinking        # Strip <thinking> tags from response
```

## Interactive Mode

When running `rco` without `-y`, you'll enter interactive mode:

1. **Review generated message** - See the AI-generated commit message
2. **Edit before committing** - Press key to edit in your `$EDITOR`
3. **Regenerate** - Press key to generate a new variation
4. **Multiple variations** - With `-g 3`, see all variations and pick one

### Workflow
```bash
git add .                   # Stage your changes
rco                         # Enter interactive mode
# Choose to: edit, regenerate, select different variation, or abort
```

## PR Description Generation

Generate comprehensive pull request descriptions from your changes:

### Generate PR Description
```bash
rco pr generate             # Generate PR description for current branch
rco pr generate --base main # Compare against main branch
```

### Open PR in Browser
```bash
rco pr browse               # Generate and open PR creation page
rco pr browse --base main   # Use main as base branch
```

### PR Description Features
- Analyzes all commits since base branch
- Groups changes by type (features, fixes, docs, etc.)
- Generates following PR template conventions
- Includes breaking changes and deprecation notices

## Examples

### Conventional Commit
```
feat(auth): fix token refresh edge-case

Handle clock-skew by allowing ±60s leeway during token expiry checks. Adds retry on 429 and surfaces actionable errors.
```

### GitMoji (with --fgm)
```
✨ auth: robust token refresh with retry

Allow ±60s clock-skew; add backoff on 429; improve error messages for invalid credentials.
```

### With Context
```bash
rco -c "This fixes the OAuth flow for Google sign-in"
```

## Configuration

### Configuration Locations
1. **Global**: `~/.config/rustycommit/config.{toml,json}`
2. **Per-repo**: `.rustycommit.toml` / `.rco.toml`
3. **Environment Variables**: `RCO_*` prefix

**Priority**: Per-repo > Global > Environment variables > Defaults

### Basic Commands
```bash
rco config status                          # Secure storage status
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_MODEL=claude-3-5-haiku-20241022
rco config set RCO_COMMIT_TYPE=conventional
rco config set RCO_EMOJI=true
rco config get RCO_AI_PROVIDER
rco config describe                        # Show all config options
rco config reset --all                     # Reset to defaults
```

### Common Configuration Keys

| Key | Description | Example |
|-----|-------------|---------|
| `RCO_AI_PROVIDER` | AI backend to use | `anthropic`, `openai`, `ollama` |
| `RCO_MODEL` | Model name | `claude-3-5-haiku-20241022`, `gpt-4o-mini` |
| `RCO_API_KEY` | API key | `sk-...`, `gsk_...` |
| `RCO_API_URL` | Custom endpoint | `http://localhost:11434` |
| `RCO_COMMIT_TYPE` | Commit format | `conventional`, `gitmoji` |
| `RCO_EMOJI` | Include emojis | `true`, `false` |
| `RCO_LANGUAGE` | Output language | `en`, `es`, `fr`, `zh`, `ja` |
| `RCO_MAX_TOKENS` | Max response tokens | `1024` |
| `RCO_TEMPERATURE` | Response creativity | `0.7` (0.0-1.0) |
| `RCO_DIFF_TOKENS` | Max diff tokens | `12000` |

### Set Multiple Values
```bash
rco config set RCO_AI_PROVIDER=anthropic RCO_MODEL=claude-3-5-haiku-20241022 RCO_EMOJI=true
```

### Describe All Options
```bash
rco config describe    # Comprehensive config documentation with examples
```

## Multi-Account Support

Manage multiple AI provider accounts for different providers, models, or roles (work vs personal).

### Account Management Commands

```bash
# Add a new account (interactive wizard)
rco config add-provider

# Add a specific provider with alias
rco config add-provider --provider openai --alias work-openai

# List all configured accounts
rco config list-accounts

# Switch to a different account
rco config use-account work-openai
rco config use-account personal-anthropic

# Show details of an account
rco config show-account work-openai
rco config show-account  # shows "default" account

# Remove an account
rco config remove-account work-openai
```

### Interactive Add-Provider Wizard

Running `rco config add-provider` guides you through:

1. **Provider Selection** - Choose from 18+ providers
2. **Account Alias** - Memorable name (e.g., "work", "personal", "gpt-4")
3. **Model Name** - Optional, defaults to provider's recommended model
4. **API URL** - Optional, for custom endpoints or self-hosted providers
5. **API Key** - Stored securely in system keychain

### Account Configuration File

Accounts are stored in `~/.config/rustycommit/accounts.toml`:

```toml
[accounts.work-openai]
provider = "openai"
model = "gpt-4o-mini"
api_url = "https://api.openai.com/v1"

[accounts.work-openai.auth]
type = "api_key"
key_id = "rco_work_openai"

[accounts.personal-anthropic]
provider = "anthropic"
model = "claude-3-5-haiku-20241022"

[accounts.personal-anthropic.auth]
type = "env_var"
name = "ANTHROPIC_API_KEY"

[active_account]
alias = "work-openai"
```

### Authentication Methods

- `api_key` - Stored securely in keychain
- `oauth` - OAuth tokens from provider
- `env_var` - Read from environment variable
- `bearer` - Bearer tokens in keychain

## Providers

Works with 18+ providers. Configure via `RCO_AI_PROVIDER`.

### OAuth-Enabled Providers (use `rco auth login`)
- **Claude (Anthropic)**: OAuth with PKCE for secure authentication
- **Claude Code**: OAuth for Claude Code subscription users
- **GitHub Copilot**: OAuth for Copilot access

### API Key Providers

#### OpenAI
```bash
rco config set RCO_AI_PROVIDER=openai
rco config set RCO_API_KEY=sk-...
rco config set RCO_MODEL=gpt-4o-mini
# Get keys: https://platform.openai.com/api-keys
```

#### Anthropic Claude
```bash
# OAuth (recommended)
rco auth login
# Or API key
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_API_KEY=sk-ant-...
rco config set RCO_MODEL=claude-3-5-haiku-20241022
# Keys: https://console.anthropic.com/settings/keys
```

#### Claude Code OAuth
```bash
# OAuth login (works for both Anthropic and Claude Code)
rco auth login

# Claude Code subscription users can use the Claude Code model
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_MODEL=claude-3-5-sonnet-20241022
```

#### OpenRouter
```bash
rco config set RCO_AI_PROVIDER=openrouter
rco config set RCO_API_KEY=sk-or-...
rco config set RCO_MODEL=openai/gpt-4o-mini
# Keys: https://openrouter.ai/keys
```

#### Groq
```bash
rco config set RCO_AI_PROVIDER=groq
rco config set RCO_API_KEY=gsk_...
rco config set RCO_MODEL=llama-3.1-70b-versatile
# Keys: https://console.groq.com/keys
```

#### DeepSeek
```bash
rco config set RCO_AI_PROVIDER=deepseek
rco config set RCO_API_KEY=sk-...
rco config set RCO_MODEL=deepseek-chat
# Keys: https://platform.deepseek.com/api-keys
```

#### xAI Grok
```bash
rco config set RCO_AI_PROVIDER=xai
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=grok-beta
# Keys: https://x.ai/api
```

#### Mistral AI
```bash
rco config set RCO_AI_PROVIDER=mistral
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=mistral-small-latest
# Keys: https://console.mistral.ai/api-keys
```

#### Google Gemini
```bash
rco config set RCO_AI_PROVIDER=gemini
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=gemini-pro
# Keys: https://aistudio.google.com/app/apikey
```

#### Azure OpenAI
```bash
rco config set RCO_AI_PROVIDER=azure
rco config set RCO_API_KEY=<azure_api_key>
rco config set RCO_API_URL=https://<resource>.openai.azure.com
rco config set RCO_MODEL=<deployment-name>
# Docs: https://learn.microsoft.com/azure/ai-services/openai
```

#### Perplexity
```bash
rco config set RCO_AI_PROVIDER=perplexity
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=llama-3.1-sonar-small-128k-online
# Keys: https://www.perplexity.ai/settings/api
```

#### Ollama (Local)
```bash
rco config set RCO_AI_PROVIDER=ollama
rco config set RCO_MODEL=mistral
rco config set RCO_API_URL=http://localhost:11434
# No API key needed for local models
```

#### Fireworks AI
```bash
rco config set RCO_AI_PROVIDER=fireworks
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=accounts/fireworks/models/llama-v3p1-70b-instruct
# Keys: https://app.fireworks.ai/users/api-keys
```

#### Moonshot AI (Kimi)
```bash
rco config set RCO_AI_PROVIDER=moonshot
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.moonshot.cn/v1
rco config set RCO_MODEL=kimi-k2
# Docs: https://platform.moonshot.ai/docs
```

#### Alibaba DashScope (Qwen)
```bash
rco config set RCO_AI_PROVIDER=dashscope
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
rco config set RCO_MODEL=qwen3-coder-32b-instruct
# Keys: https://dashscope.console.aliyun.com/apiKey
```

#### Together AI
```bash
rco config set RCO_AI_PROVIDER=together
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo
# Keys: https://api.together.xyz/settings/api-keys
```

#### DeepInfra
```bash
rco config set RCO_AI_PROVIDER=deepinfra
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=meta-llama/Meta-Llama-3-70B-Instruct
# Keys: https://deepinfra.com/dash/api_keys
```

#### Vertex AI (Google Cloud)
```bash
rco config set RCO_AI_PROVIDER=vertex
rco config set RCO_API_URL=https://your-gateway/v1
rco config set RCO_MODEL=google/gemini-1.5-pro
# Requires gateway to Vertex's OpenAI-compatible endpoint
```

### Secure Storage

When built with `secure-storage` feature:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring, KWallet, KeePassXC)
- **Windows**: Credential Manager
- Falls back to config file if unavailable

## Git Hooks

### Install Hook
```bash
rco hook set    # Install prepare-commit-msg hook
rco hook unset  # Uninstall
```

The hook automatically generates commit messages when you run `git commit` without `-m`.

### Optional Hooks (Advanced)

Configure pre/post hooks in config (globally or per-repo):

```bash
# Pre-generation hook (run before AI generates message)
rco config set RCO_PRE_GEN_HOOK="just lint; just test -q"

# Pre-commit hook (after generation, can edit message)
rco config set RCO_PRE_COMMIT_HOOK="./scripts/tweak_commit.sh"

# Post-commit hook (after git commit completes)
rco config set RCO_POST_COMMIT_HOOK="git push"

# Hook behavior
rco config set RCO_HOOK_STRICT=false    # Allow hook failures (default: true)
rco config set RCO_HOOK_TIMEOUT_MS=60000 # Timeout in ms (default: 30000)
```

### Per-Run Hook Control
```bash
rco --no-pre-hooks      # Skip pre-gen + pre-commit hooks
rco --no-post-hooks     # Skip post-commit hooks
```

### Hook Environment Variables

Hooks receive these environment variables:
- `RCO_REPO_ROOT`, `RCO_PROVIDER`, `RCO_MODEL`
- `RCO_MAX_TOKENS`, `RCO_DIFF_TOKENS`, `RCO_CONTEXT` (pre-gen)
- `RCO_COMMIT_MESSAGE`, `RCO_COMMIT_FILE` (pre/post-commit)

## MCP Server

Rusty Commit includes an MCP (Model Context Protocol) server for editor integrations.

### Start MCP Server

**TCP Mode (for Cursor, VS Code MCP extension):**
```bash
rco mcp server --port 3000
```

**STDIO Mode (for direct integration):**
```bash
rco mcp stdio
```

### Editor Integration

#### Cursor
1. Open Cursor Settings > Features > MCP
2. Add new MCP server:
   ```
   Type: HTTP
   URL: http://localhost:3000
   ```
3. Or use stdio with cursor-agent

#### VS Code
1. Install "MCP" extension by modelcontextprotocol
2. Add server configuration in settings.json

#### Claude Code
```bash
rco mcp stdio | claude-code connect stdio
```

### MCP Tools Available
- `generate_commit` - Generate commit message from staged changes
- `generate_commit_message` - Alias for generate_commit
- `get_config` - Get current configuration
- `set_config` - Set configuration value
- `list_accounts` - List configured accounts
- `use_account` - Switch active account

## Updates

```bash
rco update --check   # See if a new version is available
rco update           # Update using your install method
rco update --force   # Force update even if current
rco update --version 1.0.2  # Install specific version
```

### Architectures
Prebuilt archives and packages for:
- **Linux**: x86_64 (gnu, musl), aarch64 (gnu, musl), armv7, riscv64
- **macOS**: x86_64, aarch64 (Apple Silicon)
- **Windows**: x86_64, i686

## GitHub Action

```yaml
name: AI Commits
on: [push]
jobs:
  ai-commit:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: hongkongkiwi/rusty-commit@main
        with:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RCO_API_KEY: ${{ secrets.RCO_API_KEY }}
          RCO_AI_PROVIDER: 'anthropic'
          RCO_MODEL: 'claude-3-5-haiku-20241022'
```

### GitHub Action Inputs

| Input | Description | Required |
|-------|-------------|----------|
| `RCO_API_KEY` | API key for AI provider | Yes |
| `RCO_AI_PROVIDER` | Provider name | No (default: anthropic) |
| `RCO_MODEL` | Model name | No |
| `RCO_COMMIT_TYPE` | conventional or gitmoji | No |
| `GITHUB_TOKEN` | GitHub token for commit | Yes |

## Advanced Options

### Token Management

For large diffs, Rusty Commit automatically chunks the input:

```bash
rco config set RCO_DIFF_TOKENS=12000   # Max tokens per chunk (default: 12000)
rco config set RCO_MAX_TOKENS=1024     # Max response tokens (default: 1024)
rco config set RCO_TEMPERATURE=0.7     # Response creativity (0.0-1.0)
```

### Commitlint Integration

Generate commitlint configuration:
```bash
rco commitlint              # Generate commitlint.config.js
rco commitlint --set        # Non-interactive generation
```

### Model Selection

List available models for your provider:
```bash
rco model --list            # List models for current provider
rco model --list --provider openai  # List OpenAI models
```

### OAuth Authentication

```bash
rco auth login              # Login with default provider (Claude)
rco auth login --provider anthropic  # Specific provider
rco auth logout             # Remove stored tokens
rco auth status             # Check authentication status
```

## File Exclusion

### Exclude Files from AI Analysis

**Via command line:**
```bash
rco -x "package-lock.json" -x "yarn.lock"      # Exclude specific files
rco -x "*.lock" -x "*.min.js"                   # Exclude patterns
```

**Via .rcoignore file:**
Create `.rcoignore` in your repository root:
```
# Dependencies
node_modules/
vendor/
Cargo.lock

# Build artifacts
*.min.js
*.map
dist/
build/

# IDE
.idea/
.vscode/
*.swp

# OS
.DS_Store
Thumbs.db

# Logs
*.log
npm-debug.log*
```

Files matching patterns in `.rcoignore` are excluded from the diff sent to AI.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| **401 / Invalid API key** | Re-authenticate (`rco auth login`) or set valid `RCO_API_KEY`. For accounts, check `rco config show-account`. |
| **Rate-limited (429)** | Wait briefly; try lighter model or switch accounts (`rco config use-account <alias>`). |
| **Secure storage unavailable** | Falls back to file storage; check `rco config status`. Build with `secure-storage` feature for keychain support. |
| **Account not found** | Verify accounts exist with `rco config list-accounts`. Use exact alias name when switching. |
| **Wrong account used** | Check active account with `rco config show-account`. Switch with `rco config use-account <alias>`. |
| **Hooks not running** | Ensure `.git/hooks/prepare-commit-msg` exists and is executable. Re-install via `rco hook set`. |
| **MCP server connection refused** | Ensure server is running (`rco mcp server`). Check port matches editor config. |
| **OAuth browser not opening** | Set `BROWSER=none` to get URL manually, or use `rco auth login --no-browser`. |
| **Windows PATH issues** | Add install dir (`%USERPROFILE%\.cargo\bin`) to PATH. |
| **Corporate proxy** | Set `HTTP_PROXY`/`HTTPS_PROXY` environment variables. |
| **Large diff truncated** | Increase `RCO_DIFF_TOKENS` or exclude files with `.rcoignore` / `-x`. |
| **Reasoning model outputs thinking tags** | Use `--strip-thinking` flag or set `RCO_STRIP_THINKING=true`. |
| **Commit message too long** | Decrease `RCO_MAX_TOKENS` or set `RCO_COMMIT_TYPE=conventional`. |

### Debug Mode
```bash
RUST_LOG=debug rco -y     # Enable debug logging
rco --show-prompt         # See exact prompt sent to AI
rco --timing              # Show timing breakdown
```

## Uninstall

- **Homebrew**: `brew uninstall rusty-commit`
- **Cargo**: `cargo uninstall rusty-commit`
- **APT**: `sudo apt remove rusty-commit`
- **RPM**: `sudo rpm -e rusty-commit`
- **Remove config**: `rm -rf ~/.config/rustycommit/`
- **Remove accounts**: `rm -f ~/.config/rustycommit/accounts.toml`

## Compatibility

- Git 2.23+
- Rust 1.70+
- Works with per-repo overrides, multiple providers, and multi-account configurations
- Accounts stored separately in `~/.config/rustycommit/accounts.toml`

### Provider Compatibility Matrix

| Provider | OAuth | API Key | Local | Notes |
|----------|-------|---------|-------|-------|
| OpenAI | No | Yes | No | |
| Anthropic | Yes | Yes | No | Claude, Claude Code |
| OpenRouter | No | Yes | No | Aggregator |
| Groq | No | Yes | No | Fast inference |
| DeepSeek | No | Yes | No | |
| Ollama | No | No | Yes | Local models |
| Gemini | No | Yes | No | |
| Azure | No | Yes | No | |
| xAI | No | Yes | No | Grok |
| Mistral | No | Yes | No | |
| Perplexity | No | Yes | No | |
| Fireworks | No | Yes | No | |
| Moonshot | No | Yes | No | Kimi |
| DashScope | No | Yes | No | Qwen |
| Together | No | Yes | No | |
| DeepInfra | No | Yes | No | |
| GitHub Copilot | Yes | No | No | |

## Development

```bash
# Build
cargo build
cargo build --release
cargo build --features secure-storage

# Test
cargo test
cargo test --all-features
cargo test --doc

# Lint
cargo clippy --all-features -- -D warnings
cargo fmt

# Documentation
cargo doc --no-deps
```

### Just Commands
```bash
just build              # Build project
just test               # Run tests
just fmt                # Format code
just clippy             # Run clippy
just all                # Run fmt, clippy, and test
just release            # Create release build
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Add tests for new functionality
6. Submit a PR

## Security & Verification

All releases are cryptographically signed with multiple methods for maximum security.

### Automatic Verification

The install script automatically verifies all downloads using the strongest available method.

### Manual Verification

```bash
# Modern: Cosign/Sigstore (recommended)
cosign verify-blob \
  --bundle rustycommit-linux-x86_64.tar.gz.cosign.bundle \
  --certificate-identity-regexp "https://github.com/hongkongkiwi/rusty-commit/.github/workflows/release.yml@.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  rustycommit-linux-x86_64.tar.gz

# Traditional: GPG signatures
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD
gpg --verify rustycommit-linux-x86_64.tar.gz.asc rustycommit-linux-x86_64.tar.gz

# GitHub attestations
gh attestation verify rustycommit-linux-x86_64.tar.gz --repo hongkongkiwi/rusty-commit

# Package signatures (native)
dpkg-sig --verify rusty-commit_1.0.0_amd64.deb  # Debian/Ubuntu
rpm --checksig rusty-commit-1.0.0-1.x86_64.rpm  # Fedora/RHEL
```

**Full verification guide:** [docs/VERIFICATION.md](docs/VERIFICATION.md)

## Support the Project

If Rusty Commit saves you time, consider supporting ongoing development:

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-@hongkongkiwi-fd2e83?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/hongkongkiwi)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-ffdd00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/hongkongkiwi)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Credits

Rusty Commit is inspired by the original [OpenCommit](https://github.com/di-sukharev/opencommit) by [@di-sukharev](https://github.com/di-sukharev).

---

**Build faster. Commit smarter.**
