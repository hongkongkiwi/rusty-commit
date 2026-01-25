use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::chat::{
        ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use super::{split_prompt, AIProvider};
use crate::config::accounts::AccountConfig;
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct OpenAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenAIProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("OpenAI API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://platform.openai.com/api-keys")?;

        let openai_config = OpenAIConfig::new().with_api_key(api_key).with_api_base(
            config
                .api_url
                .as_deref()
                .unwrap_or("https://api.openai.com/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = config
            .model
            .as_deref()
            .unwrap_or("gpt-3.5-turbo")
            .to_string();

        Ok(Self { client, model })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(account: &AccountConfig, api_key: &str, config: &Config) -> Result<Self> {
        let openai_config = OpenAIConfig::new().with_api_key(api_key).with_api_base(
            account
                .api_url
                .as_deref()
                .or(config.api_url.as_deref())
                .unwrap_or("https://api.openai.com/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("gpt-3.5-turbo")
            .to_string();

        Ok(Self { client, model })
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            ChatCompletionRequestSystemMessage::from(system_prompt).into(),
            ChatCompletionRequestUserMessage::from(user_prompt).into(),
        ];

        // Handle model-specific parameters
        let request = if self.model.contains("gpt-5-nano") {
            // GPT-5-nano doesn't support temperature=0, use 1.0 (default)
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .temperature(1.0)
                .max_tokens(config.tokens_max_output.unwrap_or(500) as u16)
                .build()?
        } else {
            // Standard models support temperature=0.7 and max_tokens
            CreateChatCompletionRequestArgs::default()
                .model(&self.model)
                .messages(messages)
                .temperature(0.7)
                .max_tokens(config.tokens_max_output.unwrap_or(500) as u16)
                .build()?
        };

        let response = retry_async(|| async {
            match self.client.chat().create(request.clone()).await {
                Ok(resp) => Ok(resp),
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("401") || error_msg.contains("invalid_api_key") {
                        Err(anyhow::anyhow!("Invalid OpenAI API key. Please check your API key configuration."))
                    } else if error_msg.contains("insufficient_quota") {
                        Err(anyhow::anyhow!("OpenAI API quota exceeded. Please check your billing status."))
                    } else {
                        Err(anyhow::anyhow!(e).context("Failed to generate commit message from OpenAI"))
                    }
                }
            }
        }).await.context("Failed to generate commit message from OpenAI after retries. Please check your internet connection and API configuration.")?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("OpenAI returned an empty response. The model may be overloaded - please try again.")?
            .trim()
            .to_string();

        Ok(message)
    }
}

/// OpenAICompatibleProvider - A wrapper that handles OpenAI-compatible providers
/// This struct registers all OpenAI-compatible API providers in the registry
pub struct OpenAICompatibleProvider {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub default_api_url: &'static str,
    pub default_model: Option<&'static str>,
    pub compatible_providers: std::collections::HashMap<&'static str, &'static str>,
}

impl OpenAICompatibleProvider {
    pub fn new() -> Self {
        let mut compat = std::collections::HashMap::new();
        compat.insert("deepseek", "https://api.deepseek.com/v1");
        compat.insert("groq", "https://api.groq.com/openai/v1");
        compat.insert("openrouter", "https://openrouter.ai/api/v1");
        compat.insert("together", "https://api.together.ai/v1");
        compat.insert("deepinfra", "https://api.deepinfra.com/v1/openai");
        compat.insert("mistral", "https://api.mistral.ai/v1");
        compat.insert("github-models", "https://models.inference.ai.azure.com");
        compat.insert("fireworks", "https://api.fireworks.ai/v1");
        compat.insert("fireworks-ai", "https://api.fireworks.ai/v1");
        compat.insert("moonshot", "https://api.moonshot.cn/v1");
        compat.insert("moonshot-ai", "https://api.moonshot.cn/v1");
        compat.insert("dashscope", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("alibaba", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("qwen", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("qwen-coder", "https://dashscope.console.aliyuncs.com/api/v1");
        compat.insert("codex", "https://api.openai.com/v1");

        Self {
            name: "openai",
            aliases: vec!["openai"],
            default_api_url: "https://api.openai.com/v1",
            default_model: Some("gpt-4o-mini"),
            compatible_providers: compat,
        }
    }
}

impl Default for OpenAICompatibleProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl super::registry::ProviderBuilder for OpenAICompatibleProvider {
    fn name(&self) -> &'static str {
        self.name
    }

    fn aliases(&self) -> Vec<&'static str> {
        self.aliases.clone()
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::OpenAICompatible
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(OpenAIProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        self.default_model
    }
}
