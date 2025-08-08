use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct AnthropicProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

impl AnthropicProvider {
    pub fn new(config: &Config) -> Result<Self> {
        // Try OAuth token first, then fall back to API key
        let api_key = if let Some(token) = crate::auth::token_storage::get_access_token()? {
            token
        } else {
            config
                .api_key
                .as_ref()
                .context(
                    "Not authenticated with Claude.\nRun: oco auth login (for OAuth)\nOr: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://console.anthropic.com/settings/keys",
                )?
                .clone()
        };

        let client = Client::new();
        let model = config
            .model
            .as_deref()
            .unwrap_or("claude-3-sonnet-20240229")
            .to_string();

        Ok(Self {
            client,
            api_key,
            model,
        })
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            max_tokens: config.tokens_max_output.unwrap_or(500),
            temperature: 0.7,
        };

        let anthropic_response: AnthropicResponse = retry_async(|| async {
            // Build request with appropriate auth header
            let mut req = self
                .client
                .post("https://api.anthropic.com/v1/messages");
            
            // Check if this is an OAuth token (starts with "ey") or API key (starts with "sk-")
            if self.api_key.starts_with("ey") {
                // OAuth token - use Authorization header
                req = req.header(header::AUTHORIZATION, format!("Bearer {}", &self.api_key));
            } else {
                // API key - use x-api-key header
                req = req.header("x-api-key", &self.api_key);
            }
            
            let response = req
                .header("anthropic-version", "2023-06-01")
                .header(header::CONTENT_TYPE, "application/json")
                .json(&request)
                .send()
                .await
                .context("Failed to connect to Anthropic")?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await?;
                
                if status.as_u16() == 401 {
                    return Err(anyhow::anyhow!("Invalid Anthropic API key. Please check your API key configuration."));
                } else if status.as_u16() == 403 {
                    return Err(anyhow::anyhow!("Access forbidden. Please check your Anthropic API permissions."));
                } else {
                    return Err(anyhow::anyhow!("Anthropic API error ({}): {}", status, error_text));
                }
            }

            let anthropic_response: AnthropicResponse = response
                .json()
                .await
                .context("Failed to parse Anthropic response")?;
                
            Ok(anthropic_response)
        }).await.context("Failed to generate commit message from Anthropic after retries. Please check your internet connection and API configuration.")?;

        let message = anthropic_response
            .content
            .first()
            .map(|c| c.text.trim().to_string())
            .context("Anthropic returned an empty response. The model may be overloaded - please try again.")?;

        Ok(message)
    }
}
