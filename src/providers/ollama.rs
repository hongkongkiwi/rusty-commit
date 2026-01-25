use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{build_prompt, AIProvider};
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct OllamaProvider {
    client: Client,
    api_url: String,
    model: String,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

impl OllamaProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:11434")
            .to_string();
        let model = config.model.as_deref().unwrap_or("mistral").to_string();

        Ok(Self {
            client,
            api_url,
            model,
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
            .unwrap_or("http://localhost:11434")
            .to_string();
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("mistral")
            .to_string();

        Ok(Self {
            client,
            api_url,
            model,
        })
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: 0.7,
                num_predict: config.tokens_max_output.unwrap_or(500) as i32,
            },
        };

        let ollama_response: OllamaResponse = retry_async(|| async {
            let url = format!("{}/api/generate", self.api_url);
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .context("Failed to connect to Ollama")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
            }

            let ollama_response: OllamaResponse = response
                .json()
                .await
                .context("Failed to parse Ollama response")?;

            Ok(ollama_response)
        })
        .await
        .context("Failed to generate commit message from Ollama after retries")?;

        Ok(ollama_response.response.trim().to_string())
    }
}
