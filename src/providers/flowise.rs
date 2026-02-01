//! Flowise Provider - Self-hosted LLM workflow platform
//!
//! Flowise is a low-code/no-code drag & drop workflow builder for LLMs.
//! This provider connects to a self-hosted Flowise instance.
//!
//! Setup:
//! 1. Install Flowise: `npm install -g flowise`
//! 2. Start Flowise: `npx flowise start`
//! 3. Configure rco: `rco config set RCO_AI_PROVIDER=flowise RCO_API_URL=http://localhost:3000`
//!
//! Docs: https://docs.flowiseai.com/

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct FlowiseProvider {
    client: Client,
    api_url: String,
    api_key: Option<String>,
}

#[derive(Serialize)]
struct FlowiseRequest {
    question: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    history: Option<Vec<FlowiseMessage>>,
}

#[derive(Serialize, Deserialize, Clone)]
struct FlowiseMessage {
    message: String,
    #[serde(rename = "type")]
    message_type: String,
}

#[derive(Deserialize)]
struct FlowiseResponse {
    text: String,
    #[serde(rename = "sessionId")]
    #[allow(dead_code)]
    session_id: Option<String>,
}

impl FlowiseProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:3000")
            .to_string();
        let api_key = config.api_key.clone();

        Ok(Self {
            client,
            api_url,
            api_key,
        })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(
        account: &crate::config::accounts::AccountConfig,
        _api_key: &str,
        config: &Config,
    ) -> Result<Self> {
        let client = Client::new();
        let api_url = account
            .api_url
            .as_deref()
            .or(config.api_url.as_deref())
            .unwrap_or("http://localhost:3000")
            .to_string();

        Ok(Self {
            client,
            api_url,
            api_key: None,
        })
    }
}

#[async_trait]
impl AIProvider for FlowiseProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let request = FlowiseRequest {
            question: prompt,
            history: None,
        };

        let flowise_response: FlowiseResponse = retry_async(|| async {
            let url = format!("{}/api/v1/prediction/flowise", self.api_url);
            let mut req = self.client.post(&url).json(&request);
            
            // Add API key if available
            if let Some(ref key) = self.api_key {
                req = req.header("Authorization", format!("Bearer {}", key));
            }
            
            let response = req
                .send()
                .await
                .context("Failed to connect to Flowise server. Is Flowise running?")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                if error_text.contains("Unauthorized") || error_text.contains("401") {
                    return Err(anyhow::anyhow!("Invalid Flowise API key. Please check your configuration."));
                }
                return Err(anyhow::anyhow!("Flowise API error: {}", error_text));
            }

            let flowise_response: FlowiseResponse = response
                .json()
                .await
                .context("Failed to parse Flowise response")?;

            Ok(flowise_response)
        })
        .await
        .context("Failed to generate commit message from Flowise after retries")?;

        let message = flowise_response
            .text
            .trim()
            .to_string();

        if message.is_empty() {
            anyhow::bail!("Flowise returned an empty response");
        }

        Ok(message)
    }
}

/// ProviderBuilder for Flowise
pub struct FlowiseProviderBuilder;

impl super::registry::ProviderBuilder for FlowiseProviderBuilder {
    fn name(&self) -> &'static str {
        "flowise"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["flowise-ai"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Local
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(FlowiseProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        false // Flowise is self-hosted, API key is optional
    }

    fn default_model(&self) -> Option<&'static str> {
        None // Flowise uses workflows, not direct model selection
    }
}
