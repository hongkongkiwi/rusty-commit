pub mod oauth;
pub mod token_storage;

use crate::config::Config;
use anyhow::Result;

/// Check if the user is authenticated with Claude
#[allow(dead_code)]
pub async fn is_authenticated(config: &Config) -> bool {
    // Check for API key first (backward compatibility)
    if config.api_key.is_some() {
        return true;
    }

    // Check for OAuth tokens
    token_storage::has_valid_token()
}

/// Get the authentication header value for API requests
#[allow(dead_code)]
pub async fn get_auth_header(config: &Config) -> Result<String> {
    // Prefer OAuth token if available
    if let Some(token) = token_storage::get_access_token()? {
        return Ok(format!("Bearer {}", token));
    }

    // Fall back to API key
    if let Some(api_key) = &config.api_key {
        return Ok(api_key.clone());
    }

    anyhow::bail!("Not authenticated. Please run 'rco auth login' or set RCO_API_KEY")
}
