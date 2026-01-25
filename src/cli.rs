use clap::{Parser, Subcommand};

use crate::output::prelude::OutputFormat;

#[derive(Parser)]
#[command(
    name = "rco",
    version,
    author,
    about = "Rusty Commit - AI-powered commit message generator written in Rust 🚀🤖",
    after_help = r#"EXAMPLES:
    # Generate a commit message for staged changes
    rco

    # Generate with context and copy to clipboard
    rco -c "Focus on auth changes" --clipboard

    # Generate 3 variations and skip confirmation
    rco -g 3 -y

    # Use GitMoji format
    rco --fgm

    # Authenticate with Anthropic
    rco auth login

    # Setup git hooks
    rco hook set

    # Generate PR description
    rco pr generate --base main

    # Generate shell completions
    rco completions bash
    rco completions zsh
    rco completions fish
"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub global: GlobalOptions,
}

#[derive(Parser, Clone)]
pub struct GlobalOptions {
    /// Use full GitMoji specification
    #[arg(long = "fgm", default_value = "false")]
    pub full_gitmoji: bool,

    /// Additional user input context for the commit message
    #[arg(short = 'c', long = "context")]
    pub context: Option<String>,

    /// Skip commit confirmation prompt
    #[arg(short = 'y', long = "yes", default_value = "false")]
    pub skip_confirmation: bool,

    /// Show the prompt that would be used without generating commit
    #[arg(long = "show-prompt", default_value = "false")]
    pub show_prompt: bool,

    /// Disable running pre-hooks
    #[arg(long = "no-pre-hooks", default_value = "false")]
    pub no_pre_hooks: bool,

    /// Disable running post-hooks
    #[arg(long = "no-post-hooks", default_value = "false")]
    pub no_post_hooks: bool,

    /// Number of commit message variations to generate (1-5)
    #[arg(short = 'g', long = "generate", default_value = "1")]
    pub generate_count: u8,

    /// Copy generated message to clipboard instead of committing
    #[arg(short = 'C', long = "clipboard", default_value = "false")]
    pub clipboard: bool,

    /// Exclude specific files from the diff sent to AI
    #[arg(short = 'x', long = "exclude")]
    pub exclude_files: Option<Vec<String>>,

    /// Show detailed timing information
    #[arg(long = "timing", default_value = "false")]
    pub timing: bool,

    /// Strip <thinking> tags from AI responses (for reasoning models)
    #[arg(long = "strip-thinking", default_value = "false")]
    pub strip_thinking: bool,

    /// Output commit message to stdout instead of committing (for hooks)
    #[arg(long = "print", default_value = "false")]
    pub print_message: bool,

    /// Output format (pretty, json, markdown)
    #[arg(long = "output-format", default_value = "pretty")]
    pub output_format: OutputFormat,
}

#[derive(Parser)]
pub struct SetupCommand {
    /// Skip interactive prompts and use defaults
    #[arg(long, default_value = "false")]
    pub defaults: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage Rusty Commit configuration
    Config(ConfigCommand),

    /// Setup git hooks
    Hook(HookCommand),

    /// Generate commitlint configuration
    #[command(name = "commitlint")]
    CommitLint(CommitLintCommand),

    /// Authenticate with Claude using OAuth
    Auth(AuthCommand),

    /// Start MCP (Model Context Protocol) server
    Mcp(McpCommand),

    /// Check for updates and update rusty-commit
    Update(UpdateCommand),

    /// Generate PR description
    Pr(PrCommand),

    /// Interactive model selection
    Model(ModelCommand),

    /// Interactive setup wizard
    Setup(SetupCommand),

    /// Generate shell completions
    Completions(CompletionsCommand),
}

#[derive(Parser)]
pub struct PrCommand {
    #[command(subcommand)]
    pub action: PrAction,
}

#[derive(Subcommand)]
pub enum PrAction {
    /// Generate a PR description
    Generate {
        /// Base branch to compare against (default: main)
        #[arg(short, long)]
        base: Option<String>,
    },
    /// Open PR creation page in browser
    Browse {
        /// Base branch to compare against (default: main)
        #[arg(short, long)]
        base: Option<String>,
    },
}

#[derive(Parser)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key=value pairs
        #[arg(required = true)]
        pairs: Vec<String>,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset configuration to defaults
    Reset {
        /// Reset all configuration
        #[arg(long)]
        all: bool,
        /// Specific keys to reset
        keys: Vec<String>,
    },
    /// Show secure storage status
    Status,
    /// Describe all configuration options with examples and descriptions
    Describe,
    /// Add a new provider account
    AddProvider {
        /// Provider to add (openai, anthropic, claude-code, qwen, ollama, xai, gemini, perplexity, azure)
        #[arg(short, long)]
        provider: Option<String>,
        /// Account alias (e.g., "work", "personal")
        #[arg(short, long)]
        alias: Option<String>,
    },
    /// List all configured accounts
    ListAccounts,
    /// Switch to a different account
    UseAccount {
        /// Account alias to use
        alias: String,
    },
    /// Remove an account
    RemoveAccount {
        /// Account alias to remove
        alias: String,
    },
    /// Show account details
    ShowAccount {
        /// Account alias (defaults to "default")
        alias: Option<String>,
    },
}

#[derive(Parser)]
pub struct HookCommand {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Install prepare-commit-msg git hook
    PrepareCommitMsg,
    /// Install commit-msg git hook (non-interactive)
    CommitMsg,
    /// Uninstall git hooks
    Unset,
    /// Install or uninstall pre-commit hooks
    Precommit {
        /// Install pre-commit hooks
        #[arg(long)]
        set: bool,
        /// Uninstall pre-commit hooks
        #[arg(long)]
        unset: bool,
    },
}

#[derive(Parser)]
pub struct CommitLintCommand {
    /// Set configuration non-interactively
    #[arg(long)]
    pub set: bool,
}

#[derive(Parser)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub action: AuthAction,
}

#[derive(Subcommand)]
pub enum AuthAction {
    /// Login with Claude OAuth
    Login,
    /// Logout and remove stored tokens
    Logout,
    /// Check authentication status
    Status,
}

#[derive(Parser)]
pub struct McpCommand {
    #[command(subcommand)]
    pub action: McpAction,
}

#[derive(Subcommand)]
pub enum McpAction {
    /// Start MCP server on TCP port (for Cursor integration)
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "3000")]
        port: Option<u16>,
    },
    /// Start MCP server over STDIO (for direct integration)
    Stdio,
}

#[derive(Parser)]
pub struct UpdateCommand {
    /// Check for updates without installing
    #[arg(short, long)]
    pub check: bool,

    /// Force update even if already on latest version
    #[arg(short, long)]
    pub force: bool,

    /// Specify version to update to (e.g., "1.0.2")
    #[arg(short, long)]
    pub version: Option<String>,
}

#[derive(Parser)]
pub struct ModelCommand {
    /// List available models for current provider
    #[arg(long = "list")]
    pub list: bool,
    /// Specify provider to list models for
    #[arg(short, long)]
    pub provider: Option<String>,
}

#[derive(Parser)]
pub struct CompletionsCommand {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: clap_complete::Shell,
}
