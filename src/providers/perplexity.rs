use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{split_prompt, AIProvider};
use crate::config::Config;

pub struct PerplexityProvider {
    client: Client,
    model: String,
    api_key: String,
}

#[derive(Serialize)]
struct PerplexityRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct PerplexityResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

impl PerplexityProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("Perplexity API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://www.perplexity.ai/settings/api")?;

        let client = Client::new();
        let model = config
            .model
            .as_deref()
            .unwrap_or("llama-3.1-sonar-small-128k-online")
            .to_string();

        Ok(Self {
            client,
            model,
            api_key: api_key.clone(),
        })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(
        _account: &crate::config::accounts::AccountConfig,
        api_key: &str,
        config: &Config,
    ) -> Result<Self> {
        let client = Client::new();
        let model = _account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("llama-3.1-sonar-small-128k-online")
            .to_string();

        Ok(Self {
            client,
            model,
            api_key: api_key.to_string(),
        })
    }
}

#[async_trait]
impl AIProvider for PerplexityProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            Message {
                role: "system".to_string(),
                content: system_prompt,
            },
            Message {
                role: "user".to_string(),
                content: user_prompt,
            },
        ];

        let request = PerplexityRequest {
            model: self.model.clone(),
            messages,
            max_tokens: config.tokens_max_output.unwrap_or(500),
            temperature: 0.7,
            stream: false,
        };

        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("https://api.perplexity.ai/chat/completions");

        let response = match self
            .client
            .post(api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                anyhow::bail!("Failed to connect to Perplexity API: {}. Please check your internet connection.", e);
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            match status.as_u16() {
                401 => anyhow::bail!(
                    "Invalid Perplexity API key. Please check your API key configuration."
                ),
                429 => anyhow::bail!(
                    "Perplexity API rate limit exceeded. Please wait a moment and try again."
                ),
                400 => {
                    if error_text.contains("insufficient_quota") {
                        anyhow::bail!(
                            "Perplexity API quota exceeded. Please check your billing status."
                        );
                    }
                    anyhow::bail!("Bad request to Perplexity API: {}", error_text);
                }
                _ => anyhow::bail!("Perplexity API error ({}): {}", status, error_text),
            }
        }

        let perplexity_response: PerplexityResponse = response
            .json()
            .await
            .context("Failed to parse Perplexity API response")?;

        let message = perplexity_response
            .choices
            .first()
            .map(|choice| &choice.message.content)
            .context("Perplexity returned an empty response. The model may be overloaded - please try again.")?
            .trim()
            .to_string();

        Ok(message)
    }
}

/// ProviderBuilder for Perplexity
pub struct PerplexityProviderBuilder;

impl super::registry::ProviderBuilder for PerplexityProviderBuilder {
    fn name(&self) -> &'static str {
        "perplexity"
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(PerplexityProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("llama-3.1-sonar-small-128k-online")
    }
}
