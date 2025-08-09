pub mod anthropic;
pub mod azure;
pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod perplexity;

use crate::config::Config;
use anyhow::Result;
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
        // OpenAI-compatible providers
        "deepseek" | "groq" | "openrouter" | "together" | "deepinfra" | "huggingface"
        | "mistral" | "github-models" | "amazon-bedrock" => {
            Ok(Box::new(openai::OpenAIProvider::new(config)?))
        }
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
