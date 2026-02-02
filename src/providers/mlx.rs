//! MLX Provider - Apple's ML framework for Apple Silicon
//!
//! MLX is Apple's machine learning framework optimized for Apple Silicon.
//! This provider connects to an MLX HTTP server running locally.
//!
//! Setup:
//! 1. Install mlx-lm: `pip install mlx-lm`
//! 2. Start server: `python -m mlx_lm.server --model mlx-community/Llama-3.2-3B-Instruct-4bit`
//! 3. Configure rco: `rco config set RCO_AI_PROVIDER=mlx RCO_API_URL=http://localhost:8080`

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::prompt::build_prompt;
use super::AIProvider;
use crate::config::Config;
use crate::utils::retry::retry_async;

pub struct MlxProvider {
    client: Client,
    api_url: String,
    model: String,
}

#[derive(Serialize)]
struct MlxRequest {
    model: String,
    messages: Vec<MlxMessage>,
    max_tokens: i32,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct MlxMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct MlxResponse {
    choices: Vec<MlxChoice>,
}

#[derive(Deserialize)]
struct MlxChoice {
    message: MlxMessage,
}

impl MlxProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::new();
        let api_url = config
            .api_url
            .as_deref()
            .unwrap_or("http://localhost:8080")
            .to_string();
        let model = config.model.as_deref().unwrap_or("default").to_string();

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
            .unwrap_or("http://localhost:8080")
            .to_string();
        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("default")
            .to_string();

        Ok(Self {
            client,
            api_url,
            model,
        })
    }
}

#[async_trait]
impl AIProvider for MlxProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let prompt = build_prompt(diff, context, config, full_gitmoji);

        // MLX uses OpenAI-compatible chat format
        let messages = vec![
            MlxMessage {
                role: "system".to_string(),
                content: "You are an expert at writing clear, concise git commit messages."
                    .to_string(),
            },
            MlxMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ];

        let request = MlxRequest {
            model: self.model.clone(),
            messages,
            max_tokens: config.tokens_max_output.unwrap_or(500) as i32,
            temperature: 0.7,
            stream: false,
        };

        let mlx_response: MlxResponse = retry_async(|| async {
            let url = format!("{}/v1/chat/completions", self.api_url);
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .context("Failed to connect to MLX server")?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow::anyhow!("MLX API error: {}", error_text));
            }

            let mlx_response: MlxResponse = response
                .json()
                .await
                .context("Failed to parse MLX response")?;

            Ok(mlx_response)
        })
        .await
        .context("Failed to generate commit message from MLX after retries")?;

        let message = mlx_response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .context("MLX returned an empty response")?;

        Ok(message)
    }
}

/// ProviderBuilder for MLX
pub struct MlxProviderBuilder;

impl super::registry::ProviderBuilder for MlxProviderBuilder {
    fn name(&self) -> &'static str {
        "mlx"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["mlx-lm", "apple-mlx"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Local
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(MlxProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("mlx-community/Llama-3.2-3B-Instruct-4bit")
    }
}
