use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_config::Region;
use aws_sdk_bedrockruntime as bedrock;
use aws_sdk_bedrockruntime::types::{ContentBlock, SystemContentBlock};

use super::{split_prompt, AIProvider};
use crate::config::Config;

pub struct BedrockProvider {
    client: bedrock::Client,
    model: String,
    region: String,
}

impl BedrockProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let rt = tokio::runtime::Runtime::new().context("Failed to create runtime")?;
        rt.block_on(async { Self::new_async(config).await })
    }

    async fn new_async(config: &Config) -> Result<Self> {
        let region = config
            .api_url
            .as_ref()
            .and_then(|url| {
                url.split("bedrock.")
                    .nth(1)
                    .and_then(|s| s.split('.').next())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()));

        let region_provider = Region::new(region.clone());
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        let client = bedrock::Client::new(&shared_config);

        let model = config.model.as_deref()
            .unwrap_or("anthropic.claude-3-5-sonnet-20241022-v2:0")
            .to_string();

        Ok(Self {
            client,
            model,
            region,
        })
    }

    #[allow(dead_code)]
    pub fn from_account(
        account: &crate::config::accounts::AccountConfig,
        _api_key: &str,
        config: &Config,
    ) -> Result<Self> {
        let rt = tokio::runtime::Runtime::new().context("Failed to create runtime")?;
        rt.block_on(async { Self::from_account_async(account, config).await })
    }

    async fn from_account_async(
        account: &crate::config::accounts::AccountConfig,
        config: &Config,
    ) -> Result<Self> {
        let region = account
            .api_url
            .as_ref()
            .and_then(|url| {
                url.split("bedrock.")
                    .nth(1)
                    .and_then(|s| s.split('.').next())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "us-east-1".to_string());

        let region_provider = Region::new(region.clone());
        let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region_provider)
            .load()
            .await;

        let client = bedrock::Client::new(&shared_config);

        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("anthropic.claude-3-5-sonnet-20241022-v2:0")
            .to_string();

        Ok(Self {
            client,
            model,
            region,
        })
    }
}

#[async_trait]
impl AIProvider for BedrockProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        // Build system message
        let system_block = SystemContentBlock::Text(system_prompt);

        // Build user message
        let user_content = ContentBlock::Text(user_prompt);
        let user_message = bedrock::types::Message::builder()
            .role(bedrock::types::ConversationRole::User)
            .content(user_content)
            .build()
            .context("Failed to build user message")?;

        let inference_config = bedrock::types::InferenceConfiguration::builder()
            .max_tokens(config.tokens_max_output.unwrap_or(500) as i32)
            .temperature(0.7)
            .build();

        let converse_output = self.client
            .converse()
            .model_id(&self.model)
            .messages(user_message)
            .system(system_block)
            .inference_config(inference_config)
            .send()
            .await
            .context("Failed to communicate with Bedrock")?;

        let message = converse_output
            .output()
            .and_then(|o| o.as_message().ok())
            .context("No response from Bedrock")?;

        let content = message
            .content()
            .first()
            .and_then(|c| c.as_text().ok())
            .context("Empty response from Bedrock")?;

        Ok(content.trim().to_string())
    }
}

/// ProviderBuilder for Bedrock
pub struct BedrockProviderBuilder;

impl super::registry::ProviderBuilder for BedrockProviderBuilder {
    fn name(&self) -> &'static str {
        "bedrock"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["aws-bedrock", "amazon-bedrock"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Cloud
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(BedrockProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("anthropic.claude-3-5-sonnet-20241022-v2:0")
    }
}
