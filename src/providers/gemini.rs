use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::prompt::split_prompt;
use super::AIProvider;
use crate::config::Config;

pub struct GeminiProvider {
    client: Client,
    api_key: String,
    model: String,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    system_instruction: Option<SystemInstruction>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct SystemInstruction {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

impl GeminiProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config
            .api_key
            .as_ref()
            .context("Gemini API key not configured. Run: rco config set RCO_API_KEY=<your_key>")?
            .clone();

        let client = Client::new();
        let model = config.model.as_deref().unwrap_or("gemini-pro").to_string();

        Ok(Self {
            client,
            api_key,
            model,
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
            .unwrap_or("gemini-pro")
            .to_string();

        Ok(Self {
            client,
            api_key: api_key.to_string(),
            model,
        })
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        let request = GeminiRequest {
            contents: vec![Content {
                role: "user".to_string(),
                parts: vec![Part { text: user_prompt }],
            }],
            system_instruction: Some(SystemInstruction {
                role: "system".to_string(),
                parts: vec![Part {
                    text: system_prompt,
                }],
            }),
            generation_config: GenerationConfig {
                temperature: 0.7,
                max_output_tokens: config.tokens_max_output.unwrap_or(500),
            },
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            self.model
        );

        let response = self
            .client
            .post(&url)
            .header("X-Goog-Api-Key", &self.api_key)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Gemini")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Gemini API error: {}", error_text);
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .context("Failed to parse Gemini response")?;

        let message = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.trim().to_string())
            .context("No response from Gemini")?;

        Ok(message)
    }
}

/// ProviderBuilder for Gemini
pub struct GeminiProviderBuilder;

impl super::registry::ProviderBuilder for GeminiProviderBuilder {
    fn name(&self) -> &'static str {
        "gemini"
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(GeminiProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        true
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("gemini-1.5-pro")
    }
}
