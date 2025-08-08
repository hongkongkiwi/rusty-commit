//! Rusty Commit - AI-powered commit message generator written in Rust
//!
//! This library provides the core functionality for generating commit messages
//! using various AI providers. It supports multiple providers including OpenAI,
//! Anthropic, Ollama, Gemini, and Azure.
//!
//! # Example
//!
//! ```no_run
//! use rustycommit::config::Config;
//! use rustycommit::providers::create_provider;
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

pub mod auth;
pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod providers;
pub mod utils;
