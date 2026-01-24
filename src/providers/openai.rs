use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use super::{build_prompt, AIProvider};
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
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            ChatCompletionRequestMessage::System(
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are an expert at writing clear, concise git commit messages.")
                    .build()?,
            ),
            ChatCompletionRequestMessage::User(
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?,
            ),
        ];

        // Handle model-specific parameters
        let request = if self.model.contains("gpt-5-nano") {
            // GPT-5-nano doesn't support temperature=0, use 1.0 (default)
            // Note: For now we use regular max_tokens until async-openai supports max_completion_tokens
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
