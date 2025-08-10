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

## Installation

### One‑liner (recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/hongkongkiwi/rusty-commit/main/install.sh | bash
```

The script auto‑detects your platform and installs via Homebrew, .deb/.rpm, Cargo, or binary.

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

## Development
```bash
cargo build        # build
cargo test         # run tests
cargo clippy --all-features -- -D warnings
cargo fmt
```

## License
MIT

## Credits
Rusty Commit is inspired by and compatible with the original
[OpenCommit](https://github.com/di-sukharev/opencommit) by [@di-sukharev](https://github.com/di-sukharev).
