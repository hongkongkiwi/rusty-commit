use anyhow::Result;
use colored::Colorize;
use dialoguer::{Input, Select};

use crate::cli::SetupCommand;
use crate::config::Config;

pub async fn execute(_cmd: SetupCommand) -> Result<()> {
    println!();
    println!(
        "{} {}",
        "🚀".green(),
        "Rusty Commit Setup Wizard".bold().white()
    );
    println!();

    // Step 1: Select AI Provider
    let providers = [
        "OpenAI (GPT-4, GPT-3.5)",
        "Anthropic (Claude)",
        "Google Gemini",
        "Ollama (local)",
    ];

    let provider_selection = Select::new()
        .with_prompt("1. Select your AI provider")
        .items(providers)
        .default(0)
        .interact()?;

    let (provider, default_model) = match provider_selection {
        0 => ("openai", "gpt-4o-mini"),
        1 => ("anthropic", "claude-3-5-haiku-20241022"),
        2 => ("gemini", "gemini-1.5-flash"),
        3 => ("ollama", "llama3.2"),
        _ => ("openai", "gpt-4o-mini"),
    };

    println!();
    println!("{} Selected: {}", "✓".green(), provider.yellow());

    // Step 2: Enter API Key (skip for Ollama)
    let api_key = if provider_selection == 3 {
        println!();
        println!(
            "{} Local Ollama detected - API key not required",
            "ℹ".blue()
        );
        None
    } else {
        println!();
        let input: String = Input::new()
            .with_prompt(format!("2. Enter your {} API key", provider))
            .interact()?;
        if input.trim().is_empty() {
            println!(
                "{} No API key entered - you'll need to set it later",
                "⚠".yellow()
            );
        }
        Some(input)
    };

    // Step 3: Select commit format
    let commit_formats = ["Conventional (feat, fix, etc.)", "GitMoji (🎉, 🐛, ✨)"];
    let format_selection = Select::new()
        .with_prompt("3. Select commit message format")
        .items(commit_formats)
        .default(0)
        .interact()?;

    let commit_type = if format_selection == 0 {
        "conventional"
    } else {
        "gitmoji"
    };

    println!();
    println!("{} Selected format: {}", "✓".green(), commit_type.yellow());

    // Save configuration
    let mut config = Config::load()?;
    config.ai_provider = Some(provider.to_string());
    config.model = Some(default_model.to_string());
    config.commit_type = Some(commit_type.to_string());
    config.description_capitalize = Some(true);
    config.description_add_period = Some(false);
    config.generate_count = Some(1);

    if let Some(key) = api_key {
        if !key.trim().is_empty() {
            config.api_key = Some(key);
        }
    }

    config.save()?;

    // Completion message
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );
    println!();
    println!("{} Setup complete!", "✓".green().bold());
    println!();
    println!("{} You can now run: {}", "→".cyan(), "rco".bold().white());
    println!();
    println!(
        "{} To change settings later, use: {}",
        "→".cyan(),
        "rco config set <key>=<value>".bold().white()
    );
    println!(
        "    Or run this setup again: {}",
        "rco setup".bold().white()
    );
    println!();
    println!(
        "{}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".dimmed()
    );

    Ok(())
}
