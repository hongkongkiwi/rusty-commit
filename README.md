# Rusty Commit (rco)

[![CI](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml/badge.svg)](https://github.com/hongkongkiwi/rusty-commit/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rusty-commit.svg)](https://crates.io/crates/rusty-commit)
[![Documentation](https://docs.rs/rusty-commit/badge.svg)](https://docs.rs/rusty-commit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

🚀 Write great commit messages in seconds — fast, local, and secure. 🦀

`rco` is a Rust-native, AI‑powered commit assistant. It’s a drop‑in, high‑performance alternative compatible with OpenCommit configs.

### 🌟 Why Rusty Commit
- **Speed**: Native Rust binary with instant startup
- **Choice**: Works with 16+ AI providers (OpenAI, Anthropic/Claude, OpenRouter, Groq, DeepSeek, GitHub Copilot, Ollama…)
- **Secure**: Optional keychain storage via `--features secure-storage`
- **Flexible**: Conventional commits, GitMoji, templates, multi‑language
- **Integrated**: Git hooks, GitHub Actions, MCP server for editors

## Contents
- Installation
- Quick start
- Examples
- Configuration
- Providers
- CLI overview
- Git hooks
- Updates
- GitHub Action
- Advanced
- Troubleshooting
- Uninstall
- Compatibility
- Development
- License & Credits

## Installation

### One‑liner (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

The script auto‑detects your platform and installs via Homebrew, .deb/.rpm, Cargo, or binary.
All packages and binaries are signed with GPG and checksums are verified automatically.

### Cargo
```bash
cargo install rusty-commit                      # basic
cargo install rusty-commit --features secure-storage  # store API keys in system keychain
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
- **Legacy**: reads `~/.opencommit` if present

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
```

Anthropic (Claude):
```bash
# OAuth (recommended)
rco auth login
# Or API key
rco config set RCO_AI_PROVIDER=anthropic
rco config set RCO_API_KEY=sk-ant-...
rco config set RCO_MODEL=claude-3-5-haiku-20241022
```

OpenRouter:
```bash
rco config set RCO_AI_PROVIDER=openrouter
rco config set RCO_API_KEY=sk-or-...
rco config set RCO_API_URL=https://openrouter.ai/api/v1
rco config set RCO_MODEL=openai/gpt-4o-mini
```

Groq:
```bash
rco config set RCO_AI_PROVIDER=groq
rco config set RCO_API_KEY=gsk_...
rco config set RCO_API_URL=https://api.groq.com/openai/v1
rco config set RCO_MODEL=llama-3.1-70b-versatile
```

DeepSeek:
```bash
rco config set RCO_AI_PROVIDER=deepseek
rco config set RCO_API_KEY=sk-...
rco config set RCO_API_URL=https://api.deepseek.com/v1
rco config set RCO_MODEL=deepseek-chat
```

Together AI:
```bash
rco config set RCO_AI_PROVIDER=together
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.together.xyz/v1
rco config set RCO_MODEL=meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo
```

DeepInfra:
```bash
rco config set RCO_AI_PROVIDER=deepinfra
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.deepinfra.com/v1/openai
rco config set RCO_MODEL=meta-llama/Meta-Llama-3-70B-Instruct
```

Mistral AI:
```bash
rco config set RCO_AI_PROVIDER=mistral
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.mistral.ai/v1
rco config set RCO_MODEL=mistral-small-latest
```

Azure OpenAI:
```bash
rco config set RCO_AI_PROVIDER=azure
rco config set RCO_API_KEY=<azure_api_key>
rco config set RCO_API_URL=https://<your-resource>.openai.azure.com
# Use your deployment name, not the model name
rco config set RCO_MODEL=<deployment-name>
```

Google Gemini:
```bash
rco config set RCO_AI_PROVIDER=gemini
rco config set RCO_API_KEY=...
rco config set RCO_MODEL=gemini-pro
```

Perplexity:
```bash
rco config set RCO_AI_PROVIDER=perplexity
rco config set RCO_API_KEY=...
# Optional: custom endpoint
# rco config set RCO_API_URL=https://api.perplexity.ai/chat/completions
rco config set RCO_MODEL=llama-3.1-sonar-small-128k-online
```

Fireworks AI:
```bash
rco config set RCO_AI_PROVIDER=fireworks
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.fireworks.ai/inference/v1
rco config set RCO_MODEL=accounts/fireworks/models/llama-v3p1-70b-instruct
```

Moonshot AI (Kimi):
```bash
rco config set RCO_AI_PROVIDER=moonshot
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://api.moonshot.cn/v1
rco config set RCO_MODEL=kimi-k2
```

Alibaba Model Studio (DashScope / Qwen Coder):
```bash
rco config set RCO_AI_PROVIDER=dashscope
rco config set RCO_API_KEY=...
rco config set RCO_API_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
rco config set RCO_MODEL=qwen3-coder-32b-instruct
```

## Git hooks
```bash
rco hook set    # install prepare-commit-msg hook
rco hook unset  # uninstall
```

## Updates
```bash
rco update --check   # see if a new version is available
rco update           # update using your install method
```

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
- Full OpenCommit config compatibility; easy migration.
- Works with per‑repo overrides and multiple providers.

## Development
```bash
cargo build        # build
cargo test         # run tests
cargo clippy --all-features -- -D warnings
cargo fmt
```

## Support the project

If Rusty Commit saves you time, consider supporting ongoing development:

[![GitHub Sponsors](https://img.shields.io/badge/Sponsor-@hongkongkiwi-fd2e83?logo=github-sponsors&logoColor=white)](https://github.com/sponsors/hongkongkiwi)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-ffdd00?logo=buymeacoffee&logoColor=black)](https://buymeacoffee.com/hongkongkiwi)

## License
MIT

## Credits
Rusty Commit is inspired by and compatible with the original
[OpenCommit](https://github.com/di-sukharev/opencommit) by [@di-sukharev](https://github.com/di-sukharev).
