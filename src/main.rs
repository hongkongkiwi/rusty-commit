#![allow(clippy::uninlined_format_args)]
#![allow(clippy::bind_instead_of_map)]
#![allow(clippy::manual_async_fn)]
#![allow(dead_code)]

mod auth;
mod cli;
mod commands;
mod config;
mod git;
mod providers;
mod update;
mod utils;

use anyhow::Result;
use clap::Parser;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Check if we're being called as a git hook
    let args: Vec<String> = env::args().collect();
    if commands::githook::is_hook_called(&args) {
        return commands::githook::prepare_commit_msg_hook(&args).await;
    }

    // Parse CLI arguments
    let cli = cli::Cli::parse();

    // Run migrations if needed
    config::migrations::run_migrations()?;

    // Check for updates
    utils::version::check_is_latest_version().await?;

    // Execute the appropriate command
    match cli.command {
        Some(cli::Commands::Config(cmd)) => commands::config::execute(cmd).await,
        Some(cli::Commands::Hook(cmd)) => commands::githook::execute(cmd).await,
        Some(cli::Commands::CommitLint(cmd)) => commands::commitlint::execute(cmd).await,
        Some(cli::Commands::Auth(cmd)) => commands::auth::execute(cmd).await,
        Some(cli::Commands::Mcp(cmd)) => commands::mcp::execute(cmd).await,
        Some(cli::Commands::Update(cmd)) => commands::update::execute(cmd).await,
        None => {
            // Default to commit command
            commands::commit::execute(cli.global).await
        }
    }
}
