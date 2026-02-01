#![allow(clippy::uninlined_format_args)]
#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::manual_async_fn)]

//! Rusty Commit - AI-powered commit message generator written in Rust
//!
//! This library provides the core functionality for generating commit messages
//! using various AI providers. It supports multiple providers including OpenAI,
//! Anthropic, Claude, Ollama, Gemini, and Azure.
//!
//! # Features
//!
//! - **Multiple AI Providers**: OpenAI, Anthropic, Ollama, Gemini, Azure, and more
//! - **Conventional Commits**: Generate properly formatted conventional commits
//! - **GitMoji Support**: Generate commits with GitMoji emojis
//! - **MCP Server**: Use as a Model Context Protocol server for editor integration
//! - **Secure Storage**: Optional keychain integration for API keys
//!
//! # Quick Start
//!
//! ```no_run
//! use rusty_commit::config::Config;
//! use rusty_commit::providers::create_provider;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = Config::load()?;
//! let provider = create_provider(&config)?;
//! let diff = "your git diff here";
//! let message = provider.generate_commit_message(
//!     diff,
//!     None,
//!     false,
//!     &config
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Documentation
//!
//! ## Getting Started
//!
//! ```no_run
//! // 1. Configure your AI provider
//! // Use OAuth (recommended): rco auth login
//! // Or set API key: rco config set RCO_API_KEY=sk-...
//!
//! // 2. Generate a commit
//! // git add .
//! // rco
//! # fn main() {}
//! ```
//!
//! ## Configuration
//!
//! Configuration can be set via environment variables or config files:
//!
//! - Global: `~/.config/rustycommit/config.toml`
//! - Per-repo: `.rustycommit.toml`
//!
//! Common keys:
//! - `RCO_AI_PROVIDER` - Provider name (e.g., `anthropic`, `openai`, `ollama`)
//! - `RCO_MODEL` - Model name
//! - `RCO_API_KEY` - API key
//! - `RCO_COMMIT_TYPE` - `conventional` or `gitmoji`
//!
//! ## AI Providers
//!
//! Supported providers:
//! - **Cloud**: OpenAI, Anthropic Claude, OpenRouter, Groq, DeepSeek, Gemini, Azure, Together AI, DeepInfra, Mistral, Perplexity, Fireworks, Moonshot, DashScope, XAI
//! - **Local**: Ollama
//!
//! Use `rco auth login` for OAuth providers or `rco config set RCO_API_KEY=...` for API key providers.

pub mod auth;
pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod output;
pub mod providers;
pub mod update;
pub mod utils;
