use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};

use crate::cli::ModelCommand;
use crate::config::Config;

pub async fn execute(cmd: ModelCommand) -> Result<()> {
    let mut config = Config::load()?;

    if cmd.list {
        list_models(&config, cmd.provider.as_deref()).await?;
        return Ok(());
    }

    // Interactive model selection
    select_model_interactive(&mut config).await?;

    Ok(())
}

async fn list_models(config: &Config, provider_override: Option<&str>) -> Result<()> {
    let provider = provider_override
        .unwrap_or(config.ai_provider.as_str())
        .to_lowercase();

    println!(
        "{}",
        format!("Available models for provider: {}", provider).green()
    );
    println!("{}", "─".repeat(50).dimmed());

    let models = match provider.as_str() {
        "openai" | "deepseek" | "groq" | "openrouter" | "together" | "deepinfra"
        | "huggingface" | "mistral" | "fireworks" | "moonshot" | "qwen" | "qwen-coder"
        | "amazon-bedrock" | "github-models" => vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4",
            "gpt-3.5-turbo",
        ],
        "anthropic" | "claude" => vec![
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-sonnet-4",
            "claude-opus-4",
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-haiku-20240307",
            "claude-3-sonnet-20240229",
            "claude-3-opus-20240229",
        ],
        "ollama" => vec![
            "llama3.3",
            "llama3.2",
            "llama3.1",
            "llama3",
            "mistral",
            "mixtral",
            "qwen2.5",
            "codellama",
            "deepseek-coder",
            "starcoder2",
        ],
        "gemini" | "vertex" | "google-vertex" => vec![
            "gemini-2.0-flash-exp",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "gemini-1.0-pro",
        ],
        "azure" | "azure-openai" => vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-35-turbo"],
        "perplexity" => vec!["sonar-reasoning", "sonar", "r1-1776", "doctl"],
        _ => vec!["gpt-3.5-turbo", "gpt-4", "gpt-4o", "claude-3-5-sonnet"],
    };

    for (i, model) in models.iter().enumerate() {
        let marker = if config.model == *model {
            "✓"
        } else {
            " "
        };
        println!("{}. {} {}", i + 1, marker, model);
    }

    println!();
    println!("{}", "To set a model:".yellow());
    println!("  rco config set RCO_MODEL=<model_name>");
    println!("  rco model  # interactive selection");

    Ok(())
}

async fn select_model_interactive(config: &mut Config) -> Result<()> {
    let provider = config
        .ai_provider
        .as_str()
        .to_lowercase();

    println!("{}", "🤖 Interactive Model Selection".green().bold());
    println!("Current provider: {}", provider.cyan());
    println!("Current model: {}", config.model.cyan());
    println!("{}", "─".repeat(50).dimmed());

    // Get model list for provider
    let models = get_provider_models(&provider);

    // Add "Custom model" option
    let mut options = models.clone();
    options.push("Enter custom model name".to_string());

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a model")
        .items(&options)
        .default(0)
        .interact()?;

    if selection == options.len() - 1 {
        // Custom model
        let custom_model: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter model name")
            .interact_text()?;

        config.model = custom_model;
    } else {
        config.model = options[selection].clone();
    }

    // Save config
    config.save()?;

    println!();
    println!("{}", format!("✅ Model set to: {}", config.model).green());

    Ok(())
}

fn get_provider_models(provider: &str) -> Vec<String> {
    match provider {
        "openai" | "deepseek" | "groq" | "openrouter" | "together" | "deepinfra"
        | "huggingface" | "mistral" | "fireworks" | "moonshot" | "amazon-bedrock"
        | "github-models" => vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-3.5-turbo".to_string(),
        ],
        "anthropic" | "claude" => vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
        ],
        "qwen" | "qwen-coder" | "dashscope" | "alibaba" => vec![
            "qwen3-coder:480b".to_string(),
            "qwen3-coder:30b-a3b".to_string(),
            "qwen3-vl-235b-instruct".to_string(),
            "qwen-turbo".to_string(),
            "qwen-plus".to_string(),
            "qwen-max".to_string(),
        ],
        "ollama" => vec![
            "llama3.3".to_string(),
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "mixtral".to_string(),
            "qwen2.5".to_string(),
            "codellama".to_string(),
            "deepseek-coder".to_string(),
        ],
        "gemini" | "vertex" | "google-vertex" => vec![
            "gemini-2.0-flash-exp".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ],
        "xai" | "grok" | "x-ai" => vec![
            "grok-2-1212".to_string(),
            "grok-2".to_string(),
            "grok-beta".to_string(),
            "grok-2-vision-1212".to_string(),
        ],
        "codex" => vec![
            "gpt-5.1-codex".to_string(),
            "gpt-5.1-codex-mini".to_string(),
            "gpt-5.1-codex-max".to_string(),
        ],
        "perplexity" => vec![
            "sonar-reasoning".to_string(),
            "sonar".to_string(),
            "r1-1776".to_string(),
        ],
        _ => vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-4o".to_string(),
            "claude-3-5-sonnet".to_string(),
        ],
    }
}
