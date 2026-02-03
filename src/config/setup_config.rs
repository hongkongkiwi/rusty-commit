//! Configuration built during TUI setup
//!
//! This module defines the SetupConfig struct that accumulates configuration
//! during the interactive TUI setup process.

use serde::{Deserialize, Serialize};

/// Commit format options for generated commit messages
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CommitFormat {
    Conventional,
    Gitmoji,
    Simple,
}

impl CommitFormat {
    /// Get display name for the commit format
    pub fn display(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "Conventional Commits (feat:, fix:, docs:, etc.)",
            CommitFormat::Gitmoji => "GitMoji (✨ feat:, 🐛 fix:, 📝 docs:, etc.)",
            CommitFormat::Simple => "Simple (no prefix)",
        }
    }

    /// Get string representation for config
    pub fn as_str(&self) -> &'static str {
        match self {
            CommitFormat::Conventional => "conventional",
            CommitFormat::Gitmoji => "gitmoji",
            CommitFormat::Simple => "simple",
        }
    }

    /// Get all available commit formats
    pub fn all() -> Vec<Self> {
        vec![
            CommitFormat::Conventional,
            CommitFormat::Gitmoji,
            CommitFormat::Simple,
        ]
    }
}

impl Default for CommitFormat {
    fn default() -> Self {
        Self::Conventional
    }
}

/// Configuration built during TUI setup
///
/// This struct holds all configuration options that are collected
/// during the interactive setup process before being saved to the
/// main Config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    /// Selected AI provider
    pub provider: Option<ProviderOption>,

    /// Model to use for the selected provider
    pub model: String,

    /// API key for the provider (if required)
    pub api_key: Option<String>,

    /// Custom API URL (if using a custom endpoint)
    pub api_url: Option<String>,

    /// Commit message format style
    pub commit_style: CommitFormat,

    /// Output language for commit messages
    pub language: String,

    /// Capitalize first letter of commit messages
    pub description_capitalize: bool,

    /// Add period at end of commit messages
    pub description_add_period: bool,

    /// Maximum length for commit message descriptions
    pub description_max_length: usize,

    /// Number of commit variations to generate
    pub generate_count: u8,

    /// Use emojis in commit messages
    pub emoji: bool,

    /// Automatically push commits to remote
    pub gitpush: bool,

    /// Use one-line commits (no body)
    pub one_line_commit: bool,

    /// Allow multi-line commit messages with body
    pub enable_commit_body: bool,

    /// Learn commit style from repository history
    pub learn_from_history: bool,

    /// Number of commits to analyze for style learning
    pub history_commits_count: usize,

    /// Copy commit message to clipboard on timeout/error
    pub clipboard_on_timeout: bool,

    /// Strict hook mode (fail on hook errors)
    pub hook_strict: bool,

    /// Hook timeout in milliseconds
    pub hook_timeout_ms: u64,

    /// Maximum input tokens
    pub tokens_max_input: usize,

    /// Maximum output tokens
    pub tokens_max_output: u32,
}

/// Provider option for the setup wizard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderOption {
    /// Provider identifier (e.g., "openai", "anthropic")
    pub name: String,

    /// Display name shown in UI
    pub display: String,

    /// Default model for this provider
    pub default_model: String,

    /// Whether this provider requires an API key
    pub requires_key: bool,

    /// Category for organizing providers
    pub category: ProviderCategory,
}

impl PartialEq for ProviderOption {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// Category for organizing AI providers
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProviderCategory {
    Popular,
    Local,
    Cloud,
    Enterprise,
    Specialized,
}

impl ProviderCategory {
    /// Get display name for the category
    pub fn display(&self) -> &'static str {
        match self {
            ProviderCategory::Popular => "Popular Providers",
            ProviderCategory::Local => "Local/Private",
            ProviderCategory::Cloud => "Cloud Providers",
            ProviderCategory::Enterprise => "Enterprise",
            ProviderCategory::Specialized => "Specialized",
        }
    }
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            provider: None,
            model: String::new(),
            api_key: None,
            api_url: None,
            commit_style: CommitFormat::Conventional,
            language: "en".to_string(),
            description_capitalize: true,
            description_add_period: false,
            description_max_length: 100,
            generate_count: 1,
            emoji: false,
            gitpush: false,
            one_line_commit: false,
            enable_commit_body: false,
            learn_from_history: false,
            history_commits_count: 50,
            clipboard_on_timeout: true,
            hook_strict: true,
            hook_timeout_ms: 30000,
            tokens_max_input: 4096,
            tokens_max_output: 500,
        }
    }
}
