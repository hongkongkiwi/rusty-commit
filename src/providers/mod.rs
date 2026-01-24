pub mod anthropic;
pub mod azure;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod perplexity;
pub mod xai;

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
            match self.generate_commit_message(diff, context, full_gitmoji, config).await {
                Ok(msg) => messages.push(msg),
                Err(e) => tracing::warn!("Failed to generate message: {}", e),
            }
        }
        Ok(messages)
    }

    /// Generate a PR description from commits
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
            async_openai::types::ChatCompletionRequestMessage::System(
                async_openai::types::ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are an expert at writing pull request descriptions.")
                    .build()?,
            ),
            async_openai::types::ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?,
            ),
        ];

        let request = async_openai::types::CreateChatCompletionRequestArgs::default()
            .model(&config.model.clone().unwrap_or_else(|| "gpt-3.5-turbo".to_string()))
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
}

pub fn create_provider(config: &Config) -> Result<Box<dyn AIProvider>> {
    let provider = config.ai_provider.as_deref().unwrap_or("openai");

    match provider.to_lowercase().as_str() {
        "openai" => Ok(Box::new(openai::OpenAIProvider::new(config)?)),
        "anthropic" | "claude" => Ok(Box::new(anthropic::AnthropicProvider::new(config)?)),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(config)?)),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::new(config)?)),
        "azure" | "azure-openai" => Ok(Box::new(azure::AzureProvider::new(config)?)),
        "perplexity" => Ok(Box::new(perplexity::PerplexityProvider::new(config)?)),
        "xai" | "grok" | "x-ai" => Ok(Box::new(xai::XAIProvider::new(config)?)),
        // OpenAI-compatible providers
        "deepseek" | "groq" | "openrouter" | "together" | "deepinfra" | "huggingface"
        | "mistral" | "github-models" | "amazon-bedrock" | "fireworks" | "fireworks-ai"
        | "moonshot" | "moonshot-ai" | "dashscope" | "alibaba" | "qwen" | "qwen-coder"
        | "vertex" | "vertex-ai" | "google-vertex" => Ok(Box::new(openai::OpenAIProvider::new(config)?)),
        _ => anyhow::bail!("Unsupported AI provider: {}", provider),
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
