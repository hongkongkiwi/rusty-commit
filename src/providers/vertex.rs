use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::Serialize;
use serde_json::Value;

use super::{split_prompt, AIProvider};
use crate::config::Config;

#[derive(Clone)]
pub struct VertexProvider {
    client: Client,
    model: String,
    project_id: String,
    location: String,
    access_token: String,
}

#[derive(Serialize)]
struct VertexRequest {
    model: String,
    contents: Vec<VertexContent>,
    system_instruction: Option<VertexSystemInstruction>,
    generation_config: VertexGenerationConfig,
}

#[derive(Serialize)]
struct VertexContent {
    role: String,
    parts: Vec<VertexPart>,
}

#[derive(Serialize)]
struct VertexPart {
    text: String,
}

#[derive(Serialize)]
struct VertexSystemInstruction {
    role: String,
    parts: Vec<VertexPart>,
}

#[derive(Serialize)]
struct VertexGenerationConfig {
    max_output_tokens: u32,
    temperature: f32,
}

impl VertexProvider {
    pub fn new(config: &Config) -> Result<Self> {
        let rt = tokio::runtime::Runtime::new().context("Failed to create runtime")?;
        rt.block_on(async { Self::new_async(config).await })
    }

    async fn new_async(config: &Config) -> Result<Self> {
        let client = Client::new();

        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| std::env::var("CLOUDSDK_CORE_PROJECT"))
            .context("Google Cloud project ID not set. Set GOOGLE_CLOUD_PROJECT or run 'gcloud init'")?;

        let location = config
            .api_url
            .as_ref()
            .and_then(|url| {
                url.split(".googleapis.com")
                    .next()
                    .and_then(|s| s.split('-').next())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "us-central1".to_string());

        let model = config.model.as_deref()
            .unwrap_or("gemini-1.5-pro")
            .to_string();

        let access_token = Self::get_gcloud_token().await?;

        Ok(Self {
            client,
            model,
            project_id,
            location,
            access_token,
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
        let client = Client::new();

        let project_id = std::env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| std::env::var("CLOUDSDK_CORE_PROJECT"))
            .context("Google Cloud project ID not set")?;

        let location = account
            .api_url
            .as_ref()
            .and_then(|url| {
                url.split(".googleapis.com")
                    .next()
                    .and_then(|s| s.split('-').next())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| "us-central1".to_string());

        let model = account
            .model
            .as_deref()
            .or(config.model.as_deref())
            .unwrap_or("gemini-1.5-pro")
            .to_string();

        let access_token = Self::get_gcloud_token().await?;

        Ok(Self {
            client,
            model,
            project_id,
            location,
            access_token,
        })
    }

    async fn get_gcloud_token() -> Result<String> {
        let output = tokio::process::Command::new("gcloud")
            .args(&["auth", "print-access-token"])
            .output()
            .await
            .context("Failed to execute gcloud command")?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to get Google Cloud access token. Run 'gcloud auth login' first."
            );
        }

        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();

        Ok(token)
    }
}

#[async_trait]
impl AIProvider for VertexProvider {
    async fn generate_commit_message(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
        config: &Config,
    ) -> Result<String> {
        let (system_prompt, user_prompt) = split_prompt(diff, context, config, full_gitmoji);

        let request = VertexRequest {
            model: format!(
                "projects/{}/locations/{}/publishers/google/models/{}",
                self.project_id, self.location, self.model
            ),
            contents: vec![VertexContent {
                role: "user".to_string(),
                parts: vec![VertexPart {
                    text: user_prompt,
                }],
            }],
            system_instruction: Some(VertexSystemInstruction {
                role: "system".to_string(),
                parts: vec![VertexPart {
                    text: system_prompt,
                }],
            }),
            generation_config: VertexGenerationConfig {
                max_output_tokens: config.tokens_max_output.unwrap_or(500),
                temperature: 0.7,
            },
        };

        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/{}:streamGenerateContent",
            self.location, self.project_id, self.location, self.model
        );

        let response = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.access_token))
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to Vertex AI")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Vertex AI API error: {}", error_text);
        }

        let json: Value = response
            .json()
            .await
            .context("Failed to parse Vertex AI response")?;

        // Extract text from response
        let message = json
            .get("candidates")
            .and_then(|candidates| candidates.as_array())
            .and_then(|candidates| candidates.first())
            .and_then(|cand| cand.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .and_then(|parts| parts.first())
            .and_then(|part| part.get("text"))
            .and_then(|text| text.as_str())
            .map(|s| s.to_string())
            .context("No response from Vertex AI")?;

        Ok(message.trim().to_string())
    }
}

/// ProviderBuilder for Vertex AI
pub struct VertexProviderBuilder;

impl super::registry::ProviderBuilder for VertexProviderBuilder {
    fn name(&self) -> &'static str {
        "vertex"
    }

    fn aliases(&self) -> Vec<&'static str> {
        vec!["vertex-ai", "google-vertex", "gcp-vertex"]
    }

    fn category(&self) -> super::registry::ProviderCategory {
        super::registry::ProviderCategory::Cloud
    }

    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>> {
        Ok(Box::new(VertexProvider::new(config)?))
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn default_model(&self) -> Option<&'static str> {
        Some("gemini-1.5-pro")
    }
}
