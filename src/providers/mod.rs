// AI Provider modules - conditionally compiled based on features
#[cfg(feature = "anthropic")]
pub mod anthropic;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "gemini")]
pub mod gemini;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "perplexity")]
pub mod perplexity;
#[cfg(feature = "xai")]
pub mod xai;

use crate::config::accounts::AccountConfig;
use crate::config::Config;
use anyhow::{Context, Result};
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String>;

    /// Generate multiple commit message variations
    async fn generate_commit_messages(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
        count: u8,
    ) -> Result<Vec<String>> {
        let mut messages = Vec::with_capacity(count as usize);
        for _ in 0..count {
            match self
                .generate_commit_message(diff, context, full_gitmoji, config)
                .await
            {
                Ok(msg) => messages.push(msg),
                Err(e) => tracing::warn!("Failed to generate message: {}", e),
            }
        }
        Ok(messages)
    }

    /// Generate a PR description from commits
    #[cfg(any(feature = "openai", feature = "xai"))]
    async fn generate_pr_description(
        &self,
        commits: &[String],
        diff: &str,
        config: &Config,
    ) -> Result<String> {
        let commits_text = commits.join("\n");
        let prompt = format!(
            "Generate a professional pull request description based on the following commits:\n\n{}\n\nDiff:\n{}\n\nFormat the output as:\n## Summary\n## Changes\n## Testing\n## Breaking Changes\n\nKeep it concise and informative.",
            commits_text, diff
        );

        let messages = vec![
            async_openai::types::chat::ChatCompletionRequestSystemMessage::from(
                "You are an expert at writing pull request descriptions.",
            )
            .into(),
            async_openai::types::chat::ChatCompletionRequestUserMessage::from(prompt).into(),
        ];

        let request = async_openai::types::chat::CreateChatCompletionRequestArgs::default()
            .model(
                config
                    .model
                    .clone()
                    .unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
            )
            .messages(messages)
            .temperature(0.7)
            .max_tokens(config.tokens_max_output.unwrap_or(1000) as u16)
            .build()?;

        // Create a new client for this request
        let api_key = config
            .api_key
            .as_ref()
            .context("API key not configured. Run: rco config set RCO_API_KEY=<your_key>")?;
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1");

        let openai_config = async_openai::config::OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_url);

        let client = async_openai::Client::with_config(openai_config);

        let response = client.chat().create(request).await?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("AI returned an empty response")?
            .trim()
            .to_string();

        Ok(message)
    }

    /// Generate a PR description - stub when OpenAI/xAI features are disabled
    #[cfg(not(any(feature = "openai", feature = "xai")))]
    async fn generate_pr_description(
        &self,
        _commits: &[String],
        _diff: &str,
        _config: &Config,
    ) -> Result<String> {
        anyhow::bail!(
            "PR description generation requires the 'openai' or 'xai' feature to be enabled"
        );
    }
}

pub fn create_provider(config: &Config) -> Result<Box<dyn AIProvider>> {
    let provider = config.ai_provider.as_deref().unwrap_or("openai");

    match provider.to_lowercase().as_str() {
        #[cfg(feature = "openai")]
        "openai" => Ok(Box::new(openai::OpenAIProvider::new(config)?)),
        #[cfg(feature = "anthropic")]
        "anthropic" | "claude" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        #[cfg(feature = "ollama")]
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        #[cfg(feature = "gemini")]
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(config)?)),
        #[cfg(feature = "azure")]
        "azure" | "azure-openai" => Ok(Box::new(azure::AzureProvider::new(config)?)),
        #[cfg(feature = "perplexity")]
        "perplexity" => Ok(Box::new(perplexity::PerplexityProvider::new(config)?)),
        #[cfg(feature = "xai")]
        "xai" | "grok" | "x-ai" => Ok(Box::new(xai::XAIProvider::new(config)?)),
        // OpenAI-compatible providers (use openai module)
        #[cfg(feature = "openai")]
        "deepseek" | "groq" | "openrouter" | "together" | "deepinfra" | "huggingface"
        | "mistral" | "github-models" | "amazon-bedrock" | "fireworks" | "fireworks-ai"
        | "moonshot" | "moonshot-ai" | "dashscope" | "alibaba" | "qwen" | "qwen-coder"
        | "vertex" | "vertex-ai" | "google-vertex" | "codex" => {
            Ok(Box::new(openai::OpenAIProvider::new(config)?))
        }
        _ => {
            // Build available providers list based on enabled features
            #[allow(clippy::useless_vec)]
            let available = vec![
                #[cfg(feature = "openai")]
                "openai",
                #[cfg(feature = "anthropic")]
                "anthropic / claude",
                #[cfg(feature = "ollama")]
                "ollama",
                #[cfg(feature = "gemini")]
                "gemini",
                #[cfg(feature = "azure")]
                "azure",
                #[cfg(feature = "perplexity")]
                "perplexity",
                #[cfg(feature = "xai")]
                "xai / grok",
                #[cfg(feature = "openai")]
                "deepseek, groq, openrouter, together, deepseek (OpenAI-compatible)",
            ];

            if available.is_empty() {
                anyhow::bail!(
                    "No AI provider features enabled. Please enable at least one provider feature: \
                     --features openai,anthropic,ollama,gemini,azure,perplexity,xai"
                );
            }

            anyhow::bail!(
                "Unsupported or disabled AI provider: {}\n\n\
                 Available providers (based on enabled features):\n{}\n\n\
                 Set with: rco config set RCO_AI_PROVIDER=<provider_name>",
                provider,
                available
                    .iter()
                    .map(|p| format!("- {}", p))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        }
    }
}

pub fn build_prompt(
    diff: &str,
    context: Option<&str>,
    config: &Config,
    full_gitmoji: bool,
) -> String {
    let mut prompt = String::new();

    // System message
    prompt.push_str("You are an expert at writing clear, concise git commit messages.\n\n");

    // Add locale if specified
    if let Some(locale) = &config.language {
        prompt.push_str(&format!(
            "Generate the commit message in {locale} language.\n"
        ));
    }

    // Add commit type preference
    let commit_type = config.commit_type.as_deref().unwrap_or("conventional");
    match commit_type {
        "conventional" => {
            prompt.push_str("Use conventional commit format (type(scope): description).\n");
            prompt.push_str(
                "Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore\n",
            );
        }
        "gitmoji" => {
            if full_gitmoji {
                prompt.push_str("Use GitMoji format with the full emoji specification from https://gitmoji.dev/\n");
            } else {
                prompt.push_str("Use GitMoji format with these emojis: 🐛 (fix), ✨ (feat), 📝 (docs), 🚀 (deploy), ✅ (test), ♻️ (refactor), ⬆️ (upgrade), 🔧 (config), 🌐 (i18n), 💡 (comments)\n");
            }
        }
        _ => {}
    }

    // Add description requirements
    let max_length = config.description_max_length.unwrap_or(100);
    prompt.push_str(&format!(
        "Keep the commit message under {max_length} characters.\n"
    ));

    if config.description_capitalize.unwrap_or(true) {
        prompt.push_str("Capitalize the first letter of the description.\n");
    }

    if !config.description_add_period.unwrap_or(false) {
        prompt.push_str("Do not end the description with a period.\n");
    }

    // Add context if provided
    if let Some(ctx) = context {
        prompt.push_str(&format!("\nAdditional context: {ctx}\n"));
    }

    // Add the diff
    prompt.push_str("\nGenerate a commit message for the following git diff:\n\n");
    prompt.push_str("```diff\n");
    prompt.push_str(diff);
    prompt.push_str("\n```\n\n");

    prompt.push_str(
        "Return ONLY the commit message, without any additional explanation or formatting.",
    );

    prompt
}

/// Create an AI provider from an account configuration
#[allow(dead_code)]
pub fn create_provider_for_account(
    account: &AccountConfig,
    config: &Config,
) -> Result<Box<dyn AIProvider>> {
    use crate::auth::token_storage;
    use crate::config::secure_storage;

    let provider = account.provider.to_lowercase();

    // Extract credentials from the account's auth method
    let credentials = match &account.auth {
        crate::config::accounts::AuthMethod::ApiKey { key_id } => {
            // Get API key from secure storage using the account's key_id
            token_storage::get_api_key_for_account(key_id)?
                .or_else(|| secure_storage::get_secret(key_id).ok().flatten())
        }
        crate::config::accounts::AuthMethod::OAuth {
            provider: _oauth_provider,
            account_id,
        } => {
            // Get OAuth access token from secure storage
            token_storage::get_tokens_for_account(account_id)?.map(|t| t.access_token)
        }
        crate::config::accounts::AuthMethod::EnvVar { name } => std::env::var(name).ok(),
        crate::config::accounts::AuthMethod::Bearer { token_id } => {
            // Get bearer token from secure storage
            token_storage::get_bearer_token_for_account(token_id)?
                .or_else(|| secure_storage::get_secret(token_id).ok().flatten())
        }
    };

    match provider.as_str() {
        #[cfg(feature = "openai")]
        "openai" | "codex" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(openai::OpenAIProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(openai::OpenAIProvider::new(config)?))
            }
        }
        #[cfg(feature = "anthropic")]
        "anthropic" | "claude" | "claude-code" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(anthropic::AnthropicProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(anthropic::AnthropicProvider::new(config)?))
            }
        }
        #[cfg(feature = "ollama")]
        "ollama" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(ollama::OllamaProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(ollama::OllamaProvider::new(config)?))
            }
        }
        #[cfg(feature = "gemini")]
        "gemini" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(gemini::GeminiProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(gemini::GeminiProvider::new(config)?))
            }
        }
        #[cfg(feature = "azure")]
        "azure" | "azure-openai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(azure::AzureProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(azure::AzureProvider::new(config)?))
            }
        }
        #[cfg(feature = "perplexity")]
        "perplexity" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(perplexity::PerplexityProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(perplexity::PerplexityProvider::new(config)?))
            }
        }
        #[cfg(feature = "xai")]
        "xai" | "grok" | "x-ai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(xai::XAIProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(xai::XAIProvider::new(config)?))
            }
        }
        _ => {
            anyhow::bail!(
                "Unsupported AI provider for account: {}\n\n\
                 Account provider: {}\n\
                 Supported providers: openai, anthropic, ollama, gemini, azure, perplexity, xai",
                account.alias,
                provider
            );
        }
    }
}
