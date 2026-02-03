//! Setup wizard implementations

use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input, Select};

use crate::cli::SetupCommand;
use crate::config::Config;

use super::providers::CommitFormat;
use super::ui::{print_completion_message, print_section_header, print_welcome_header};

/// Main entry point for setup command
pub async fn execute(cmd: SetupCommand) -> Result<()> {
    print_welcome_header();

    // Determine if we're doing quick or advanced setup
    let is_advanced = if cmd.defaults {
        // Non-interactive defaults mode
        return apply_defaults().await;
    } else if cmd.advanced {
        true
    } else {
        // Ask user which mode they prefer
        println!();
        println!("{}", "Choose your setup mode:".bold());
        println!();

        let modes = vec![
            "🚀 Quick Setup - Just the essentials (recommended)",
            "⚙️  Advanced Setup - Full configuration options",
        ];

        let selection = Select::new()
            .with_prompt("Select mode")
            .items(&modes)
            .default(0)
            .interact()?;

        selection == 1
    };

    if is_advanced {
        run_advanced_setup().await
    } else {
        run_quick_setup().await
    }
}

/// Quick setup with essential options only
async fn run_quick_setup() -> Result<()> {
    let mut config = Config::load()?;

    // Step 1: Provider selection
    let provider = super::prompts::select_provider_quick()?;
    config.ai_provider = provider.name.to_string();
    config.model = provider.default_model.to_string();

    // Step 2: API key (if needed)
    if provider.requires_key {
        let api_key = super::prompts::prompt_for_api_key(provider.name)?;
        if !api_key.is_empty() {
            config.api_key = Some(api_key);
        }
    } else {
        println!();
        println!(
            "{} {} doesn't require an API key - great for privacy!",
            "ℹ️".blue(),
            provider.name.bright_cyan()
        );
    }

    // Step 3: Commit format
    let format = super::prompts::select_commit_format()?;
    config.commit_type = format.as_str().to_string();
    config.emoji = matches!(format, CommitFormat::Gitmoji);

    // Save configuration
    config.save()?;

    print_completion_message(
        &config.ai_provider,
        &config.model,
        &config.commit_type,
        &config.language,
        false,
    );
    Ok(())
}

/// Advanced setup with all configuration options
async fn run_advanced_setup() -> Result<()> {
    let mut config = Config::load()?;

    // Section 1: AI Provider Configuration
    print_section_header("🤖 AI Provider Configuration");

    let provider = super::prompts::select_provider_advanced()?;
    config.ai_provider = provider.name.to_string();

    // Custom model selection
    let default_model = provider.default_model;
    let use_custom_model = Confirm::new()
        .with_prompt(format!(
            "Use default model ({}), or specify a custom one?",
            default_model.bright_cyan()
        ))
        .default(true)
        .interact()?;

    if use_custom_model {
        config.model = default_model.to_string();
    } else {
        let custom_model: String = Input::new()
            .with_prompt("Enter model name")
            .default(default_model.to_string())
            .interact()?;
        config.model = custom_model;
    }

    // API key or custom endpoint
    if provider.requires_key {
        let api_key = super::prompts::prompt_for_api_key(provider.name)?;
        if !api_key.is_empty() {
            config.api_key = Some(api_key);
        }

        // Custom API URL option
        let use_custom_url = Confirm::new()
            .with_prompt("Use a custom API endpoint URL?")
            .default(false)
            .interact()?;

        if use_custom_url {
            let custom_url: String = Input::new()
                .with_prompt("Enter custom API URL")
                .default(format!("https://api.{}.com/v1", provider.name))
                .interact()?;
            config.api_url = Some(custom_url);
        }
    }

    // Section 2: Commit Message Style
    print_section_header("📝 Commit Message Style");

    let format = super::prompts::select_commit_format()?;
    config.commit_type = format.as_str().to_string();
    config.emoji = matches!(format, CommitFormat::Gitmoji);

    // Capitalization
    config.description_capitalize =
        Confirm::new()
            .with_prompt("Capitalize the first letter of commit messages?")
            .default(true)
            .interact()?;

    // Period at end
    config.description_add_period =
        Confirm::new()
            .with_prompt("Add period at the end of commit messages?")
            .default(false)
            .interact()?;

    // Max length
    let max_length: usize = Input::new()
        .with_prompt("Maximum commit message length")
        .default(100)
        .validate_with(|input: &usize| -> Result<(), &str> {
            if *input >= 50 && *input <= 200 {
                Ok(())
            } else {
                Err("Please enter a value between 50 and 200")
            }
        })
        .interact()?;
    config.description_max_length = max_length;

    // Language selection
    let language = super::prompts::select_language()?;
    config.language = language.to_string();

    // Section 3: Behavior Settings
    print_section_header("⚙️  Behavior Settings");

    // Generate count
    let generate_count: u8 = Input::new()
        .with_prompt("Number of commit variations to generate (1-5)")
        .default(1)
        .validate_with(|input: &u8| -> Result<(), &str> {
            if *input >= 1 && *input <= 5 {
                Ok(())
            } else {
                Err("Please enter a value between 1 and 5")
            }
        })
        .interact()?;
    config.generate_count = generate_count;

    // Git push option
    config.gitpush =
        Confirm::new()
            .with_prompt("Automatically push commits to remote?")
            .default(false)
            .interact()?;

    // One-line commits
    config.one_line_commit =
        Confirm::new()
            .with_prompt("Always use one-line commits (no body)?")
            .default(false)
            .interact()?;

    // Enable commit body
    config.enable_commit_body =
        Confirm::new()
            .with_prompt("Allow multi-line commit messages with body?")
            .default(false)
            .interact()?;

    // Section 4: Advanced Features
    print_section_header("🔧 Advanced Features");

    // Learn from history
    config.learn_from_history =
        Confirm::new()
            .with_prompt("Learn commit style from repository history?")
            .default(false)
            .interact()?;

    if config.learn_from_history {
        let history_count: usize = Input::new()
            .with_prompt("Number of commits to analyze for style")
            .default(50)
            .validate_with(|input: &usize| -> Result<(), &str> {
                if *input >= 10 && *input <= 200 {
                    Ok(())
                } else {
                    Err("Please enter a value between 10 and 200")
                }
            })
            .interact()?;
        config.history_commits_count = history_count;
    }

    // Clipboard on timeout
    config.clipboard_on_timeout =
        Confirm::new()
            .with_prompt("Copy commit message to clipboard on timeout/error?")
            .default(true)
            .interact()?;

    // Hook settings
    config.hook_strict =
        Confirm::new()
            .with_prompt("Strict hook mode (fail on hook errors)?")
            .default(true)
            .interact()?;

    let hook_timeout: u64 = Input::new()
        .with_prompt("Hook timeout (milliseconds)")
        .default(30000)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input >= 1000 && *input <= 300000 {
                Ok(())
            } else {
                Err("Please enter a value between 1000 and 300000")
            }
        })
        .interact()?;
    config.hook_timeout_ms = hook_timeout;

    // Section 5: Token Limits (for advanced users)
    print_section_header("🎯 Token Limits (Optional)");

    let configure_tokens = Confirm::new()
        .with_prompt("Configure token limits? (Most users can skip this)")
        .default(false)
        .interact()?;

    if configure_tokens {
        let max_input: usize = Input::new()
            .with_prompt("Maximum input tokens")
            .default(4096)
            .interact()?;
        config.tokens_max_input = max_input;

        let max_output: u32 = Input::new()
            .with_prompt("Maximum output tokens")
            .default(500)
            .interact()?;
        config.tokens_max_output = max_output;
    }

    // Save configuration
    config.save()?;

    print_completion_message(
        &config.ai_provider,
        &config.model,
        &config.commit_type,
        &config.language,
        true,
    );
    Ok(())
}

/// Apply sensible defaults without prompting
async fn apply_defaults() -> Result<()> {
    let mut config = Config::load()?;

    // Apply sensible defaults without prompting
    config.ai_provider = "openai".to_string();
    config.model = "gpt-4o-mini".to_string();
    config.commit_type = "conventional".to_string();
    config.description_capitalize = true;
    config.description_add_period = false;
    config.description_max_length = 100;
    config.language = "en".to_string();
    config.generate_count = 1;
    config.emoji = false;
    config.gitpush = false;
    config.one_line_commit = false;
    config.enable_commit_body = false;
    config.learn_from_history = false;
    config.clipboard_on_timeout = true;
    config.hook_strict = true;
    config.hook_timeout_ms = 30000;
    config.tokens_max_input = 4096;
    config.tokens_max_output = 500;

    config.save()?;

    println!();
    println!("{} Default configuration applied!", "✓".green().bold());
    println!();
    println!("   Provider: openai (gpt-4o-mini)");
    println!("   Format: conventional commits");
    println!();
    println!(
        "   Set your API key: {}",
        "rco config set RCO_API_KEY=<your_key>".bright_cyan()
    );
    println!();

    Ok(())
}
