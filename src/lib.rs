#![allow(clippy::uninlined_format_args)]
#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::manual_async_fn)]

//! Rusty Commit - AI-powered commit message generator written in Rust
//!
//! This library provides the core functionality for generating commit messages
//! using various AI providers. It supports multiple providers including OpenAI,
//! Anthropic, Ollama, Gemini, and Azure.
//!
//! # Example
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

pub mod auth;
pub mod cli;
pub mod commands;
pub mod config;
pub mod git;
pub mod providers;
pub mod update;
pub mod utils;
