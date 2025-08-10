# Rusty Commit (rco)

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
<!-- Uncomment these badges once published to crates.io:
[![Crates.io](https://img.shields.io/crates/v/rusty-commit.svg)](https://crates.io/crates/rusty-commit)
[![Documentation](https://docs.rs/rusty-commit/badge.svg)](https://docs.rs/rusty-commit)
-->
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

🚀 Write great commit messages in seconds — fast, local, and secure. 🦀

`rco` is a Rust-native, AI‑powered commit assistant.

### 🌟 Why Rusty Commit
- **Speed**: Native Rust binary with instant startup
- **Choice**: Works with 16+ AI providers (OpenAI, Anthropic/Claude, OpenRouter, Groq, DeepSeek, GitHub Copilot, Ollama, Fireworks, Moonshot/Kimi, Alibaba DashScope/Qwen…)
- **Secure**: Optional keychain storage via `--features secure-storage`
- **Flexible**: Conventional commits, GitMoji, templates, multi‑language
- **Integrated**: Git hooks, GitHub Actions, MCP server for editors

## Contents
- [Installation](#installation)
- [Quick start](#quick-start)
- [Examples](#examples)
- [Configuration](#configuration)
- [Providers](#providers)
- [CLI overview](#cli-overview)
- [Git hooks](#git-hooks)
- [Updates](#updates)
- [GitHub Action](#github-action-minimal)
- [Advanced](#advanced)
- [Troubleshooting](#troubleshooting)
- [Uninstall](#uninstall)
- [Compatibility](#compatibility)
- [Development](#development)
- [License](#license)
- [Credits](#credits)

## Installation

### One‑liner (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

**🔐 Security-conscious users:** [Verify the install script](docs/INSTALL-SCRIPT-VERIFICATION.md) before running it.

The script auto‑detects your platform and installs via Homebrew, .deb/.rpm, Cargo, or binary.
**All packages are cryptographically signed and verified automatically:**
- 🔐 Cosign/Sigstore signatures (keyless, modern)
- 🔑 GPG signatures (traditional)
- ✅ SHA256 checksums
- 📋 GitHub build attestations

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

## Quick start
```bash
# 1) Authenticate (Claude OAuth) or set an API key
rco auth login
# or
rco config set RCO_API_KEY=sk-...

# 2) Generate a commit
git add .
rco
```

Useful flags:
```bash
rco -c "Fix login bug"     # extra context
rco --fgm                   # full GitMoji
rco -y                      # skip confirmation
rco --show-prompt           # print the AI prompt only
```

## Examples
Conventional commit example:
```text
feat(auth): fix token refresh edge-case

Handle clock-skew by allowing ±60s leeway during token expiry checks. Adds retry on 429 and surfaces actionable errors.
```

GitMoji example (with --fgm or RCO_COMMIT_TYPE=gitmoji):
```text
✨ auth: robust token refresh with retry

Allow ±60s clock-skew; add backoff on 429; improve error messages for invalid credentials.
```

## Configuration
- **Global**: `~/.config/rustycommit/config.{toml,json}`
- **Per‑repo**: `.rustycommit.toml` / `.rco.toml`

Basics:
```bash
rco config status                          # secure storage status
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_MODEL=claude-3-5-haiku-20241022
rco config set RCO_COMMIT_TYPE=conventional
rco config set RCO_EMOJI=true
rco config get RCO_AI_PROVIDER
rco config reset --all
```

Common keys (compact):

| Key | What it does | Example |
| --- | ------------- | ------- |
| `RCO_AI_PROVIDER` | Which AI backend to use | `anthropic`, `openai`, `openrouter`, `groq`, `ollama`, `github-copilot`, ... |
| `RCO_MODEL` | Model name for the provider | `claude-3-5-haiku-20241022`, `gpt-4o-mini`, `llama-3.1-70b-versatile` |
| `RCO_API_KEY` | API key if required | `sk-...`, `gsk_...`, etc. |
| `RCO_API_URL` | Custom endpoint (e.g., Ollama) | `http://localhost:11434` |
| `RCO_COMMIT_TYPE` | Commit format | `conventional` or `gitmoji` |
| `RCO_EMOJI` | Emojis in messages | `true` / `false` |
| `RCO_LANGUAGE` | Output language | `en`, `es`, `fr`, ... |

Tip: You can set multiple values at once:
```bash
rco config set RCO_AI_PROVIDER=anthropic RCO_MODEL=claude-3-5-haiku-20241022 RCO_EMOJI=true
```

## Providers
Works with 16+ providers. Examples:
- **Claude (OAuth)**: `rco auth login`
- **OpenAI / OpenRouter / Groq / DeepSeek / GitHub Copilot**: `rco auth login` or `rco config set RCO_API_KEY=...`
- **Ollama (local)**:
  ```bash
  rco config set RCO_AI_PROVIDER=ollama
  rco config set RCO_MODEL=mistral
  # Remote Ollama:
  rco config set RCO_API_URL=http://localhost:11434
  ```

Security & storage (optional `secure-storage` feature):
- macOS: Keychain
- Linux: Secret Service (GNOME Keyring / KWallet / KeePassXC)
- Windows: Credential Manager
- Automatic fallback to config file if unavailable

### Provider‑specific examples

OpenAI:
```bash
rco config set RCO_AI_PROVIDER=openai
rco config set RCO_API_KEY=sk-...
rco config set RCO_MODEL=gpt-4o-mini
# Optional custom endpoint:
# rco config set RCO_API_URL=https://api.openai.com/v1
# Get API key: https://platform.openai.com/api-keys
```

Anthropic (Claude):
```bash
# OAuth (recommended)
rco auth login
# Or API key
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_API_KEY=sk-ant-...
rco config set RCO_MODEL=claude-3-5-haiku-20241022
# API docs & keys: https://console.anthropic.com/settings/keys
```

OpenRouter:
```bash
rco config set RCO_AI_PROVIDER=openrouter
rco config set RCO_API_KEY=sk-or-...
rco config set RCO_API_URL=https://openrouter.ai/api/v1
rco config set RCO_MODEL=openai/gpt-4o-mini
# Keys: https://openrouter.ai/keys
```

Groq:
```bash
rco config set RCO_AI_PROVIDER=groq
rco config set RCO_API_KEY=gsk_...
rco config set RCO_API_URL=https://api.groq.com/openai/v1
rco config set RCO_MODEL=llama-3.1-70b-versatile
# Keys: https://console.groq.com/keys
```

DeepSeek:
```bash
rco config set RCO_AI_PROVIDER=deepseek
rco config set RCO_API_KEY=sk-...
rco config set RCO_API_URL=https://api.deepseek.com/v1
rco config set RCO_MODEL=deepseek-chat
# Keys: https://platform.deepseek.com/api-keys
```

Together AI:
```bash
rco config set RCO_AI_PROVIDER=together
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.together.xyz/v1
rco config set RCO_MODEL=meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo
# Keys: https://api.together.xyz/settings/api-keys
```

DeepInfra:
```bash
rco config set RCO_AI_PROVIDER=deepinfra
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.deepinfra.com/v1/openai
rco config set RCO_MODEL=meta-llama/Meta-Llama-3-70B-Instruct
# Keys: https://deepinfra.com/dash/api_keys
```

Mistral AI:
```bash
rco config set RCO_AI_PROVIDER=mistral
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.mistral.ai/v1
rco config set RCO_MODEL=mistral-small-latest
# Keys: https://console.mistral.ai/api-keys
```

Azure OpenAI:
```bash
rco config set RCO_AI_PROVIDER=azure
rco config set RCO_API_KEY=<azure_api_key>
rco config set RCO_API_URL=https://<your-resource>.openai.azure.com
# Use your deployment name, not the model name
rco config set RCO_MODEL=<deployment-name>
# Docs: https://learn.microsoft.com/azure/ai-services/openai/how-to/create-resource
```

Google Gemini:
```bash
rco config set RCO_AI_PROVIDER=gemini
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=gemini-pro
# Keys: https://aistudio.google.com/app/apikey
```

Perplexity:
```bash
rco config set RCO_AI_PROVIDER=perplexity
rco config set RCO_API_KEY=...
# Optional: custom endpoint
# rco config set RCO_API_URL=https://api.perplexity.ai/chat/completions
rco config set RCO_MODEL=llama-3.1-sonar-small-128k-online
# Keys: https://www.perplexity.ai/settings/api
```

Fireworks AI:
```bash
rco config set RCO_AI_PROVIDER=fireworks
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.fireworks.ai/inference/v1
rco config set RCO_MODEL=accounts/fireworks/models/llama-v3p1-70b-instruct
# Keys: https://app.fireworks.ai/users/api-keys
```

Moonshot AI (Kimi):
```bash
rco config set RCO_AI_PROVIDER=moonshot
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.moonshot.cn/v1
rco config set RCO_MODEL=kimi-k2
# Docs & keys: https://platform.moonshot.ai/docs/introduction#text-generation-model
```

Alibaba Model Studio (DashScope / Qwen Coder):
```bash
rco config set RCO_AI_PROVIDER=dashscope
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
rco config set RCO_MODEL=qwen3-coder-32b-instruct
# Docs: https://www.alibabacloud.com/help/en/model-studio/qwen-coder
# Keys: https://dashscope.console.aliyun.com/apiKey
```

Vertex AI (Google Cloud):
```bash
rco config set RCO_AI_PROVIDER=vertex
# Set up a service that proxies to Vertex’s OpenAI-compatible endpoint or use a gateway
# Example placeholder (requires gateway):
rco config set RCO_API_URL=https://<your-gateway>/v1
rco config set RCO_MODEL=google/gemini-1.5-pro
# Getting started: https://cloud.google.com/vertex-ai/docs/generative-ai/start/quickstarts
```

## Git hooks
```bash
rco hook set    # install prepare-commit-msg hook
rco hook unset  # uninstall
```

### Optional pre/post hooks (advanced)
Disabled by default. If you want to run custom commands around commit generation, set these keys (globally or per‑repo). Hooks run in your shell and support strict mode and timeouts.

Config keys:
- `RCO_PRE_GEN_HOOK`: commands before message generation
- `RCO_PRE_COMMIT_HOOK`: commands after generation; may edit the message via `RCO_COMMIT_FILE`
- `RCO_POST_COMMIT_HOOK`: commands after `git commit`
- `RCO_HOOK_STRICT` (default `true`), `RCO_HOOK_TIMEOUT_MS` (default `30000`)

Examples:
```bash
# Run lint and tests before generating the message
rco config set RCO_PRE_GEN_HOOK="just lint; just test -q"

# Allow a script to edit the commit message before committing
rco config set RCO_PRE_COMMIT_HOOK="./scripts/tweak_commit.sh"

# Push after committing
rco config set RCO_POST_COMMIT_HOOK="git push"

# Looser behavior with longer timeout
rco config set RCO_HOOK_STRICT=false
rco config set RCO_HOOK_TIMEOUT_MS=60000

# Per‑run disable
rco --no-pre-hooks      # skip pre-gen + pre-commit hooks
rco --no-post-hooks     # skip post-commit hooks
```

Hooks receive environment variables:
- `RCO_REPO_ROOT`, `RCO_PROVIDER`, `RCO_MODEL`
- `RCO_MAX_TOKENS`, `RCO_DIFF_TOKENS`, `RCO_CONTEXT` (pre‑gen)
- `RCO_COMMIT_MESSAGE`, `RCO_COMMIT_FILE` (pre‑commit and post‑commit)

## Updates
```bash
rco update --check   # see if a new version is available
rco update           # update using your install method
```

### Architectures
Prebuilt archives and packages are provided for:
- Linux: x86_64 (gnu, musl), aarch64 (gnu, musl), armv7 (gnueabihf), riscv64 (gnu, musl)
- macOS: x86_64, aarch64
- Windows: x86_64, i686

## GitHub Action (minimal)
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

## Advanced
- **MCP server** (for editors like Cursor): `rco mcp server --port 3000` or `rco mcp stdio`
- **Commitlint config**: `rco commitlint`

## CLI overview
Subcommands:
- `config` — set/get/reset values and check secure storage status
- `hook` — install/uninstall git hooks
- `commitlint` — generate commitlint configuration
- `auth` — login/logout/status for OAuth (e.g., Claude)
- `mcp` — run MCP server over TCP or stdio
- `update` — check and install updates (supports Homebrew, Cargo, .deb/.rpm, binary, Snap)

Global flags you can use with the default `rco` command:
```text
--fgm                 Use full GitMoji specification
-c, --context <TEXT>  Additional context for the commit
-y, --yes             Skip confirmation
    --show-prompt     Print the AI prompt and exit
```

## Troubleshooting
- **401 / Invalid API key**: Re‑authenticate (`rco auth login`) or set a valid `RCO_API_KEY`.
- **Rate‑limited (429)**: Wait briefly; try a lighter model or another provider.
- **Secure storage unavailable**: We automatically fall back to file storage; check `rco config status`.
- **Hooks not running**: Ensure `.git/hooks/prepare-commit-msg` exists and is executable. Re‑install via `rco hook set`.
- **Windows PATH issues**: Add the install dir (e.g., `%USERPROFILE%\\.cargo\\bin`) to PATH.
- **Corporate proxy**: Set `HTTP_PROXY`/`HTTPS_PROXY` environment variables.

## Uninstall
- Homebrew: `brew uninstall rusty-commit`
- Cargo: `cargo uninstall rusty-commit`
- Remove config: delete `~/.config/rustycommit/`

## Compatibility
- Works with per‑repo overrides and multiple providers.

## Development
```bash
cargo build        # build
cargo test         # run tests
cargo clippy --all-features -- -D warnings
cargo fmt
```

## Security & Verification

All releases are cryptographically signed with multiple methods for maximum security:

### Automatic Verification
The install script automatically verifies all downloads using the strongest available method on your system.

### Manual Verification
For manual downloads, you can verify package authenticity:

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

📖 **Full verification guide:** [docs/VERIFICATION.md](docs/VERIFICATION.md)

## Support the project

If Rusty Commit saves you time, consider supporting ongoing development:

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-@hongkongkiwi-fd2e83?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/hongkongkiwi)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-ffdd00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/hongkongkiwi)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Credits
Rusty Commit is inspired by the original
[OpenCommit](https://github.com/di-sukharev/opencommit) by [@di-sukharev](https://github.com/di-sukharev).
