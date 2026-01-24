use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::sleep;

// Vercel OAuth endpoints
pub const VERCEL_AUTHORIZE_URL: &str = "https://vercel.com/oauth/authorize";
pub const VERCEL_TOKEN_URL: &str = "https://api.vercel.com/oauth/token";
pub const VERCEL_API_URL: &str = "https://api.vercel.com";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct VercelTokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    code_verifier: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct VercelRefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
pub struct VercelTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VercelErrorResponse {
    error: String,
    error_description: Option<String>,
}

/// OAuth client for Vercel AI (AI SDK integration)
pub struct VercelOAuthClient {
    client: Client,
    client_id: String,
    redirect_uri: String,
}

impl Default for VercelOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VercelOAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            client_id: "rusty-commit-cli".to_string(), // Placeholder - would need registration
            redirect_uri: "http://localhost:1456/auth/callback".to_string(),
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
            ("scope", "openid profile email"),
            ("state", state.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
        ];

        let query = serde_urlencoded::to_string(params).context("Failed to encode OAuth params")?;
        let auth_url = format!("{}?{}", VERCEL_AUTHORIZE_URL, query);

        Ok((auth_url, verifier))
    }

    /// Start local server to receive OAuth callback
    pub async fn start_callback_server(&self, verifier: String) -> Result<VercelTokenResponse> {
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
                warp::reply::html(r#"<!DOCTYPE html><html><head><title>Authenticated!</title></head><body style="font-family: system-ui; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #000;"><div style="background: white; padding: 2rem; border-radius: 8px; text-align: center;"><h1 style="color: #000;">Authentication Successful!</h1><p>You can close this window.</p></div></body></html>"#)
            });

        let server = warp::serve(callback).bind(([127, 0, 0, 1], 1456));
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
    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<VercelTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("client_id", self.client_id.as_str()),
            ("code_verifier", verifier),
        ];

        let response = self
            .client
            .post(VERCEL_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;

        if response.status().is_success() {
            response.json::<VercelTokenResponse>().await.context("Failed to parse token response")
        } else {
            let error: VercelErrorResponse = response.json().await?;
            anyhow::bail!("Token exchange failed: {} - {}", error.error, error.error_description.unwrap_or_default())
        }
    }

    /// Refresh an access token
    #[allow(dead_code)]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<VercelTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", self.client_id.as_str()),
        ];

        let response = self
            .client
            .post(VERCEL_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if response.status().is_success() {
            response.json::<VercelTokenResponse>().await.context("Failed to parse refresh token response")
        } else {
            let error: VercelErrorResponse = response.json().await?;
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
