<div align="center">

# 🦀 Rusty Commit (`rco`)

### **AI-powered commit message generator written in Rust**

**Generate conventional commits, GitMoji messages, and PR descriptions using 100+ AI providers**

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rusty-commit.svg)](https://crates.io/crates/rusty-commit)
[![Documentation](https://docs.rs/rusty-commit/badge.svg)](https://docs.rs/rusty-commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**🚀 Fast · 🔒 Secure · 🏠 Local-first · 🔌 Editor integrations via MCP**

[Installation](#-installation) · [Quick Start](#-quick-start) · [Features](#-features) · [Providers](#-providers) · [Configuration](#-configuration)

</div>

---

## ✨ Why Rusty Commit?

<table>
<tr>
<td width="50%">

**⚡ Blazing Fast**

Native Rust binary with instant startup time. No Node.js bloat, no waiting.

**🤖 100+ AI Providers**

Works with OpenAI, Claude, Groq, Cerebras, DeepSeek, GitHub Copilot, Ollama, and 90+ more.

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
<summary>🔐 Security-conscious? Verify first</summary>

```bash
# Download and inspect
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh -o install.sh
# Verify with SHA256 checksums, Cosign, GPG, or GitHub attestations
# See: docs/INSTALL-SCRIPT-VERIFICATION.md
```

**Verify release binaries with GPG:**

```bash
# Download release files
wget https://github.com/hongkongkiwi/rusty-commit/releases/download/v1.0.0/rusty-commit-v1.0.0-x86_64-unknown-linux-musl.tar.gz
wget https://github.com/hongkongkiwi/rusty-commit/releases/download/v1.0.0/rusty-commit-v1.0.0-x86_64-unknown-linux-musl.tar.gz.asc
wget https://github.com/hongkongkiwi/rusty-commit/releases/download/v1.0.0/SHA256SUMS.txt
wget https://github.com/hongkongkiwi/rusty-commit/releases/download/v1.0.0/SHA256SUMS.txt.asc

# Verify GPG signature
gpg --verify SHA256SUMS.txt.asc

# Verify checksums
sha256sum -c SHA256SUMS.txt
```

Or verify individual files directly:

```bash
gpg --verify rusty-commit-v1.0.0-x86_64-unknown-linux-musl.tar.gz.asc
```

**GPG Public Key:** [`0x0EC2DFF577818B86`](https://keys.openpgp.org/search?q=0x0EC2DFF577818B86) (full: `0EC2DFF577818B86BA38DA3F164E3F90E425B2AD`)

</details>

<details>
<summary>📦 Package Managers</summary>

| Platform | Command | Repo |
|----------|---------|------|
| **Homebrew** | `brew tap hongkongkiwi/rusty-commit && brew install rusty-commit` | [homebrew-rusty-commit](https://github.com/hongkongkiwi/homebrew-rusty-commit) |
| **Cargo** | `cargo install rusty-commit --features secure-storage` | - |
| **Debian/Ubuntu** | `wget .../rusty-commit_amd64.deb && sudo dpkg -i rusty-commit_amd64.deb` | - |
| **Fedora/RHEL** | `sudo dnf install https://.../rusty-commit.x86_64.rpm` | - |
| **Alpine** | `wget .../rusty-commit-x86_64.apk && sudo apk add --allow-untrusted rusty-commit-x86_64.apk` | - |
| **Arch Linux (AUR)** | `yay -S rusty-commit` or `paru -S rusty-commit` | Community |
| **Nix/NixOS** | `nix-env -iA nixpkgs.rusty-commit` or via flake | [nixpkgs-overlays](https://github.com/hongkongkiwi/nixpkgs-overlays) |
| **Windows (Scoop)** | `scoop bucket add rusty-commit && scoop install rusty-commit` | [scoop-rusty-commit](https://github.com/hongkongkiwi/scoop-rusty-commit) |
| **Windows (Winget)** | `winget install hongkongkiwi.rusty-commit` | Community (via [winget-pkgs](https://github.com/microsoft/winget-pkgs)) |
| **Windows (Chocolatey)** | `choco install rusty-commit` | Community |
| **Windows (Binary)** | [Download from releases](https://github.com/hongkongkiwi/rusty-commit/releases) | - |

</details>

---

## 🚀 Quick Start

### Interactive Setup

```bash
rco setup              # Quick interactive wizard (recommended)
rco setup --advanced   # Full configuration
rco setup --defaults   # Non-interactive with sensible defaults
```

**Quick Setup** asks for:
1. **AI Provider** - Choose from 30+ providers (100+ total supported)
2. **API Key** - Securely stored in your system's keychain
3. **Commit Format** - Conventional commits, GitMoji, or simple

### Generate Your First Commit

```bash
git add .
rco                    # Interactive mode with review
```

<details>
<summary>⚡ Common Options</summary>

```bash
rco --dry-run          # Preview without committing
rco --edit             # Open in $EDITOR before committing
rco --clipboard        # Copy to clipboard instead
rco --generate 3       # Generate 3 variations
rco --fgm              # Full GitMoji specification
rco -y                 # Auto-commit without confirmation
rco -c "context"       # Add extra context
```

</details>

<details>
<summary>🔄 Multi-Account Workflow</summary>

```bash
# Add multiple provider accounts
rco config add-provider                    # Interactive wizard
rco config add-provider --provider openai --alias work-openai

# Switch between them
rco config use-account work-openai
git add . && rco

rco config use-account personal-anthropic
git add . && rco
```

</details>

---

## 🎯 Features

| Feature | Command |
|---------|---------|
| **Interactive mode** | `rco` |
| **Auto-commit** | `rco -y` |
| **Dry-run preview** | `rco --dry-run` |
| **Edit in $EDITOR** | `rco --edit` |
| **Copy to clipboard** | `rco --clipboard` |
| **Generate variations** | `rco -g 3` |
| **Add context** | `rco -c "Fix OAuth"` |
| **Full GitMoji** | `rco --fgm` |
| **Show prompt** | `rco --show-prompt` |
| **Debug logging** | `RUST_LOG=debug rco` |

---

## 🤖 Providers

Rusty Commit supports **100+ AI providers**. Configure interactively with `rco setup` or manually:

### 🔑 OAuth (No API Key)

| Provider | Command |
|----------|---------|
| **Claude (Anthropic)** | `rco auth login` |
| **GitHub Copilot** | `rco auth login --provider github-copilot` |

### 🔐 API Key Providers

<details>
<summary>🌟 Popular Providers</summary>

| Provider | Setup |
|----------|-------|
| **OpenAI** | `rco config set RCO_AI_PROVIDER=openai RCO_API_KEY=sk-... RCO_MODEL=gpt-4o-mini` |
| **Anthropic** | `rco config set RCO_AI_PROVIDER=anthropic RCO_API_KEY=sk-ant-... RCO_MODEL=claude-3-5-haiku-20241022` |
| **Google Gemini** | `rco config set RCO_AI_PROVIDER=gemini RCO_API_KEY=... RCO_MODEL=gemini-2.5-flash` |
| **xAI/Grok** | `rco config set RCO_AI_PROVIDER=xai RCO_API_KEY=... RCO_MODEL=grok-2` |
| **DeepSeek** | `rco config set RCO_AI_PROVIDER=deepseek RCO_API_KEY=sk-... RCO_MODEL=deepseek-chat` |

</details>

<details>
<summary>⚡ Ultra-Fast Inference</summary>

| Provider | Setup |
|----------|-------|
| **Groq** | `rco config set RCO_AI_PROVIDER=groq RCO_API_KEY=gsk_... RCO_MODEL=llama-3.3-70b-versatile` |
| **Cerebras** | `rco config set RCO_AI_PROVIDER=cerebras RCO_API_KEY=... RCO_MODEL=llama-3.3-70b` |
| **SambaNova** | `rco config set RCO_AI_PROVIDER=sambanova RCO_API_KEY=... RCO_MODEL=Meta-Llama-3.3-70B-Instruct` |
| **Nebius** | `rco config set RCO_AI_PROVIDER=nebius RCO_API_KEY=...` |

</details>

<details>
<summary>🌐 Multi-Model Aggregators</summary>

| Provider | Setup |
|----------|-------|
| **OpenRouter** | `rco config set RCO_AI_PROVIDER=openrouter RCO_API_KEY=sk-or-...` |
| **Together AI** | `rco config set RCO_AI_PROVIDER=together RCO_API_KEY=...` |
| **Fireworks** | `rco config set RCO_AI_PROVIDER=fireworks RCO_API_KEY=...` |
| **Replicate** | `rco config set RCO_AI_PROVIDER=replicate RCO_API_KEY=...` |
| **DeepInfra** | `rco config set RCO_AI_PROVIDER=deepinfra RCO_API_KEY=...` |
| **Novita** | `rco config set RCO_AI_PROVIDER=novita RCO_API_KEY=...` |

</details>

<details>
<summary>🏢 Enterprise Providers</summary>

| Provider | Setup |
|----------|-------|
| **Azure OpenAI** | `rco config set RCO_AI_PROVIDER=azure RCO_API_KEY=... RCO_API_URL=https://<resource>.openai.azure.com` |
| **AWS Bedrock** | `rco config set RCO_AI_PROVIDER=bedrock` |
| **Google Vertex AI** | `rco config set RCO_AI_PROVIDER=vertex` |
| **Mistral** | `rco config set RCO_AI_PROVIDER=mistral RCO_API_KEY=... RCO_MODEL=mistral-small-latest` |
| **Cohere** | `rco config set RCO_AI_PROVIDER=cohere RCO_API_KEY=... RCO_MODEL=command-r` |
| **AI21 Labs** | `rco config set RCO_AI_PROVIDER=ai21 RCO_API_KEY=... RCO_MODEL=jamba-1.5-mini` |
| **Perplexity** | `rco config set RCO_AI_PROVIDER=perplexity RCO_API_KEY=...` |

</details>

<details>
<summary>🏠 Local/Self-Hosted</summary>

| Provider | Setup |
|----------|-------|
| **Ollama** | `rco config set RCO_AI_PROVIDER=ollama RCO_MODEL=llama3.2 RCO_API_URL=http://localhost:11434` |
| **LM Studio** | `rco config set RCO_AI_PROVIDER=lmstudio RCO_API_URL=http://localhost:1234/v1` |
| **llama.cpp** | `rco config set RCO_AI_PROVIDER=llamacpp RCO_API_URL=http://localhost:8080/v1` |

</details>

<details>
<summary>🌏 China-based Providers</summary>

| Provider | Setup |
|----------|-------|
| **Moonshot/Kimi** | `rco config set RCO_AI_PROVIDER=moonshot RCO_API_KEY=...` |
| **SiliconFlow** | `rco config set RCO_AI_PROVIDER=siliconflow RCO_API_KEY=...` |
| **Zhipu AI** | `rco config set RCO_AI_PROVIDER=zhipu RCO_API_KEY=...` |
| **MiniMax** | `rco config set RCO_AI_PROVIDER=minimax RCO_API_KEY=...` |
| **Alibaba Qwen** | `rco config set RCO_AI_PROVIDER=dashscope RCO_API_KEY=...` |

</details>

<details>
<summary>📋 All 100+ Supported Providers</summary>

**GPU Cloud & Inference:**
Cerebras, SambaNova, Nebius, Lambda, Hyperbolic, Kluster, Together, Fireworks, Replicate, Novita, Predibase, TensorOps, Baseten, Chutes, IO.Net, Scaleway, OVHcloud, Friendli, ModelScope

**Enterprise & Specialized:**
Cohere, AI21 Labs, Upstage/Solar, Jina AI, Abacus AI, Bailing, Poe

**AI Gateways & Proxies:**
Helicone, Cloudflare Workers AI, Vercel AI Gateway, Requesty

**China-based:**
Moonshot, SiliconFlow, Zhipu, MiniMax, Baichuan, 01.AI, Dashscope/Alibaba

**Local/Self-hosted:**
Ollama, LM Studio, llama.cpp, KoboldCpp, Text Generation WebUI, Tabby

**Additional Providers:**
Venice, Cortecs, Synthetic, NanoGPT, ZenMux, V0, Morph, Corcel, CyberNative, Edgen, GigaChat, Hydra, Lingyi, Monica, Pollinations, ShuttleAI, Teknium, TheB, TryLeap, Targon, 302.AI, SAP AI Core

</details>

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

### Interactive Configuration

```bash
rco setup              # Quick setup (essential settings)
rco setup --advanced   # Advanced setup (all settings)
rco setup --tui        # Force TUI mode (interactive terminal UI)
rco setup --no-tui     # Force dialoguer prompts (no TUI)
```

The setup wizard automatically detects whether to use TUI mode based on your terminal. Use `--tui` or `--no-tui` to override.

**TUI Features:**
- Arrow key navigation through menus
- Real-time preview of commit message formats
- Visual confirmation of selections
- Color-coded provider categories

### Configuration Priority

```
Per-repo config > Global config > Environment variables > Defaults
```

### Manual Config Commands

```bash
rco config status                          # Check secure storage status
rco config set RCO_AI_PROVIDER=anthropic   # Set provider
rco config set RCO_MODEL=claude-3-5-haiku  # Set model
rco config get RCO_AI_PROVIDER             # Get current value
rco config describe                        # Show all options
rco config reset --all                     # Reset to defaults
```

<details>
<summary>🔧 Common Configuration Options</summary>

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
| `RCO_ENABLE_COMMIT_BODY` | Add commit body | `false` |
| `RCO_LEARN_FROM_HISTORY` | Learn from git history | `false` |

</details>

---

## 🎣 Git Hooks

### Install/Uninstall

```bash
rco hook set    # Install prepare-commit-msg hook
rco hook unset  # Remove hook
```

Once installed, `git commit` (without `-m`) automatically generates commit messages!

<details>
<summary>⚙️ Advanced Hooks</summary>

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

**Skip Hooks (Per-Run):**
```bash
rco --no-pre-hooks      # Skip pre-gen + pre-commit hooks
rco --no-post-hooks     # Skip post-commit hooks
```

</details>

---

## 🧠 Advanced Features

<details>
<summary>📝 Commit Body Generation</summary>

Enable detailed commit messages with body explaining the "why":

```bash
rco config set RCO_ENABLE_COMMIT_BODY=true
```

**Output:**
```
feat(auth): implement OAuth2 PKCE flow

- Add secure token storage with automatic refresh
- Support GitHub, GitLab, and generic OAuth2 providers
- Handle token expiration gracefully
```

</details>

<details>
<summary>🧠 Style Learning from History</summary>

Automatically learn and match your team's commit style:

```bash
rco config set RCO_LEARN_FROM_HISTORY=true
```

Analyzes last 50 commits to detect:
- Common commit types and scopes
- Average description length
- Capitalization preferences
- Gitmoji usage patterns

</details>

<details>
<summary>😄 GitMoji Support</summary>

25+ emojis from [gitmoji.dev](https://gitmoji.dev):

| Emoji | Meaning | Use Case |
|-------|---------|----------|
| ✨ | `:sparkles:` | New feature |
| 🐛 | `:bug:` | Bug fix |
| 📝 | `:memo:` | Documentation |
| 🎨 | `:art:` | Code structure/format |
| ♻️ | `:recycle:` | Refactoring |
| ⚡ | `:zap:` | Performance |
| ✅ | `:white_check_mark:` | Tests |
| 🔒 | `:lock:` | Security fix |
| ⬆️ | `:arrow_up:` | Upgrade dependencies |
| 🔥 | `:fire:` | Remove code/files |
| 🚀 | `:rocket:` | Deployment |
| 💥 | `:boom:` | Breaking changes |

</details>

<details>
<summary>📂 Repository Context Awareness</summary>

Rusty Commit automatically detects project context:

```bash
# Create custom context file
echo "Payment processing microservice" > .rco/context.txt
```

**Auto-detected sources:**
- `.rco/context.txt` - Custom project description
- `README.md` - First paragraph
- `Cargo.toml` - Rust project description
- `package.json` - Node.js project description

</details>

---

## 🎨 Custom Skills

Create reusable commit message templates:

```bash
rco skills list                           # List available skills
rco skills create my-template             # Create a new skill
rco skills create my-team-style --project # Project-specific skill
rco skills show my-team-style             # Show skill details
rco skills remove my-team-style           # Remove a skill
rco --skill my-team-template              # Use a skill
```

<details>
<summary>📥 Import Skills from External Sources</summary>

```bash
# Import from Claude Code
rco skills available                      # List Claude Code skills
rco skills import claude-code --name <skill-name>

# Import from GitHub
rco skills import github:owner/repo

# Import from GitHub Gist
rco skills import gist:<gist-id>

# Import from URL
rco skills import https://example.com/skill-definition.md
```

</details>

<details>
<summary>📝 Custom Prompt Template</summary>

Create `~/.config/rustycommit/skills/my-skill/prompt.md`:

```markdown
# Custom Commit Prompt

Analyze this {language} code change:

{diff}

Context: {context}
Format: {commit_type}
Max length: {max_length}

Generate a commit message following our team conventions:
- Use present tense
- Include ticket number if obvious from branch name
```

</details>

---

## 🔌 MCP Server

Rusty Commit includes an **MCP (Model Context Protocol)** server for editor integrations.

```bash
rco mcp server --port 3000   # TCP Mode (Cursor, VS Code)
rco mcp stdio                # STDIO Mode (Direct integration)
```

**Cursor:** `Settings > Features > MCP > Add Server` → Type: HTTP → URL: `http://localhost:3000`

**Claude Code:** `rco mcp stdio | claude-code connect stdio`

---

## 📋 PR Description Generation

```bash
rco pr generate             # Generate PR description for current branch
rco pr generate --base main # Compare against main branch
rco pr browse               # Generate and open PR creation page
```

---

## 🚫 File Exclusion

Exclude files from AI analysis via `.rcoignore`:

```gitignore
# Dependencies
node_modules/
vendor/

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
```

Or via command line: `rco -x "*.lock" -x "*.min.js"`

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

```yaml
- uses: hongkongkiwi/rusty-commit@v1
  with:
    provider: 'anthropic'
    api-key: ${{ secrets.ANTHROPIC_API_KEY }}
    auto-commit: 'true'
```

---

## 🤝 Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and contribution guidelines.

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

---

<div align="center">

**[⬆ Back to Top](#-rusty-commit-rco)**

Made with 🦀 by the Rusty Commit Contributors

</div>
