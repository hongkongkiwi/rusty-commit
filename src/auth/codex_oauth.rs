use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::Mutex;
use tokio::time::sleep;

// OpenAI Codex OAuth endpoints (ChatGPT Pro/Plus)
pub const CODEX_AUTHORIZE_URL: &str = "https://auth.openai.com/oauth/authorize";
pub const CODEX_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
pub const CODEX_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const CODEX_REDIRECT_URI: &str = "http://localhost:1455/auth/callback";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CodeXTokenResponse {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<u64>,
    pub token_type: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct CodeXTokenRequest {
    grant_type: String,
    code: String,
    redirect_uri: String,
    client_id: String,
    code_verifier: String,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct CodeXRefreshTokenRequest {
    grant_type: String,
    refresh_token: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
struct CodeXErrorResponse {
    error: String,
    error_description: Option<String>,
}

/// OAuth client for OpenAI Codex authentication (ChatGPT Pro/Plus)
pub struct CodexOAuthClient {
    client: Client,
    client_id: String,
    redirect_uri: String,
}

impl Default for CodexOAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexOAuthClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            client_id: CODEX_CLIENT_ID.to_string(),
            redirect_uri: CODEX_REDIRECT_URI.to_string(),
        }
    }

    /// Generate PKCE challenge and verifier
    fn generate_pkce() -> Result<(String, String)> {
        // Generate random verifier (43 characters as per RFC 7636)
        let mut bytes = [0u8; 32];
        generate_random_bytes(&mut bytes)?;
        let verifier = URL_SAFE_NO_PAD.encode(bytes);

        // Generate challenge from verifier
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
            ("scope", "openid profile email offline_access"),
            ("state", state.as_str()),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("id_token_add_organizations", "true"),
            ("codex_cli_simplified_flow", "true"),
            ("originator", "rusty-commit"),
        ];

        let query = serde_urlencoded::to_string(params).context("Failed to encode OAuth params")?;
        let auth_url = format!("{}?{}", CODEX_AUTHORIZE_URL, query);

        Ok((auth_url, verifier))
    }

    /// Start local server to receive OAuth callback
    pub async fn start_callback_server(&self, verifier: String) -> Result<CodeXTokenResponse> {
        use warp::Filter;

        let code = Arc::new(Mutex::new(None));
        let code_clone = code.clone();

        // Create callback route
        let callback = warp::path("auth")
            .and(warp::path("callback"))
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .map(move |params: std::collections::HashMap<String, String>| {
                if let Some(auth_code) = params.get("code") {
                    let mut code_lock = code_clone.blocking_lock();
                    *code_lock = Some(auth_code.clone());
                }

                warp::reply::html(r#"
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
                            background: linear-gradient(135deg, #10a37f 0%, #1a7f64 100%);
                        }
                        .container {
                            background: white;
                            padding: 3rem;
                            border-radius: 12px;
                            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                            text-align: center;
                            max-width: 400px;
                        }
                        h1 { color: #1a7f64; margin-bottom: 1rem; }
                        p { color: #666; line-height: 1.6; }
                        .check {
                            width: 60px;
                            height: 60px;
                            margin: 0 auto 1.5rem;
                            background: #10a37f;
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
                "#)
            });

        // Start server in background
        let server = warp::serve(callback).bind(([127, 0, 0, 1], 1455));
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
    async fn exchange_code_for_token(
        &self,
        code: &str,
        verifier: &str,
    ) -> Result<CodeXTokenResponse> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.redirect_uri),
            ("client_id", &self.client_id),
            ("code_verifier", verifier),
        ];

        let response = self
            .client
            .post(CODEX_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to exchange code for token")?;

        if response.status().is_success() {
            response
                .json::<CodeXTokenResponse>()
                .await
                .context("Failed to parse token response")
        } else {
            let error: CodeXErrorResponse = response.json().await?;
            anyhow::bail!(
                "Token exchange failed: {} - {}",
                error.error,
                error.error_description.unwrap_or_default()
            )
        }
    }

    /// Refresh an access token
    #[allow(dead_code)]
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<CodeXTokenResponse> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.client_id),
        ];

        let response = self
            .client
            .post(CODEX_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to refresh token")?;

        if response.status().is_success() {
            response
                .json::<CodeXTokenResponse>()
                .await
                .context("Failed to parse refresh token response")
        } else {
            let error: CodeXErrorResponse = response.json().await?;
            anyhow::bail!(
                "Token refresh failed: {} - {}",
                error.error,
                error.error_description.unwrap_or_default()
            )
        }
    }
}

// Generate random bytes
fn generate_random_bytes(dest: &mut [u8]) -> Result<()> {
    use rand::RngCore;
    let mut rng = rand::rng();
    rng.fill_bytes(dest);
    Ok(())
}
