use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::Config;

pub struct AzureProvider {
    client: Client,
    api_key: String,
    endpoint: String,
    deployment: String,
}

#[derive(Serialize)]
struct AzureRequest {
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
struct AzureResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl AzureProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("Azure API key not configured. Run: rco config set RCO_API_KEY=<your_key>")?
            .clone();

        let endpoint = config
            .api_url
            .as_ref()
            .context(
                "Azure endpoint not configured. Run: rco config set RCO_API_URL=<your_endpoint>",
            )?
            .clone();

        let deployment = config
            .model
            .as_deref()
            .unwrap_or("gpt-35-turbo")
            .to_string();

        let client = Client::new();

        Ok(Self {
            client,
            api_key,
            endpoint,
            deployment,
        })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(
        account: &crate::config::accounts::AccountConfig,
        api_key: &str,
        config: &Config,
    ) -> Result<Self> {
        let endpoint = account
            .api_url
            .as_ref()
            .context(
                "Azure endpoint required. Set with: rco config set RCO_API_URL=<your_endpoint>",
            )?
            .clone();

        let deployment = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("gpt-35-turbo")
            .to_string();

        let client = Client::new();

        Ok(Self {
            client,
            api_key: api_key.to_string(),
            endpoint,
            deployment,
        })
    }
}

#[async_trait]
impl AIProvider for AzureProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let request = AzureRequest {
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are an expert at writing clear, concise git commit messages."
                        .to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: config.tokens_max_output.unwrap_or(500),
            temperature: 0.7,
        };

        let url = format!(
            "{}/openai/deployments/{}/chat/completions?api-version=2024-02-01",
            self.endpoint, self.deployment
        );

        let response = self
            .client
            .post(&url)
            .header("api-key", &self.api_key)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Azure OpenAI")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Azure OpenAI API error: {}", error_text);
        }

        let azure_response: AzureResponse = response
            .json()
            .await
            .context("Failed to parse Azure OpenAI response")?;

        let message = azure_response
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .context("No response from Azure OpenAI")?;

        Ok(message)
    }
}
