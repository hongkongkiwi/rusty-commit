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

use super::{build_prompt, AIProvider};
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

        let openai_config = OpenAIConfig::new().with_api_key(api_key).with_api_base(
            config
                .api_url
                .as_deref()
                .unwrap_or("https://api.x.ai/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = config
            .model
            .as_deref()
            .unwrap_or("grok-beta")
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
                .unwrap_or("https://api.x.ai/v1"),
        );

        let client = Client::with_config(openai_config);
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("grok-beta")
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
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            ChatCompletionRequestSystemMessage::from(
                "You are an expert at writing clear, concise git commit messages.",
            )
            .into(),
            ChatCompletionRequestUserMessage::from(prompt).into(),
        ];

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(0.7)
            .max_tokens(config.tokens_max_output.unwrap_or(500) as u16)
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
