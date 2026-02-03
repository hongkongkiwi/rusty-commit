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

use super::prompt::split_prompt;
use super::AIProvider;
use crate::config::accounts::AccountConfig;
use crate::config::Config;

pub struct XAIProvider {
    client: Client<OpenAIConfig>,
    model: String,
}

impl XAIProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("xAI API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://x.ai/api")?;

        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(config.api_url.as_deref().unwrap_or("https://api.x.ai/v1"));

        let client = Client::with_config(openai_config);
        let model = config.model.clone();

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
                .unwrap_or("https://api.x.ai/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = account
            .model
            .as_deref()
            .unwrap_or(&config.model)
            .to_string();

        Ok(Self { client, model })
    }
}

#[async_trait]
impl AIProvider for XAIProvider {
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

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.7)
            .max_tokens(config.tokens_max_output as u16)
            .build()?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .context("Failed to generate commit message from xAI")?;

        let message = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("xAI returned an empty response")?
            .trim()
            .to_string();

        Ok(message)
    }
}

/// ProviderBuilder for XAI
pub struct XAIProviderBuilder;

impl super::registry::ProviderBuilder for XAIProviderBuilder {
    fn name(&self) -> &'static str {
        "xai"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["grok", "x-ai"]
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(XAIProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("grok-beta")
    }
}
