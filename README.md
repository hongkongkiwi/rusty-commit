<div align="center">

# 🦀 Rusty Commit (`rco`)

### **AI-powered commit message generator written in Rust**

**Generate conventional commits, GitMoji messages, and PR descriptions using 18+ AI providers**

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rusty-commit.svg)](https://crates.io/crates/rusty-commit)
[![Documentation](https://docs.rs/rusty-commit/badge.svg)](https://docs.rs/rusty-commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**🚀 Fast · 🔒 Secure · 🏠 Local-first · 🔌 Editor integrations via MCP**

[Installation](#installation) · [Quick Start](#quick-start) · [Features](#features) · [Providers](#providers) · [Configuration](#configuration) · [Git Hooks](#git-hooks)

</div>

---

## ✨ Why Rusty Commit?

<table>
<tr>
<td width="50%">

**⚡ Blazing Fast**

Native Rust binary with instant startup time. No Node.js bloat, no waiting.

**🤖 18+ AI Providers**

Works with OpenAI, Claude, OpenRouter, Groq, DeepSeek, GitHub Copilot, Ollama, and more.

**🔐 Secure by Default**

Optional keychain storage keeps your API keys safe. OAuth support for major providers.

</td>
<td width="50%">

**🎨 Flexible Formats**

Conventional commits, GitMoji, or custom templates. Multi-language support.

**🔄 Multi-Account Support**

Seamlessly switch between work and personal accounts, different providers, or models.

**🔌 Editor Integration**

MCP server for Cursor, VS Code, Claude Code, and other AI-powered editors.

</td>
</tr>
</table>

---

## 📦 Installation

### One-liner (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

<details>
<summary>🔐 Security-conscious? Verify the install script first</summary>

```bash
# Download and inspect
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh -o install.sh
# Verify with SHA256 checksums, Cosign, GPG, or GitHub attestations
# See: docs/INSTALL-SCRIPT-VERIFICATION.md
```

</details>

### Package Managers

| Platform | Command |
|----------|---------|
| **Homebrew** | `brew tap hongkongkiwi/tap && brew install rusty-commit` |
| **Cargo** | `cargo install rusty-commit --features secure-storage` |
| **Debian/Ubuntu** | `wget .../rusty-commit_amd64.deb && sudo dpkg -i rusty-commit_amd64.deb` |
| **Fedora/RHEL** | `sudo dnf install https://.../rusty-commit.x86_64.rpm` |
| **Alpine** | `wget .../rusty-commit-x86_64.apk && sudo apk add --allow-untrusted rusty-commit-x86_64.apk` |
| **Windows (Scoop)** | `scoop install rusty-commit` |
| **Windows (Binary)** | [Download from releases](https://github.com/hongkongkiwi/rusty-commit/releases) |

---

## 🚀 Quick Start

### Interactive Setup Wizard

```bash
rco setup          # Configure everything interactively
```

### Generate Your First Commit

```bash
# 1. Stage your changes
git add .

# 2. Generate commit message
rco

# 3. Review, edit, or regenerate interactively
```

### Multi-Account Workflow (Recommended)

```bash
# Add multiple provider accounts
rco config add-provider    # Add work-openai
rco config add-provider    # Add personal-anthropic

# Switch between them
rco config use-account work-openai
git add . && rco

rco config use-account personal-anthropic
git add . && rco
```

---

## 🎯 Features

### 📝 Commit Message Generation

```bash
rco                         # Interactive mode with review
rco -y                      # Auto-commit without confirmation
rco -g 3                    # Generate 3 variations to choose from
rco -c "Fix OAuth flow"     # Add extra context
```

### 😄 GitMoji Support

```bash
rco --fgm                   # Full GitMoji specification
rco -fgm -y                 # GitMoji + auto-commit
```

**Output:**
```
✨ feat(auth): implement OAuth2 PKCE flow

🔐 Add secure authentication with automatic token refresh.
Supports GitHub, GitLab, and generic OAuth2 providers.
```

### 📋 Clipboard Mode

```bash
rco -C                      # Copy to clipboard instead of committing
```

### 🔍 Debug Tools

```bash
rco --show-prompt           # See the exact prompt sent to AI
rco --timing                # Show detailed timing information
RUST_LOG=debug rco          # Enable debug logging
```

---

## 🤖 Providers

Rusty Commit works with **18+ AI providers** out of the box:

### 🔑 OAuth-Enabled (No API Key Required)

| Provider | Command |
|----------|---------|
| **Claude (Anthropic)** | `rco auth login` |
| **Claude Code** | `rco auth login` |
| **GitHub Copilot** | `rco auth login --provider github-copilot` |

### 🔐 API Key Providers

| Provider | Setup |
|----------|-------|
| **OpenAI** | `rco config set RCO_AI_PROVIDER=openai RCO_API_KEY=sk-... RCO_MODEL=gpt-4o-mini` |
| **Anthropic** | `rco config set RCO_AI_PROVIDER=anthropic RCO_API_KEY=sk-ant-... RCO_MODEL=claude-3-5-haiku-20241022` |
| **OpenRouter** | `rco config set RCO_AI_PROVIDER=openrouter RCO_API_KEY=sk-or-...` |
| **Groq** | `rco config set RCO_AI_PROVIDER=groq RCO_API_KEY=gsk_... RCO_MODEL=llama-3.1-70b-versatile` |
| **DeepSeek** | `rco config set RCO_AI_PROVIDER=deepseek RCO_API_KEY=sk-... RCO_MODEL=deepseek-chat` |
| **xAI/Grok** | `rco config set RCO_AI_PROVIDER=xai RCO_API_KEY=... RCO_MODEL=grok-beta` |
| **Mistral** | `rco config set RCO_AI_PROVIDER=mistral RCO_API_KEY=... RCO_MODEL=mistral-small-latest` |
| **Google Gemini** | `rco config set RCO_AI_PROVIDER=gemini RCO_API_KEY=... RCO_MODEL=gemini-pro` |
| **Azure OpenAI** | `rco config set RCO_AI_PROVIDER=azure RCO_API_KEY=... RCO_API_URL=https://<resource>.openai.azure.com` |
| **Perplexity** | `rco config set RCO_AI_PROVIDER=perplexity RCO_API_KEY=...` |
| **Ollama (Local)** | `rco config set RCO_AI_PROVIDER=ollama RCO_MODEL=mistral RCO_API_URL=http://localhost:11434` |
| **Fireworks** | `rco config set RCO_AI_PROVIDER=fireworks RCO_API_KEY=...` |
| **Moonshot/Kimi** | `rco config set RCO_AI_PROVIDER=moonshot RCO_API_KEY=... RCO_API_URL=https://api.moonshot.cn/v1 RCO_MODEL=kimi-k2` |
| **Alibaba Qwen** | `rco config set RCO_AI_PROVIDER=dashscope RCO_API_KEY=... RCO_API_URL=https://dashscope.aliyuncs.com/compatible-mode/v1 RCO_MODEL=qwen3-coder-32b-instruct` |
| **Together AI** | `rco config set RCO_AI_PROVIDER=together RCO_API_KEY=...` |
| **DeepInfra** | `rco config set RCO_AI_PROVIDER=deepinfra RCO_API_KEY=...` |

<details>
<summary>🔒 Secure Storage</summary>

When built with `--features secure-storage`:
- **macOS**: Keychain
- **Linux**: Secret Service (GNOME Keyring, KWallet, KeePassXC)
- **Windows**: Credential Manager

Falls back to config file if keychain is unavailable.

</details>

---

## ⚙️ Configuration

### Configuration Priority

```
Per-repo config > Global config > Environment variables > Defaults
```

### Quick Config Commands

```bash
rco config status                          # Check secure storage status
rco config set RCO_AI_PROVIDER=anthropic   # Set provider
rco config set RCO_MODEL=claude-3-5-haiku  # Set model
rco config set RCO_COMMIT_TYPE=conventional # conventional or gitmoji
rco config set RCO_EMOJI=true              # Include emojis
rco config set RCO_LANGUAGE=en             # Output language
rco config get RCO_AI_PROVIDER             # Get current value
rco config describe                        # Show all options
rco config reset --all                     # Reset to defaults
```

### Common Options

| Key | Description | Default |
|-----|-------------|---------|
| `RCO_AI_PROVIDER` | AI backend | `anthropic` |
| `RCO_MODEL` | Model name | Provider-specific |
| `RCO_API_KEY` | API key | - |
| `RCO_API_URL` | Custom endpoint | - |
| `RCO_COMMIT_TYPE` | Commit format | `conventional` |
| `RCO_EMOJI` | Include emojis | `false` |
| `RCO_LANGUAGE` | Output language | `en` |
| `RCO_MAX_TOKENS` | Max response tokens | `1024` |
| `RCO_TEMPERATURE` | Response creativity | `0.7` |

---

## 🎣 Git Hooks

### Install/Uninstall

```bash
rco hook set    # Install prepare-commit-msg hook
rco hook unset  # Remove hook
```

Once installed, `git commit` (without `-m`) automatically generates commit messages!

### Advanced Hooks

```bash
# Pre-generation hook
rco config set RCO_PRE_GEN_HOOK="just lint; just test -q"

# Pre-commit hook
rco config set RCO_PRE_COMMIT_HOOK="./scripts/tweak_commit.sh"

# Post-commit hook
rco config set RCO_POST_COMMIT_HOOK="git push"

# Hook behavior
rco config set RCO_HOOK_STRICT=false       # Allow hook failures
rco config set RCO_HOOK_TIMEOUT_MS=60000   # Timeout in ms
```

### Skip Hooks (Per-Run)

```bash
rco --no-pre-hooks      # Skip pre-gen + pre-commit hooks
rco --no-post-hooks     # Skip post-commit hooks
```

---

## 🔌 MCP Server

Rusty Commit includes an **MCP (Model Context Protocol)** server for editor integrations.

### Start the Server

```bash
# TCP Mode (Cursor, VS Code)
rco mcp server --port 3000

# STDIO Mode (Direct integration)
rco mcp stdio
```

### Editor Setup

**Cursor:**
```
Settings > Features > MCP > Add Server
Type: HTTP
URL: http://localhost:3000
```

**Claude Code:**
```bash
rco mcp stdio | claude-code connect stdio
```

### Available MCP Tools

- `generate_commit` - Generate commit from staged changes
- `get_config` / `set_config` - Manage configuration
- `list_accounts` / `use_account` - Switch accounts

---

## 🔄 Multi-Account Support

Perfect for switching between work and personal projects!

```bash
# Add accounts
rco config add-provider                    # Interactive wizard
rco config add-provider --provider openai --alias work-openai

# Manage accounts
rco config list-accounts                   # Show all accounts
rco config show-account work-openai        # Show account details
rco config use-account work-openai         # Switch to account
rco config remove-account work-openai      # Remove account
```

### Account Configuration

Accounts stored in `~/.config/rustycommit/accounts.toml`:

```toml
[accounts.work-openai]
provider = "openai"
model = "gpt-4o-mini"

[accounts.personal-anthropic]
provider = "anthropic"
model = "claude-3-5-haiku-20241022"

[active_account]
alias = "work-openai"
```

---

## 📋 PR Description Generation

```bash
rco pr generate             # Generate PR description for current branch
rco pr generate --base main # Compare against main branch
rco pr browse               # Generate and open PR creation page
```

Features:
- Analyzes all commits since base branch
- Groups changes by type (features, fixes, docs, etc.)
- Follows PR template conventions
- Includes breaking changes and deprecation notices

---

## 🚫 File Exclusion

Exclude files from AI analysis via `.rcoignore`:

```gitignore
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

# OS
.DS_Store
Thumbs.db
```

Or via command line:

```bash
rco -x "package-lock.json" -x "*.lock" -x "*.min.js"
```

---

## 🔄 Updates

```bash
rco update --check          # Check for updates
rco update                  # Update to latest
rco update --force          # Force update
rco update --version 1.0.2  # Install specific version
```

---

## 🏃 GitHub Action

Use in CI/CD workflows:

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
        with:
          fetch-depth: 0
      - uses: hongkongkiwi/action-rusty-commit@v1
        with:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          RCO_API_KEY: ${{ secrets.RCO_API_KEY }}
          RCO_AI_PROVIDER: 'anthropic'
          RCO_MODEL: 'claude-3-5-haiku-20241022'
```

See [action-rusty-commit](https://github.com/hongkongkiwi/action-rusty-commit) for details.

---

## 🛠️ Troubleshooting

| Issue | Solution |
|-------|----------|
| **401 / Invalid API key** | Re-authenticate: `rco auth login` or set `RCO_API_KEY` |
| **429 Rate-limited** | Wait briefly; try lighter model or switch accounts |
| **Secure storage unavailable** | Falls back to file; check `rco config status` |
| **Hooks not running** | Ensure `.git/hooks/prepare-commit-msg` is executable |
| **OAuth browser not opening** | Use `rco auth login --no-browser` |
| **Large diff truncated** | Increase `RCO_DIFF_TOKENS` or use `.rcoignore` |
| **Reasoning model thinking tags** | Use `--strip-thinking` flag |

### Debug Mode

```bash
RUST_LOG=debug rco -y       # Enable debug logging
rco --show-prompt           # See exact prompt sent to AI
rco --timing                # Show timing breakdown
```

---

## 🧪 Development

```bash
# Build
cargo build
cargo build --release
cargo build --features secure-storage

# Test
cargo test
cargo test --all-features

# Lint
cargo clippy --all-features -- -D warnings
cargo fmt

# Just commands
just build
just test
just all    # fmt + clippy + test
```

---

## 🔒 Security & Verification

All releases are cryptographically signed with multiple methods:

### Automatic Verification

The install script automatically verifies downloads using the strongest available method.

### Manual Verification

```bash
# Cosign/Sigstore (modern)
cosign verify-blob \
  --bundle rustycommit-linux-x86_64.tar.gz.cosign.bundle \
  --certificate-identity-regexp "https://github.com/hongkongkiwi/rusty-commit/.github/workflows/release.yml@.*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  rustycommit-linux-x86_64.tar.gz

# GPG (traditional)
gpg --keyserver hkps://keys.openpgp.org --recv-keys 0EC2DFF577818B86BA38DA3F164E3F90E425B2AD
gpg --verify rustycommit-linux-x86_64.tar.gz.asc rustycommit-linux-x86_64.tar.gz

# GitHub attestations
gh attestation verify rustycommit-linux-x86_64.tar.gz --repo hongkongkiwi/rusty-commit
```

See [docs/VERIFICATION.md](docs/VERIFICATION.md) for details.

---

## 💖 Support the Project

If Rusty Commit saves you time, consider supporting ongoing development:

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-@hongkongkiwi-fd2e83?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/hongkongkiwi)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-ffdd00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/hongkongkiwi)

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

<div align="center">

**Inspired by [OpenCommit](https://github.com/di-sukharev/opencommit)** · Built with 🦀 Rust

**Build faster. Commit smarter.**

</div>
