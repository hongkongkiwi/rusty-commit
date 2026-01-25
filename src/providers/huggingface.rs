use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::accounts::AccountConfig;
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct HuggingFaceProvider {
    client: Client,
    api_key: String,
    model: String,
    api_url: String,
}

#[derive(Serialize)]
struct HFRequest {
    model: String,
    inputs: String,
    parameters: HFParameters,
    options: HFOptions,
}

#[derive(Serialize)]
struct HFParameters {
    temperature: Option<f32>,
    max_new_tokens: Option<u32>,
    return_full_text: bool,
}

#[derive(Serialize)]
struct HFOptions {
    use_cache: bool,
}

#[derive(Deserialize)]
struct HFResponse {
    generated_text: Option<String>,
    error: Option<String>,
}

impl HuggingFaceProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("HuggingFace API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your token from: https://huggingface.co/settings/tokens")?
            .clone();

        let client = Client::new();
        let model = config
            .model
            .as_deref()
            .unwrap_or("mistralai/Mistral-7B-Instruct-v0.2")
            .to_string();

        // Determine if this is an inference API call or dedicated endpoint
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("https://api-inference.huggingface.co");

        Ok(Self {
            client,
            api_key,
            model,
            api_url: api_url.to_string(),
        })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(account: &AccountConfig, api_key: &str, config: &Config) -> Result<Self> {
        let client = Client::new();
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("mistralai/Mistral-7B-Instruct-v0.2")
            .to_string();

        let api_url = account
            .api_url
            .as_deref()
            .or(config.api_url.as_deref())
            .unwrap_or("https://api-inference.huggingface.co")
            .to_string();

        Ok(Self {
            client,
            api_key: api_key.to_string(),
            model,
            api_url,
        })
    }
}

#[async_trait]
impl AIProvider for HuggingFaceProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        // HuggingFace Inference API uses a single prompt (no system message support)
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let request = HFRequest {
            model: self.model.clone(),
            inputs: prompt,
            parameters: HFParameters {
                temperature: Some(0.7),
                max_new_tokens: Some(config.tokens_max_output.unwrap_or(500)),
                return_full_text: false,
            },
            options: HFOptions { use_cache: true },
        };

        // Use Inference API endpoint
        let url = format!("{}/models/{}", self.api_url, self.model);

        let hf_response: HFResponse = retry_async(|| async {
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
                .context("Failed to connect to HuggingFace")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("HuggingFace API error: {}", error_text));
            }

            let hf_response: HFResponse = response
                .json()
                .await
                .context("Failed to parse HuggingFace response")?;

            Ok(hf_response)
        })
        .await
        .context("Failed to generate commit message from HuggingFace after retries")?;

        // Handle error response
        if let Some(error) = hf_response.error {
            anyhow::bail!("HuggingFace inference error: {}", error);
        }

        let message = hf_response
            .generated_text
            .context("HuggingFace returned an empty response")?
            .trim()
            .to_string();

        // Clean up the response - remove the prompt if it's included
        // HF models often return the full prompt + completion
        Ok(message)
    }
}

/// ProviderBuilder for HuggingFace
pub struct HuggingFaceProviderBuilder;

impl super::registry::ProviderBuilder for HuggingFaceProviderBuilder {
    fn name(&self) -> &'static str {
        "huggingface"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["hf"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Standard
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(HuggingFaceProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("mistralai/Mistral-7B-Instruct-v0.2")
    }
}
