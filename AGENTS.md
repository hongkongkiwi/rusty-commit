# AGENTS.md - Rusty Commit (rco) - AI Agent Reference

**Purpose**: Reference for AI agents working on the Rusty Commit codebase.

**Project**: Rusty Commit (`rco`) - AI-powered commit message generator in Rust  
**Repository**: https://github.com/hongkongkiwi/rusty-commit  
**Version**: 1.0.24 | **Last Updated**: 2026-02-02

---

## Quick Reference

| Aspect | Details |
|--------|---------|
| **Binary Name** | `rco` (also `rusty-commit`) |
| **Code Size** | ~15,500 lines across 58 source files |
| **Architecture** | Trait-based provider system, async/await with tokio |
| **Key Dependencies** | tokio 1.35, clap 4.5, git2 0.20, async-openai 0.32, rmcp 0.13 |

---

## Critical Rules

### NEVER Without Explicit Confirmation
- `git push`, `git reset --hard`, `git clean -fd`
- `rm -rf` or deleting files outside project dir
- `cargo publish`
- Modifying system files or installing system packages

### Security
- **Never hardcode or log** API keys/tokens
- **Verify `.gitignore`** excludes sensitive files before committing
- Use `keyring` for secure storage (falls back to file with 0o600 perms)
- **No `unwrap()`** in production paths (use `?`)

---

## Context Management

### When to Spawn Subagents
| Scenario | Approach |
|----------|----------|
| Diff > 5,000 lines | Request user to stage specific files |
| 2,000-5,000 lines | Use subagents for parallel exploration |
| Multiple independent modules | Spawn parallel tasks |

---

## Architecture

```
CLI (cli.rs) → Commands (commands/) → Providers (providers/) → Utils (utils/)
```

### AIProvider Trait
```rust
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_commit_message(&self, diff: &str, context: Option<&str>) -> Result<String>;
}
```

---

## Directory Structure

```
src/
├── main.rs, lib.rs, cli.rs, config.rs, git.rs, update.rs
├── auth/          # OAuth, token storage, multi-account
├── commands/      # commit, auth, config, mcp, pr, model, skills
├── providers/     # AI implementations (openai, anthropic, ollama, gemini, etc.)
├── skills/        # Extension framework
├── config/        # Format, secure_storage, accounts, migrations
└── utils/         # token, hooks, retry, version

tests/             # 10 integration test files
```

---

## Provider System

| Provider | Config Value | Feature | OAuth | Category |
|----------|--------------|---------|-------|----------|
| **Core Providers** |||||
| Anthropic | `anthropic` | `anthropic` | ✅ | Popular |
| OpenAI | `openai` | `openai` | ❌ | Popular |
| Gemini | `gemini` | `gemini` | ❌ | Popular |
| xAI/Grok | `xai` | `xai` | ❌ | Popular |
| **Fast Inference** |||||
| Groq | `groq` | `groq` | ❌ | Fast |
| Cerebras | `cerebras` | `cerebras` | ❌ | Fast |
| SambaNova | `sambanova` | `sambanova` | ❌ | Fast |
| Nebius | `nebius` | `nebius` | ❌ | GPU Cloud |
| **Multi-Model Aggregators** |||||
| OpenRouter | `openrouter` | `openrouter` | ❌ | Aggregator |
| Together AI | `together` | `together` | ❌ | Aggregator |
| Fireworks | `fireworks` | `fireworks` | ❌ | Aggregator |
| Replicate | `replicate` | `replicate` | ❌ | Aggregator |
| DeepInfra | `deepinfra` | `deepinfra` | ❌ | Aggregator |
| Novita | `novita` | `novita` | ❌ | Aggregator |
| **Enterprise** |||||
| Azure OpenAI | `azure` | `azure` | ❌ | Enterprise |
| AWS Bedrock | `bedrock` | `bedrock` | ❌ | Enterprise |
| Google Vertex AI | `vertex` | `vertex` | ❌ | Enterprise |
| Mistral | `mistral` | `mistral` | ❌ | Enterprise |
| Cohere | `cohere` | `cohere` | ❌ | Enterprise |
| AI21 Labs | `ai21` | `ai21` | ❌ | Enterprise |
| Perplexity | `perplexity` | `perplexity` | ❌ | Enterprise |
| **Local/Self-hosted** |||||
| Ollama | `ollama` | `ollama` | ❌ | Local |
| LM Studio | `lmstudio` | `lmstudio` | ❌ | Local |
| llama.cpp | `llamacpp` | `llamacpp` | ❌ | Local |
| Apple MLX | `mlx` | `mlx` | ❌ | Local |
| **China-based** |||||
| Moonshot/Kimi | `moonshot` | `moonshot` | ❌ | China |
| SiliconFlow | `siliconflow` | `siliconflow` | ❌ | China |
| Zhipu AI | `zhipu` | `zhipu` | ❌ | China |
| MiniMax | `minimax` | `minimax` | ❌ | China |
| DashScope | `dashscope` | `dashscope` | ❌ | China |
| **GPU Cloud Providers** |||||
| Lambda | `lambda` | `lambda` | ❌ | GPU Cloud |
| Hyperbolic | `hyperbolic` | `hyperbolic` | ❌ | GPU Cloud |
| Kluster | `kluster` | `kluster` | ❌ | GPU Cloud |
| Scaleway | `scaleway` | `scaleway` | ❌ | GPU Cloud |
| OVHcloud | `ovh` | `ovh` | ❌ | GPU Cloud |
| **AI Gateways** |||||
| Helicone | `helicone` | `helicone` | ❌ | Gateway |
| Cloudflare Workers AI | `workers-ai` | `workers-ai` | ❌ | Gateway |
| **Other Dedicated Providers** |||||
| Hugging Face | `huggingface` | `huggingface` | ❌ | Platform |
| NVIDIA NIM | `nvidia` | `nvidia` | ❌ | Platform |
| Flowise | `flowise` | `flowise` | ❌ | Platform |
| GitHub Copilot | `github-copilot` | - | ✅ | Copilot |

**Plus 70+ additional OpenAI-compatible providers** including: Upstage, Baseten, Chutes, IO.Net, ModelScope, Requesty, Morph, Synthetic, NanoGPT, ZenMux, V0, Venice, Cortecs, Abacus, Bailing, FastRouter, Inference.net, Submodel, Z.AI, Zhipu Coding, Poe, Predibase, TensorOps, Targon, Corcel, CyberNative, Edgen, GigaChat, Hydra, Jina AI, Lingyi, Monica, Pollinations, ShuttleAI, Teknium, TheB, TryLeap, SAP AI Core, 302.AI, and more.

**Adding a Provider:**
1. Create `src/providers/newprovider.rs`
2. Implement `AIProvider` trait
3. Add to factory in `src/providers/mod.rs`
4. Add feature flag in `Cargo.toml`
5. **Add tests** in `tests/`

---

## Configuration

**Locations** (priority: per-repo > global > env > defaults):
- Global: `~/.config/rustycommit/config.{toml,json}`
- Per-repo: `.rustycommit.toml` or `.rco.toml`
- Env: `RCO_*` prefix

**Common Keys:**
| Key | Purpose |
|-----|---------|
| `RCO_AI_PROVIDER` | Provider name |
| `RCO_MODEL` | Model name |
| `RCO_API_KEY` | API key |
| `RCO_API_URL` | Custom endpoint |
| `RCO_COMMIT_TYPE` | `conventional` or `gitmoji` |
| `RCO_EMOJI` | `true`/`false` |
| `RCO_LANGUAGE` | Output language |

---

## Skills System

Skills are stored in `~/.config/rustycommit/skills/<name>/` with:
- `skill.toml` - Manifest
- `prompt.md` - Custom prompt template
- `hooks/` - Optional scripts (pre_gen, post_gen, format)

**Prompt placeholders:** `{diff}`, `{context}`, `{language}`, `{commit_type}`

---

## Coding Standards

### Error Handling
```rust
// Good
fn process(input: &str) -> Result<String> {
    let parsed = parse_input(input)?;
    Ok(format!("Processed: {}", parsed))
}

// Avoid
fn process(input: &str) -> String {
    let parsed = parse_input(input).unwrap();  // Never in production
    format!("Processed: {}", parsed)
}
```

### Commit Messages
Follow Conventional Commits: `type(scope): description`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `ci`, `build`

---

## Testing

```bash
cargo test              # All tests
cargo test --all-features
cargo test test_name    # Specific test
cargo test -- --nocapture  # With output
```

**Must add tests for:**
- New AI providers (integration test)
- New CLI commands
- Configuration changes
- Authentication changes
- Bug fixes (regression test)

---

## Build Commands

```bash
cargo build
cargo build --release
cargo build --all-features
cargo build --features secure-storage
cargo build --no-default-features --features openai  # Minimal

just build     # via justfile
just test
just fmt
just clippy
just all       # fmt + clippy + test
```

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| 401 / Invalid API key | `rco auth login` or set `RCO_API_KEY` |
| Rate-limited (429) | Wait; try lighter model |
| Secure storage fails | Falls back to file; `rco config status` |
| Hooks not running | Check `.git/hooks/prepare-commit-msg` is executable |
| Build failures | `cargo clean && cargo update` |
| Test failures | `RUST_BACKTRACE=1 cargo test -- --nocapture` |
| Debug logging | `RUST_LOG=debug rco ...` |

---

## Documentation Files

- `README.md` - User guide
- `README_OAUTH.md` - OAuth flow
- `CONTRIBUTING.md` - Dev setup, PR process
- `INSTALL.md` - Installation
- `docs/VERIFICATION.md` - Release verification
- `docs/SECURITY.md` - Security guidelines
- `action.yml` - GitHub Action config
