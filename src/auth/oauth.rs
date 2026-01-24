use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

use super::token_storage::TokenStorage;

// Claude OAuth endpoints (similar to Claude Code)
pub const AUTHORIZE_URL: &str = "https://claude.ai/oauth/authorize";
pub const TOKEN_URL: &str = "https://claude.ai/oauth/token";
pub const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e"; // Public client ID for CLI apps
pub const REDIRECT_URI: &str = "http://localhost:8989/callback";

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct DeviceCodeRequest {
    client_id: String,
    scope: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct TokenRequest {
    grant_type: String,
    device_code: String,
    client_id: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct RefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ErrorResponse {
    error: String,
    error_description: Option<String>,
}

/// OAuth client for Claude authentication
pub struct OAuthClient {
    client: Client,
    client_id: String,
    redirect_uri: String,
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            client_id: CLIENT_ID.to_string(),
            redirect_uri: REDIRECT_URI.to_string(),
        }
    }

    /// Generate PKCE challenge and verifier
    fn generate_pkce() -> Result<(String, String)> {
        // Generate random verifier
        let mut bytes = [0u8; 32];
        generate_random_bytes(&mut bytes)?;
        let verifier = URL_SAFE_NO_PAD.encode(bytes);

        // Generate challenge from verifier
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

        Ok((verifier, challenge))
    }

    /// Build authorization URL with PKCE
    pub fn get_authorization_url(&self) -> Result<(String, String)> {
        let (verifier, challenge) = Self::generate_pkce()?;

        let state = URL_SAFE_NO_PAD.encode(uuid::Uuid::new_v4().as_bytes());

        let params = [
            ("client_id", &self.client_id),
            ("redirect_uri", &self.redirect_uri),
            ("response_type", &"code".to_string()),
            ("scope", &"openid profile email".to_string()),
            ("state", &state),
            ("code_challenge", &challenge),
            ("code_challenge_method", &"S256".to_string()),
        ];

        let query = serde_urlencoded::to_string(params).context("Failed to encode OAuth params")?;
        let auth_url = format!("{AUTHORIZE_URL}?{query}");

        Ok((auth_url, verifier))
    }

    /// Start local server to receive OAuth callback
    pub async fn start_callback_server(&self, verifier: String) -> Result<TokenResponse> {
        use warp::Filter;

        let code = Arc::new(Mutex::new(None));
        let code_clone = code.clone();

        // Create callback route
        let callback = warp::path("callback")
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .map(move |params: std::collections::HashMap<String, String>| {
                if let Some(auth_code) = params.get("code") {
                    let mut code_lock = code_clone.blocking_lock();
                    *code_lock = Some(auth_code.clone());
                }

                warp::reply::html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>Authentication Successful</title>
                        <style>
                            body {
                                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                                display: flex;
                                justify-content: center;
                                align-items: center;
                                height: 100vh;
                                margin: 0;
                                background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                            }
                            .container {
                                background: white;
                                padding: 3rem;
                                border-radius: 12px;
                                box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                                text-align: center;
                                max-width: 400px;
                            }
                            h1 { color: #2d3748; margin-bottom: 1rem; }
                            p { color: #718096; line-height: 1.6; }
                            .check {
                                width: 60px;
                                height: 60px;
                                margin: 0 auto 1.5rem;
                                background: #48bb78;
                                border-radius: 50%;
                                display: flex;
                                align-items: center;
                                justify-content: center;
                            }
                            .check::after {
                                content: '✓';
                                color: white;
                                font-size: 30px;
                                font-weight: bold;
                            }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <div class="check"></div>
                            <h1>Authentication Successful!</h1>
                            <p>You can now close this window and return to your terminal.</p>
                        </div>
                    </body>
                    </html>
                    "#
                )
            });

        // Start server in background
        let server = warp::serve(callback).bind(([127, 0, 0, 1], 8989));
        let server_handle = tokio::spawn(server);

        // Wait for code (with timeout)
        let start = SystemTime::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        loop {
            if let Some(auth_code) = &*code.lock().await {
                // Exchange code for token
                let token = self.exchange_code_for_token(auth_code, &verifier).await?;
                server_handle.abort();
                return Ok(token);
            }

            if SystemTime::now().duration_since(start)? > timeout {
                server_handle.abort();
                anyhow::bail!("Authentication timeout - no response received");
            }

            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Exchange authorization code for access token
    async fn exchange_code_for_token(&self, code: &str, verifier: &str) -> Result<TokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("code_verifier", verifier),
        ];

        let response = self
            .client
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;

        if response.status().is_success() {
            response
                .json::<TokenResponse>()
                .await
                .context("Failed to parse token response")
        } else {
            let error: ErrorResponse = response.json().await?;
            anyhow::bail!(
                "Token exchange failed: {} - {}",
                error.error,
                error.error_description.unwrap_or_default()
            )
        }
    }

    /// Refresh an access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let request = RefreshTokenRequest {
            grant_type: "refresh_token".to_string(),
            refresh_token: refresh_token.to_string(),
            client_id: self.client_id.clone(),
        };

        let response = self
            .client
            .post(TOKEN_URL)
            .json(&request)
            .send()
            .await
            .context("Failed to refresh token")?;

        if response.status().is_success() {
            response
                .json::<TokenResponse>()
                .await
                .context("Failed to parse refresh token response")
        } else {
            let error: ErrorResponse = response.json().await?;
            anyhow::bail!(
                "Token refresh failed: {} - {}",
                error.error,
                error.error_description.unwrap_or_default()
            )
        }
    }

    /// Check if a token is expired
    #[allow(dead_code)]
    pub fn is_token_expired(expires_at: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_secs();
        now >= expires_at
    }
}

// Generate random bytes for PKCE
fn generate_random_bytes(dest: &mut [u8]) -> Result<()> {
    use rand::RngCore;
    let mut rng = rand::rng();
    rng.fill_bytes(dest);
    Ok(())
}
