//! NVIDIA NIM Provider - Enterprise GPU Inference
//!
//! NVIDIA NIM (NVIDIA Inference Microservices) provides optimized inference
//! for LLMs on NVIDIA GPUs. Supports both self-hosted and cloud deployments.
//!
//! Setup:
//! 1. Get API key from: https://build.nvidia.com
//! 2. Configure rco:
//!    `rco config set RCO_AI_PROVIDER=nvidia RCO_API_KEY=<key> RCO_MODEL=meta/llama-3.1-8b-instruct`
//!
//! Docs: https://docs.nvidia.com/nim/

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct NvidiaProvider {
    client: Client,
    api_url: String,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct NvidiaRequest {
    model: String,
    messages: Vec<NvidiaMessage>,
    max_tokens: i32,
    temperature: f32,
    top_p: f32,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct NvidiaMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct NvidiaResponse {
    choices: Vec<NvidiaChoice>,
}

#[derive(Deserialize)]
struct NvidiaChoice {
    message: NvidiaMessage,
}

impl NvidiaProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();
        let api_key = config
            .api_key
            .as_ref()
            .context("NVIDIA API key not configured.\nRun: rco config set RCO_API_KEY=<your_key>\nGet your API key from: https://build.nvidia.com")?;

        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("https://integrate.api.nvidia.com/v1")
            .to_string();

        let model = config
            .model
            .as_deref()
            .unwrap_or("meta/llama-3.1-8b-instruct")
            .to_string();

        Ok(Self {
            client,
            api_url,
            api_key: api_key.clone(),
            model,
        })
    }

    /// Create provider from account configuration
    #[allow(dead_code)]
    pub fn from_account(
        account: &crate::config::accounts::AccountConfig,
        api_key: &str,
        config: &Config,
    ) -> Result<Self> {
        let client = Client::new();
        let api_url = account
            .api_url
            .as_deref()
            .or(config.api_url.as_deref())
            .unwrap_or("https://integrate.api.nvidia.com/v1")
            .to_string();

        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("meta/llama-3.1-8b-instruct")
            .to_string();

        Ok(Self {
            client,
            api_url,
            api_key: api_key.to_string(),
            model,
        })
    }
}

#[async_trait]
impl AIProvider for NvidiaProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let messages = vec![
            NvidiaMessage {
                role: "system".to_string(),
                content: "You are an expert at writing clear, concise git commit messages.".to_string(),
            },
            NvidiaMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request = NvidiaRequest {
            model: self.model.clone(),
            messages,
            max_tokens: config.tokens_max_output.unwrap_or(500) as i32,
            temperature: 0.7,
            top_p: 0.7,
            stream: false,
        };

        let nvidia_response: NvidiaResponse = retry_async(|| async {
            let url = format!("{}/chat/completions", self.api_url);
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&request)
                .send()
                .await
                .context("Failed to connect to NVIDIA NIM API")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                if error_text.contains("401") || error_text.contains("Unauthorized") {
                    return Err(anyhow::anyhow!("Invalid NVIDIA API key. Please check your API key configuration."));
                }
                return Err(anyhow::anyhow!("NVIDIA NIM API error: {}", error_text));
            }

            let nvidia_response: NvidiaResponse = response
                .json()
                .await
                .context("Failed to parse NVIDIA NIM response")?;

            Ok(nvidia_response)
        })
        .await
        .context("Failed to generate commit message from NVIDIA NIM after retries")?;

        let message = nvidia_response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .context("NVIDIA NIM returned an empty response")?;

        Ok(message)
    }
}

/// ProviderBuilder for NVIDIA NIM
pub struct NvidiaProviderBuilder;

impl super::registry::ProviderBuilder for NvidiaProviderBuilder {
    fn name(&self) -> &'static str {
        "nvidia"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["nvidia-nim", "nim", "nvidia-ai"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Cloud
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(NvidiaProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("meta/llama-3.1-8b-instruct")
    }
}
