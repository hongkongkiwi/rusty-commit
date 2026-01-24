use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::sleep;

// GitLab OAuth endpoints
pub const GITLAB_AUTHORIZE_URL: &str = "https://gitlab.com/oauth/authorize";
pub const GITLAB_TOKEN_URL: &str = "https://gitlab.com/oauth/token";
pub const GITLAB_API_URL: &str = "https://gitlab.com/api/v4";

// GitLab AI Gateway for Claude models
pub const GITLAB_AI_GATEWAY_URL: &str = "https://gitlab.ai/api/v1";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct GitLabTokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    code_verifier: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct GitLabRefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct GitLabTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitLabErrorResponse {
    error: String,
    error_description: Option<String>,
}

/// OAuth client for GitLab AI (Claude models on GitLab)
pub struct GitLabOAuthClient {
    client: Client,
    client_id: String,
    redirect_uri: String,
}

impl Default for GitLabOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitLabOAuthClient {
    pub fn new() -> Self {
        // GitLab CLI OAuth app for rusty-commit
        Self {
            client: Client::new(),
            client_id: "cde3d0a736a6f9d9e9b9e9b9e9b9e9b9e9b9e9b9e".to_string(), // Placeholder
            redirect_uri: "http://localhost:8989/auth/callback".to_string(),
        }
    }

    /// Generate PKCE challenge and verifier
    fn generate_pkce() -> Result<(String, String)> {
        let mut bytes = [0u8; 32];
        generate_random_bytes(&mut bytes)?;
        let verifier = URL_SAFE_NO_PAD.encode(bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        Ok((verifier, challenge))
    }

    /// Generate random state for CSRF protection
    fn generate_state() -> String {
        let mut bytes = [0u8; 32];
        generate_random_bytes(&mut bytes).unwrap_or_default();
        URL_SAFE_NO_PAD.encode(bytes)
    }

    /// Build authorization URL with PKCE
    pub fn get_authorization_url(&self) -> Result<(String, String)> {
        let (verifier, challenge) = Self::generate_pkce()?;
        let state = Self::generate_state();

        let params = [
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "read_user api openid"),
            ("state", state.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
        ];

        let query = serde_urlencoded::to_string(params).context("Failed to encode OAuth params")?;
        let auth_url = format!("{}?{}", GITLAB_AUTHORIZE_URL, query);

        Ok((auth_url, verifier))
    }

    /// Start local server to receive OAuth callback
    pub async fn start_callback_server(&self, verifier: String) -> Result<GitLabTokenResponse> {
        use warp::Filter;

        let code = Arc::new(Mutex::new(None));
        let code_clone = code.clone();

        let callback = warp::path("auth")
            .and(warp::path("callback"))
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .map(move |params: std::collections::HashMap<String, String>| {
                if let Some(auth_code) = params.get("code") {
                    let mut code_lock = code_clone.blocking_lock();
                    *code_lock = Some(auth_code.clone());
                }
                warp::reply::html(r#"<!DOCTYPE html><html><head><title>Authenticated!</title></head><body style="font-family: system-ui; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #fc6d26;"><div style="background: white; padding: 2rem; border-radius: 8px; text-align: center;"><h1 style="color: #fc6d26;">Authentication Successful!</h1><p>You can close this window.</p></div></body></html>"#)
            });

        let server = warp::serve(callback).bind(([127, 0, 0, 1], 8989));
        let server_handle = tokio::spawn(server);

        let start = std::time::SystemTime::now();
        let timeout = Duration::from_secs(300);

        loop {
            if let Some(auth_code) = &*code.lock().await {
                let token = self.exchange_code_for_token(auth_code, &verifier).await?;
                server_handle.abort();
                return Ok(token);
            }

            if SystemTime::now().duration_since(start)? > timeout {
                server_handle.abort();
                anyhow::bail!("Authentication timeout");
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Exchange authorization code for access token
    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<GitLabTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", self.client_id.as_str()),
            ("code_verifier", verifier),
        ];

        let response = self
            .client
            .post(GITLAB_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;

        if response.status().is_success() {
            response.json::<GitLabTokenResponse>().await.context("Failed to parse token response")
        } else {
            let error: GitLabErrorResponse = response.json().await?;
            anyhow::bail!("Token exchange failed: {} - {}", error.error, error.error_description.unwrap_or_default())
        }
    }

    /// Refresh an access token
    #[allow(dead_code)]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GitLabTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", self.client_id.as_str()),
        ];

        let response = self
            .client
            .post(GITLAB_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if response.status().is_success() {
            response.json::<GitLabTokenResponse>().await.context("Failed to parse refresh token response")
        } else {
            let error: GitLabErrorResponse = response.json().await?;
            anyhow::bail!("Token refresh failed: {} - {}", error.error, error.error_description.unwrap_or_default())
        }
    }
}

/// Generate random bytes
fn generate_random_bytes(dest: &mut [u8]) -> Result<()> {
    use rand::RngCore;
    let mut rng = rand::rng();
    rng.fill_bytes(dest);
    Ok(())
}
