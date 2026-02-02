// AI Provider modules - conditionally compiled based on features
#[cfg(feature = "anthropic")]
pub mod anthropic;
#[cfg(feature = "azure")]
pub mod azure;
#[cfg(feature = "bedrock")]
pub mod bedrock;
#[cfg(feature = "flowise")]
pub mod flowise;
#[cfg(feature = "gemini")]
pub mod gemini;
#[cfg(feature = "huggingface")]
pub mod huggingface;
#[cfg(feature = "mlx")]
pub mod mlx;
#[cfg(feature = "nvidia")]
pub mod nvidia;
#[cfg(feature = "ollama")]
pub mod ollama;
#[cfg(feature = "openai")]
pub mod openai;
#[cfg(feature = "perplexity")]
pub mod perplexity;
#[cfg(feature = "vertex")]
pub mod vertex;
#[cfg(feature = "xai")]
pub mod xai;

// Provider registry for extensible provider management
pub mod registry;

// Prompt building utilities
pub mod prompt;

use crate::config::accounts::AccountConfig;
use crate::config::Config;
use anyhow::{Context, Result};
use async_trait::async_trait;
use once_cell::sync::Lazy;

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
        use futures::stream::StreamExt;

        if count <= 1 {
            // For single message, no parallelism needed
            match self
                .generate_commit_message(diff, context, full_gitmoji, config)
                .await
            {
                Ok(msg) => Ok(vec![msg]),
                Err(e) => {
                    tracing::warn!("Failed to generate message: {}", e);
                    Ok(vec![])
                }
            }
        } else {
            // Generate messages in parallel using FuturesUnordered
            let futures = (0..count)
                .map(|_| self.generate_commit_message(diff, context, full_gitmoji, config));
            let mut stream = futures::stream::FuturesUnordered::from_iter(futures);

            let mut messages = Vec::with_capacity(count as usize);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => messages.push(msg),
                    Err(e) => tracing::warn!("Failed to generate message: {}", e),
                }
            }
            Ok(messages)
        }
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

/// Global provider registry - automatically populated based on enabled features
pub static PROVIDER_REGISTRY: Lazy<registry::ProviderRegistry> = Lazy::new(|| {
    let reg = registry::ProviderRegistry::new();

    // Register OpenAI-compatible providers (require openai feature)
    #[cfg(feature = "openai")]
    {
        let _ = reg.register(Box::new(openai::OpenAICompatibleProvider::new()));
    }

    // Register dedicated providers
    #[cfg(feature = "anthropic")]
    {
        let _ = reg.register(Box::new(anthropic::AnthropicProviderBuilder));
    }

    #[cfg(feature = "ollama")]
    {
        let _ = reg.register(Box::new(ollama::OllamaProviderBuilder));
    }

    #[cfg(feature = "gemini")]
    {
        let _ = reg.register(Box::new(gemini::GeminiProviderBuilder));
    }

    #[cfg(feature = "azure")]
    {
        let _ = reg.register(Box::new(azure::AzureProviderBuilder));
    }

    #[cfg(feature = "perplexity")]
    {
        let _ = reg.register(Box::new(perplexity::PerplexityProviderBuilder));
    }

    #[cfg(feature = "xai")]
    {
        let _ = reg.register(Box::new(xai::XAIProviderBuilder));
    }

    #[cfg(feature = "huggingface")]
    {
        let _ = reg.register(Box::new(huggingface::HuggingFaceProviderBuilder));
    }

    #[cfg(feature = "bedrock")]
    {
        let _ = reg.register(Box::new(bedrock::BedrockProviderBuilder));
    }

    #[cfg(feature = "vertex")]
    {
        let _ = reg.register(Box::new(vertex::VertexProviderBuilder));
    }

    #[cfg(feature = "mlx")]
    {
        let _ = reg.register(Box::new(mlx::MlxProviderBuilder));
    }

    #[cfg(feature = "nvidia")]
    {
        let _ = reg.register(Box::new(nvidia::NvidiaProviderBuilder));
    }

    #[cfg(feature = "flowise")]
    {
        let _ = reg.register(Box::new(flowise::FlowiseProviderBuilder));
    }

    reg
});

/// Create an AI provider instance from configuration
pub fn create_provider(config: &Config) -> Result<Box<dyn AIProvider>> {
    let provider_name = config.ai_provider.as_deref().unwrap_or("openai");

    // Try to create from registry
    if let Some(provider) = PROVIDER_REGISTRY.create(provider_name, config)? {
        return Ok(provider);
    }

    // Provider not found - build error message with available providers
    let available: Vec<String> = PROVIDER_REGISTRY
        .all()
        .unwrap_or_default()
        .iter()
        .map(|e| {
            let aliases = if e.aliases.is_empty() {
                String::new()
            } else {
                format!(" ({})", e.aliases.join(", "))
            };
            format!("- {}{}", e.name, aliases)
        })
        .chain(std::iter::once(format!(
            "- {} OpenAI-compatible providers (deepseek, groq, openrouter, etc.)",
            PROVIDER_REGISTRY
                .by_category(registry::ProviderCategory::OpenAICompatible)
                .map_or(0, |v| v.len())
        )))
        .filter(|s| !s.contains("0 OpenAI-compatible"))
        .collect();

    if available.is_empty() {
        anyhow::bail!(
            "No AI provider features enabled. Please enable at least one provider feature:\n\
             --features openai,anthropic,ollama,gemini,azure,perplexity,xai,huggingface,bedrock,vertex"
        );
    }

    anyhow::bail!(
        "Unsupported or disabled AI provider: {}\n\n\
         Available providers (based on enabled features):\n{}\n\n\
         Set with: rco config set RCO_AI_PROVIDER=<provider_name>",
        provider_name,
        available.join("\n")
    )
}

#[allow(dead_code)]
/// Get list of all available provider names
pub fn available_providers() -> Vec<&'static str> {
    let mut providers = PROVIDER_REGISTRY
        .all()
        .unwrap_or_default()
        .iter()
        .flat_map(|e| std::iter::once(e.name).chain(e.aliases.iter().copied()))
        .collect::<Vec<_>>();

    #[cfg(feature = "openai")]
    {
        providers.extend_from_slice(&[
            // ═════════════════════════════════════════════════════════════════
            // Major Cloud Providers
            // ═════════════════════════════════════════════════════════════════
            "deepseek",
            "groq",
            "openrouter",
            "together",
            "deepinfra",
            "mistral",
            "github-models",
            "fireworks",
            "moonshot",
            "dashscope",
            "perplexity",
            // ═════════════════════════════════════════════════════════════════
            // Enterprise & Specialized
            // ═════════════════════════════════════════════════════════════════
            "cohere",
            "cohere-ai",
            "ai21",
            "ai21-labs",
            "upstage",
            "upstage-ai",
            "solar",
            "solar-pro",
            // ═════════════════════════════════════════════════════════════════
            // GPU Cloud & Inference Providers
            // ═════════════════════════════════════════════════════════════════
            "nebius",
            "nebius-ai",
            "nebius-studio",
            "ovh",
            "ovhcloud",
            "ovh-ai",
            "scaleway",
            "scaleway-ai",
            "friendli",
            "friendli-ai",
            "baseten",
            "baseten-ai",
            "chutes",
            "chutes-ai",
            "ionet",
            "io-net",
            "modelscope",
            "requesty",
            "morph",
            "morph-labs",
            "synthetic",
            "nano-gpt",
            "nanogpt",
            "zenmux",
            "v0",
            "v0-vercel",
            "iflowcn",
            "venice",
            "venice-ai",
            "cortecs",
            "cortecs-ai",
            "kimi-coding",
            "abacus",
            "abacus-ai",
            "bailing",
            "fastrouter",
            "inference",
            "inference-net",
            "submodel",
            "zai",
            "zai-coding",
            "zhipu-coding",
            "poe",
            "poe-ai",
            "cerebras",
            "cerebras-ai",
            "sambanova",
            "sambanova-ai",
            "novita",
            "novita-ai",
            "predibase",
            "tensorops",
            "hyperbolic",
            "hyperbolic-ai",
            "kluster",
            "kluster-ai",
            "lambda",
            "lambda-labs",
            "replicate",
            "targon",
            "corcel",
            "cybernative",
            "cybernative-ai",
            "edgen",
            "gigachat",
            "gigachat-ai",
            "hydra",
            "hydra-ai",
            "jina",
            "jina-ai",
            "lingyi",
            "lingyiwanwu",
            "monica",
            "monica-ai",
            "pollinations",
            "pollinations-ai",
            "rawechat",
            "shuttleai",
            "shuttle-ai",
            "teknium",
            "theb",
            "theb-ai",
            "tryleap",
            "leap-ai",
            // ═════════════════════════════════════════════════════════════════
            // Local/Self-hosted Providers
            // ═════════════════════════════════════════════════════════════════
            "lmstudio",
            "lm-studio",
            "llamacpp",
            "llama-cpp",
            "kobold",
            "koboldcpp",
            "textgen",
            "text-generation",
            "tabby",
            // ═════════════════════════════════════════════════════════════════
            // China-based Providers
            // ═════════════════════════════════════════════════════════════════
            "siliconflow",
            "silicon-flow",
            "zhipu",
            "zhipu-ai",
            "bigmodel",
            "minimax",
            "minimax-ai",
            "glm",
            "chatglm",
            "baichuan",
            "01-ai",
            "yi",
            // ═════════════════════════════════════════════════════════════════
            // AI Gateway & Proxy Services
            // ═════════════════════════════════════════════════════════════════
            "helicone",
            "helicone-ai",
            "workers-ai",
            "cloudflare-ai",
            "cloudflare-gateway",
            "vercel-ai",
            "vercel-gateway",
            // ═════════════════════════════════════════════════════════════════
            // Specialized Providers
            // ═════════════════════════════════════════════════════════════════
            "302ai",
            "302-ai",
            "sap-ai",
            "sap-ai-core",
            // ═════════════════════════════════════════════════════════════════
            // Additional Providers from OpenCommit
            // ═════════════════════════════════════════════════════════════════
            "aimlapi",
            "ai-ml-api",
        ]);
    }

    providers
}

/// Get provider info for display
#[allow(dead_code)]
pub fn provider_info(provider: &str) -> Option<String> {
    PROVIDER_REGISTRY.get(provider).map(|e| {
        let aliases = if e.aliases.is_empty() {
            String::new()
        } else {
            format!(" (aliases: {})", e.aliases.join(", "))
        };
        let model = e
            .default_model
            .map(|m| format!(", default model: {}", m))
            .unwrap_or_default();
        format!("{}{}{}", e.name, aliases, model)
    })
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
        #[cfg(feature = "huggingface")]
        "huggingface" | "hf" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(huggingface::HuggingFaceProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(huggingface::HuggingFaceProvider::new(config)?))
            }
        }
        #[cfg(feature = "bedrock")]
        "bedrock" | "aws-bedrock" | "amazon-bedrock" => Ok(Box::new(
            bedrock::BedrockProvider::from_account(account, "", config)?,
        )),
        #[cfg(feature = "vertex")]
        "vertex" | "vertex-ai" | "google-vertex" | "gcp-vertex" => Ok(Box::new(
            vertex::VertexProvider::from_account(account, "", config)?,
        )),
        #[cfg(feature = "mlx")]
        "mlx" | "mlx-lm" | "apple-mlx" => {
            if let Some(_key) = credentials.as_ref() {
                Ok(Box::new(mlx::MlxProvider::from_account(
                    account, "", config,
                )?))
            } else {
                Ok(Box::new(mlx::MlxProvider::new(config)?))
            }
        }
        #[cfg(feature = "nvidia")]
        "nvidia" | "nvidia-nim" | "nim" | "nvidia-ai" => {
            if let Some(key) = credentials.as_ref() {
                Ok(Box::new(nvidia::NvidiaProvider::from_account(
                    account, key, config,
                )?))
            } else {
                Ok(Box::new(nvidia::NvidiaProvider::new(config)?))
            }
        }
        #[cfg(feature = "flowise")]
        "flowise" | "flowise-ai" => {
            if let Some(_key) = credentials.as_ref() {
                Ok(Box::new(flowise::FlowiseProvider::from_account(
                    account, "", config,
                )?))
            } else {
                Ok(Box::new(flowise::FlowiseProvider::new(config)?))
            }
        }
        _ => {
            anyhow::bail!(
                "Unsupported AI provider for account: {}\n\n\
                 Account provider: {}\n\
                 Supported providers: openai, anthropic, ollama, gemini, azure, perplexity, xai, huggingface, bedrock, vertex",
                account.alias,
                provider
            );
        }
    }
}
