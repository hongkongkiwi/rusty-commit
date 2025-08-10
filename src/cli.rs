use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rco",
    version,
    author,
    about = "Rusty Commit - AI-powered commit message generator written in Rust 🚀🤖"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub global: GlobalOptions,
}

#[derive(Parser)]
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
}

#[derive(Parser)]
pub struct HookCommand {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Install git hooks
    Set,
    /// Uninstall git hooks
    Unset,
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
