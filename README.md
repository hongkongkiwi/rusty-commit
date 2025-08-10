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
