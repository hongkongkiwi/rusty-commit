use anyhow::Result;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::auth::oauth::OAuthClient;
use crate::auth::token_storage;
use crate::cli::{AuthAction, AuthCommand};
use crate::config::Config;

/// Execute auth command from CLI
pub async fn execute(cmd: AuthCommand) -> Result<()> {
    match cmd.action {
        AuthAction::Login => login().await,
        AuthAction::Logout => logout().await,
        AuthAction::Status => status().await,
    }
}

/// Login with interactive provider selection
async fn login() -> Result<()> {
    println!(
        "\n{}",
        "🚀 Welcome to Rusty Commit Authentication".cyan().bold()
    );
    println!("{}", "──────────────────────────────────────────".dimmed());

    // Check if already authenticated
    if token_storage::has_valid_token() {
        let should_reauth = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("You are already authenticated. Do you want to re-authenticate?")
            .default(false)
            .interact()?;

        if !should_reauth {
            println!("{}", "✓ Authentication unchanged".green());
            return Ok(());
        }
    }

    // Provider selection menu - all providers supported by OpenCode
    let providers = vec![
        (
            "Anthropic Claude",
            "Use Claude Pro/Max subscription or API key",
        ),
        (
            "GitHub Copilot",
            "Use GitHub Copilot subscription (recommended)",
        ),
        ("OpenAI", "GPT models with OpenAI API key"),
        ("Google Gemini", "Google Gemini and Vertex AI models"),
        ("OpenRouter", "Access 200+ models via OpenRouter"),
        (
            "Perplexity",
            "Cost-effective AI models with web search capabilities",
        ),
        ("Groq", "Fast inference with Groq API"),
        ("DeepSeek", "DeepSeek models and API"),
        ("Mistral", "Mistral AI models and API"),
        ("AWS Bedrock", "Amazon Bedrock AI models"),
        ("Azure OpenAI", "Azure-hosted OpenAI models"),
        ("Together AI", "Together AI platform"),
        ("DeepInfra", "DeepInfra hosted models"),
        ("Hugging Face", "Hugging Face Inference API"),
        ("GitHub Models", "GitHub hosted AI models"),
        ("Ollama", "Local Ollama instance"),
        ("Other", "Custom OpenAI-compatible provider"),
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("🤖 Select AI provider")
        .items(
            &providers
                .iter()
                .map(|(name, desc)| format!("{:<20} {}", name, desc.dimmed()))
                .collect::<Vec<_>>(),
        )
        .default(0)
        .interact()?;

    match selection {
        0 => handle_anthropic_auth().await,
        1 => handle_github_copilot_auth().await,
        2 => handle_openai_auth().await,
        3 => handle_gemini_auth().await,
        4 => handle_openrouter_auth().await,
        5 => handle_perplexity_auth().await,
        6 => handle_groq_auth().await,
        7 => handle_deepseek_auth().await,
        8 => handle_mistral_auth().await,
        9 => handle_aws_bedrock_auth().await,
        10 => handle_azure_auth().await,
        11 => handle_together_auth().await,
        12 => handle_deepinfra_auth().await,
        13 => handle_huggingface_auth().await,
        14 => handle_github_models_auth().await,
        15 => handle_ollama_auth().await,
        16 => handle_manual_auth().await,
        _ => unreachable!(),
    }
}

/// Handle Anthropic/Claude authentication with multiple options
async fn handle_anthropic_auth() -> Result<()> {
    println!("\n{}", "🧠 Anthropic Claude Authentication".cyan().bold());

    let auth_methods = vec![
        "Claude Pro/Max (OAuth) - Recommended",
        "API Key (Console) - Create new key",
        "API Key (Manual) - Enter existing key",
    ];

    let method = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose authentication method")
        .items(&auth_methods)
        .default(0)
        .interact()?;

    match method {
        0 => handle_claude_oauth().await,
        1 => handle_claude_api_key_creation().await,
        2 => handle_manual_api_key("anthropic").await,
        _ => unreachable!(),
    }
}

/// Handle Claude OAuth authentication
async fn handle_claude_oauth() -> Result<()> {
    println!("\n{}", "🔐 Starting Claude OAuth authentication...".cyan());
    println!(
        "{}",
        "This will use your Claude Pro/Max subscription".dimmed()
    );

    let oauth_client = OAuthClient::new();
    let (auth_url, verifier) = oauth_client.get_authorization_url()?;

    println!(
        "\n{}",
        "Please visit the following URL to authenticate:".bold()
    );
    println!("{}", auth_url.blue().underline());

    // Try to open browser automatically
    if webbrowser::open(&auth_url).is_ok() {
        println!("\n{}", "✓ Browser opened automatically".green());
    } else {
        println!(
            "\n{}",
            "⚠ Could not open browser automatically. Please visit the URL above.".yellow()
        );
    }

    // Show progress spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.set_message("Waiting for authentication...");
    pb.enable_steady_tick(Duration::from_millis(100));

    // Wait for callback
    match oauth_client.start_callback_server(verifier).await {
        Ok(token_response) => {
            pb.finish_and_clear();

            // Store tokens using the new storage method
            token_storage::store_tokens(
                &token_response.access_token,
                token_response.refresh_token.as_deref(),
                token_response.expires_in,
            )?;

            println!("{}", "✓ Authentication successful!".green().bold());
            println!("  You can now use Rusty Commit with your Claude account.");

            // Update config to use anthropic provider
            let mut config = Config::load()?;
            config.ai_provider = Some("anthropic".to_string());
            config.save()?;

            Ok(())
        }
        Err(e) => {
            pb.finish_and_clear();
            println!("{}", format!("✗ Authentication failed: {}", e).red().bold());
            Err(e)
        }
    }
}

/// Handle Claude API key creation through console
async fn handle_claude_api_key_creation() -> Result<()> {
    println!("\n{}", "🔑 Creating Claude API Key".cyan());
    println!(
        "{}",
        "This will create a new API key in your Claude Console".dimmed()
    );

    // For now, redirect to manual entry - API key creation requires additional OAuth flow
    println!(
        "{}",
        "⚠️  Automatic API key creation not yet implemented".yellow()
    );
    println!(
        "{}",
        "Please create an API key manually at: https://console.anthropic.com/settings/keys".cyan()
    );

    handle_manual_api_key("anthropic").await
}

/// Handle OpenAI authentication
async fn handle_openai_auth() -> Result<()> {
    println!("\n{}", "🤖 OpenAI Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://platform.openai.com/api-keys".cyan()
    );

    handle_manual_api_key("openai").await
}

/// Handle Ollama authentication  
async fn handle_ollama_auth() -> Result<()> {
    println!("\n{}", "🦙 Ollama Configuration".cyan().bold());

    let use_local = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Are you using a local Ollama instance?")
        .default(true)
        .interact()?;

    let mut config = Config::load()?;
    config.ai_provider = Some("ollama".to_string());

    if use_local {
        config.api_url = Some("http://localhost:11434".to_string());
        println!(
            "{}",
            "✓ Configured for local Ollama (http://localhost:11434)".green()
        );
    } else {
        let url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter Ollama URL")
            .default("http://localhost:11434".to_string())
            .interact_text()?;

        config.api_url = Some(url.clone());
        println!(
            "{}",
            format!("✓ Configured for remote Ollama ({})", url).green()
        );
    }

    // Get available models (this would ideally query Ollama)
    let model: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter model name")
        .default("mistral".to_string())
        .interact_text()?;

    config.model = Some(model.clone());
    config.save()?;

    println!(
        "{}",
        format!("✓ Ollama configured with model: {}", model)
            .green()
            .bold()
    );
    Ok(())
}

/// Handle Gemini authentication
async fn handle_gemini_auth() -> Result<()> {
    println!("\n{}", "💎 Google Gemini Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://aistudio.google.com/app/apikey".cyan()
    );

    handle_manual_api_key("gemini").await
}

/// Handle Azure authentication
async fn handle_azure_auth() -> Result<()> {
    println!("\n{}", "☁️ Azure OpenAI Configuration".cyan().bold());

    let mut config = Config::load()?;
    config.ai_provider = Some("azure".to_string());

    let api_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter Azure OpenAI API key")
        .interact_text()?;

    let endpoint: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter Azure OpenAI endpoint")
        .default("https://your-resource.openai.azure.com".to_string())
        .interact_text()?;

    let deployment: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter deployment name")
        .default("gpt-35-turbo".to_string())
        .interact_text()?;

    config.api_key = Some(api_key);
    config.api_url = Some(endpoint);
    config.model = Some(deployment);
    config.save()?;

    println!(
        "{}",
        "✓ Azure OpenAI configured successfully".green().bold()
    );
    Ok(())
}

/// Handle manual API key entry
async fn handle_manual_api_key(provider: &str) -> Result<()> {
    let api_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter {} API key", provider))
        .interact_text()?;

    if api_key.trim().is_empty() {
        println!("{}", "❌ API key cannot be empty".red());
        return Ok(());
    }

    let mut config = Config::load()?;
    config.api_key = Some(api_key);
    config.ai_provider = Some(provider.to_string());

    // Set default model and API URL based on provider
    match provider {
        "anthropic" => {
            config.model = Some("claude-3-5-haiku-20241022".to_string());
        }
        "openai" => {
            config.model = Some("gpt-4o-mini".to_string());
        }
        "gemini" => {
            config.model = Some("gemini-1.5-pro".to_string());
            config.api_url = Some("https://generativelanguage.googleapis.com/v1beta".to_string());
        }
        "openrouter" => {
            config.model = Some("openai/gpt-4o-mini".to_string());
            config.api_url = Some("https://openrouter.ai/api/v1".to_string());
        }
        "perplexity" => {
            config.model = Some("llama-3.1-sonar-small-128k-online".to_string());
            config.api_url = Some("https://api.perplexity.ai".to_string());
        }
        "groq" => {
            config.model = Some("llama-3.1-70b-versatile".to_string());
            config.api_url = Some("https://api.groq.com/openai/v1".to_string());
        }
        "deepseek" => {
            config.model = Some("deepseek-chat".to_string());
            config.api_url = Some("https://api.deepseek.com".to_string());
        }
        "mistral" => {
            config.model = Some("mistral-large-latest".to_string());
            config.api_url = Some("https://api.mistral.ai/v1".to_string());
        }
        "together" => {
            config.model = Some("meta-llama/Llama-3.2-3B-Instruct-Turbo".to_string());
            config.api_url = Some("https://api.together.xyz/v1".to_string());
        }
        "deepinfra" => {
            config.model = Some("meta-llama/Llama-3.2-3B-Instruct".to_string());
            config.api_url = Some("https://api.deepinfra.com/v1/openai".to_string());
        }
        "huggingface" => {
            config.model = Some("meta-llama/Llama-3.2-3B-Instruct".to_string());
            config.api_url = Some("https://api-inference.huggingface.co/v1".to_string());
        }
        "github-models" => {
            config.model = Some("gpt-4o".to_string());
            config.api_url = Some("https://models.inference.ai.azure.com".to_string());
        }
        _ => {}
    }

    config.save()?;

    println!(
        "{}",
        format!("✓ {} API key configured successfully", provider)
            .green()
            .bold()
    );
    Ok(())
}

/// Handle GitHub Copilot authentication
async fn handle_github_copilot_auth() -> Result<()> {
    println!("\n{}", "🐙 GitHub Copilot Authentication".cyan().bold());
    println!(
        "{}",
        "GitHub Copilot provides free AI assistance to subscribers".dimmed()
    );

    // TODO: Implement GitHub OAuth device flow
    println!(
        "{}",
        "⚠️  GitHub OAuth device flow not yet implemented".yellow()
    );
    println!("{}", "Please use GitHub CLI: gh auth login".cyan());

    let mut config = Config::load()?;
    config.ai_provider = Some("github-copilot".to_string());
    config.model = Some("gpt-4o".to_string());
    config.save()?;

    println!(
        "{}",
        "✓ GitHub Copilot configured (requires GitHub CLI auth)"
            .green()
            .bold()
    );
    Ok(())
}

/// Handle OpenRouter authentication
async fn handle_openrouter_auth() -> Result<()> {
    println!("\n{}", "🔄 OpenRouter Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://openrouter.ai/keys".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("openrouter".to_string());
    config.model = Some("openai/gpt-4o".to_string());
    config.api_url = Some("https://openrouter.ai/api/v1".to_string());

    handle_manual_api_key("openrouter").await
}

/// Handle Groq authentication
async fn handle_groq_auth() -> Result<()> {
    println!("\n{}", "⚡ Groq Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://console.groq.com/keys".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("groq".to_string());
    config.model = Some("llama-3.1-70b-versatile".to_string());
    config.api_url = Some("https://api.groq.com/openai/v1".to_string());

    handle_manual_api_key("groq").await
}

/// Handle DeepSeek authentication
async fn handle_deepseek_auth() -> Result<()> {
    println!("\n{}", "🧠 DeepSeek Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://platform.deepseek.com/api_keys".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("deepseek".to_string());
    config.model = Some("deepseek-chat".to_string());
    config.api_url = Some("https://api.deepseek.com".to_string());

    handle_manual_api_key("deepseek").await
}

/// Handle Mistral authentication
async fn handle_mistral_auth() -> Result<()> {
    println!("\n{}", "🌪️ Mistral AI Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://console.mistral.ai/".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("mistral".to_string());
    config.model = Some("mistral-large-latest".to_string());
    config.api_url = Some("https://api.mistral.ai/v1".to_string());

    handle_manual_api_key("mistral").await
}

/// Handle AWS Bedrock authentication
async fn handle_aws_bedrock_auth() -> Result<()> {
    println!("\n{}", "☁️ AWS Bedrock Authentication".cyan().bold());
    println!(
        "{}",
        "AWS Bedrock supports multiple authentication methods".dimmed()
    );

    let auth_methods = vec![
        "API Key (Bedrock) - Recommended for quick setup",
        "AWS Profile - Use configured AWS profile",
        "Environment Variables - AWS_ACCESS_KEY_ID & AWS_SECRET_ACCESS_KEY",
        "IAM Role - For EC2/Lambda environments",
    ];

    let method = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose AWS authentication method")
        .items(&auth_methods)
        .default(0)
        .interact()?;

    let mut config = Config::load()?;
    config.ai_provider = Some("amazon-bedrock".to_string());
    config.model = Some("us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string());

    match method {
        0 => {
            println!(
                "{}",
                "Enter your AWS Bedrock API key (new feature in 2025)".cyan()
            );
            println!(
                "{}",
                "This will be stored in AWS_BEARER_TOKEN_BEDROCK".dimmed()
            );
            handle_manual_api_key_with_env("amazon-bedrock", "AWS_BEARER_TOKEN_BEDROCK").await
        }
        1 => {
            let profile: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter AWS profile name")
                .default("default".to_string())
                .interact_text()?;

            std::env::set_var("AWS_PROFILE", &profile);
            println!(
                "{}",
                format!("✓ AWS Bedrock configured with profile: {}", profile)
                    .green()
                    .bold()
            );
            config.save()?;
            Ok(())
        }
        2 => {
            println!("{}", "Please set these environment variables:".cyan());
            println!("  export AWS_ACCESS_KEY_ID=your_access_key");
            println!("  export AWS_SECRET_ACCESS_KEY=your_secret_key");
            println!("  export AWS_REGION=us-east-1  # optional");
            println!(
                "{}",
                "✓ AWS Bedrock configured for environment variables"
                    .green()
                    .bold()
            );
            config.save()?;
            Ok(())
        }
        3 => {
            println!("{}", "✓ AWS Bedrock configured for IAM role".green().bold());
            println!("  Ensure your EC2/Lambda role has bedrock:InvokeModel permissions");
            config.save()?;
            Ok(())
        }
        _ => unreachable!(),
    }
}

/// Handle Together AI authentication
async fn handle_together_auth() -> Result<()> {
    println!("\n{}", "🤝 Together AI Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://api.together.xyz/settings/api-keys".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("together".to_string());
    config.model = Some("meta-llama/Llama-3.2-3B-Instruct-Turbo".to_string());
    config.api_url = Some("https://api.together.xyz/v1".to_string());

    handle_manual_api_key("together").await
}

/// Handle DeepInfra authentication
async fn handle_deepinfra_auth() -> Result<()> {
    println!("\n{}", "🏗️ DeepInfra Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://deepinfra.com/dash/api_keys".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("deepinfra".to_string());
    config.model = Some("meta-llama/Llama-3.2-3B-Instruct".to_string());
    config.api_url = Some("https://api.deepinfra.com/v1/openai".to_string());

    handle_manual_api_key("deepinfra").await
}

/// Handle Hugging Face authentication
async fn handle_huggingface_auth() -> Result<()> {
    println!("\n{}", "🤗 Hugging Face Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://huggingface.co/settings/tokens".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("huggingface".to_string());
    config.model = Some("meta-llama/Llama-3.2-3B-Instruct".to_string());
    config.api_url = Some("https://api-inference.huggingface.co/v1".to_string());

    handle_manual_api_key("huggingface").await
}

/// Handle GitHub Models authentication
async fn handle_github_models_auth() -> Result<()> {
    println!("\n{}", "🐙 GitHub Models Authentication".cyan().bold());
    println!(
        "{}",
        "Get your token from: https://github.com/settings/personal-access-tokens".cyan()
    );
    println!("{}", "Requires 'Model Inference' permission".dimmed());

    let mut config = Config::load()?;
    config.ai_provider = Some("github-models".to_string());
    config.model = Some("gpt-4o".to_string());
    config.api_url = Some("https://models.inference.ai.azure.com".to_string());

    handle_manual_api_key("github-models").await
}

/// Handle manual API key entry with custom environment variable
async fn handle_manual_api_key_with_env(provider: &str, env_var: &str) -> Result<()> {
    let api_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Enter {} API key", provider))
        .interact_text()?;

    if api_key.trim().is_empty() {
        println!("{}", "❌ API key cannot be empty".red());
        return Ok(());
    }

    // Set environment variable
    std::env::set_var(env_var, &api_key);

    let mut config = Config::load()?;
    // For environment variable based auth, we don't store the key in config
    config.ai_provider = Some(provider.to_string());
    config.save()?;

    println!(
        "{}",
        format!(
            "✓ {} configured with environment variable {}",
            provider, env_var
        )
        .green()
        .bold()
    );
    println!(
        "{}",
        format!(
            "  Environment variable {} has been set for this session",
            env_var
        )
        .dimmed()
    );
    Ok(())
}

/// Handle manual/other provider configuration
async fn handle_manual_auth() -> Result<()> {
    println!("\n{}", "🔧 Custom Provider Configuration".cyan().bold());
    println!("{}", "Configure any OpenAI-compatible provider".dimmed());

    let provider: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter provider name")
        .interact_text()?;

    let api_url: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter API base URL")
        .default("https://api.openai.com/v1".to_string())
        .interact_text()?;

    let model: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter default model name")
        .default("gpt-3.5-turbo".to_string())
        .interact_text()?;

    let mut config = Config::load()?;
    config.ai_provider = Some(provider.clone());
    config.api_url = Some(api_url);
    config.model = Some(model);
    config.save()?;

    handle_manual_api_key(&provider).await
}

/// Logout and remove stored tokens
async fn logout() -> Result<()> {
    println!("{}", "🔐 Logging out...".cyan());

    // Remove stored tokens
    token_storage::delete_tokens()?;

    println!("{}", "✓ Successfully logged out".green().bold());
    println!("  Your authentication tokens have been removed.");

    Ok(())
}

/// Check authentication status
async fn status() -> Result<()> {
    println!("{}", "🔐 Authentication Status".cyan().bold());
    println!("{}", "─".repeat(40));

    let config = Config::load()?;

    // Check for API key
    if config.api_key.is_some() {
        println!("{}", "✓ API Key configured".green());
        println!(
            "  Provider: {}",
            config.ai_provider.as_deref().unwrap_or("openai")
        );
        return Ok(());
    }

    // Check for OAuth tokens
    if let Some(tokens) = token_storage::get_tokens()? {
        println!("{}", "✓ Authenticated with Claude OAuth".green());

        // Check token expiry
        if tokens.is_expired() {
            println!("{}", "  ⚠ Token expired - please re-authenticate".yellow());
        } else if let Some(expires_at) = tokens.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let remaining = expires_at - now;
            let hours = remaining / 3600;
            let minutes = (remaining % 3600) / 60;
            println!("  Token expires in: {}h {}m", hours, minutes);
        }

        if tokens.refresh_token.is_some() {
            println!("  Refresh token: {}", "Available".green());
        }

        // Show where tokens are stored
        #[cfg(feature = "secure-storage")]
        if crate::config::secure_storage::is_available() {
            println!("  Storage: {}", "System Keychain".green());
        } else {
            println!("  Storage: {}", "~/.config/rustycommit/auth.json".yellow());
        }

        #[cfg(not(feature = "secure-storage"))]
        println!("  Storage: {}", "~/.config/rustycommit/auth.json".yellow());
    } else {
        println!("{}", "✗ Not authenticated".red());
        println!("\n{}", "To authenticate, run one of:".yellow());
        println!(
            "  • {} - Use Claude OAuth (recommended for Pro/Max users)",
            "rco auth login".cyan()
        );
        println!(
            "  • {} - Use API key",
            "rco config set RCO_API_KEY=<your_key>".cyan()
        );
    }

    println!("\n{}", "Storage Information:".bold());
    #[cfg(feature = "secure-storage")]
    println!("  {}", crate::config::secure_storage::status_message());

    #[cfg(not(feature = "secure-storage"))]
    println!("  Using file-based storage at ~/.config/rustycommit/auth.json");

    Ok(())
}

/// Automatically refresh token if needed
pub async fn auto_refresh_token() -> Result<()> {
    // Check if we have tokens and they're expiring soon
    if let Some(tokens) = token_storage::get_tokens()? {
        if tokens.expires_soon() {
            if let Some(refresh_token) = &tokens.refresh_token {
                let oauth_client = OAuthClient::new();
                match oauth_client.refresh_token(refresh_token).await {
                    Ok(token_response) => {
                        // Update stored tokens
                        token_storage::store_tokens(
                            &token_response.access_token,
                            token_response
                                .refresh_token
                                .as_deref()
                                .or(Some(refresh_token)),
                            token_response.expires_in,
                        )?;

                        tracing::debug!("Successfully refreshed OAuth token");
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to refresh token: {}. User may need to re-authenticate.",
                            e
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle Perplexity authentication
async fn handle_perplexity_auth() -> Result<()> {
    println!("\n{}", "🔍 Perplexity Authentication".cyan().bold());
    println!(
        "{}",
        "Get your API key from: https://www.perplexity.ai/settings/api".cyan()
    );

    let mut config = Config::load()?;
    config.ai_provider = Some("perplexity".to_string());
    config.model = Some("llama-3.1-sonar-small-128k-online".to_string());
    config.api_url = Some("https://api.perplexity.ai".to_string());

    handle_manual_api_key("perplexity").await
}
