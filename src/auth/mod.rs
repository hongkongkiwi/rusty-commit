pub mod codex_oauth;
pub mod gitlab_oauth;
pub mod oauth;
pub mod token_storage;
pub mod vercel_oauth;

use crate::config::accounts::{AccountConfig, AuthMethod};
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

// ============================================
// Multi-account authentication support
// ============================================

/// Check if an account is authenticated
#[allow(dead_code)]
pub fn is_account_authenticated(account: &AccountConfig) -> bool {
    match &account.auth {
        AuthMethod::ApiKey { key_id } => {
            // Check if we have the API key stored
            token_storage::get_api_key_for_account(key_id).ok().flatten().is_some()
        }
        AuthMethod::OAuth { provider: _, account_id } => {
            // Check if we have valid OAuth tokens
            if let Ok(Some(tokens)) = token_storage::get_tokens_for_account(account_id) {
                !tokens.is_expired()
            } else {
                false
            }
        }
        AuthMethod::EnvVar { name } => {
            // Check if environment variable is set
            std::env::var(name).is_ok()
        }
        AuthMethod::Bearer { token_id } => {
            // Check if bearer token exists
            token_storage::get_bearer_token_for_account(token_id)
                .ok()
                .flatten()
                .is_some()
        }
    }
}

/// Get authentication header for an account
#[allow(dead_code)]
pub fn get_account_auth_header(account: &AccountConfig) -> Result<String> {
    match &account.auth {
        AuthMethod::ApiKey { key_id } => {
            if let Some(api_key) = token_storage::get_api_key_for_account(key_id)? {
                Ok(api_key)
            } else {
                // Try to get from environment
                if let Ok(key_from_env) = std::env::var(format!("RCO_API_KEY_{}", key_id.to_uppercase())) {
                    return Ok(key_from_env);
                }
                anyhow::bail!(
                    "API key not found for account '{}'. Run: rco config add-provider --alias {}",
                    account.alias,
                    account.alias
                )
            }
        }
        AuthMethod::OAuth { provider: _, account_id } => {
            if let Ok(Some(tokens)) = token_storage::get_tokens_for_account(account_id) {
                Ok(format!("Bearer {}", tokens.access_token))
            } else {
                anyhow::bail!(
                    "OAuth tokens not found for account '{}'. Please re-authenticate.",
                    account.alias
                )
            }
        }
        AuthMethod::EnvVar { name } => {
            if let Ok(value) = std::env::var(name) {
                Ok(value)
            } else {
                anyhow::bail!(
                    "Environment variable '{}' not set for account '{}'",
                    name,
                    account.alias
                )
            }
        }
        AuthMethod::Bearer { token_id } => {
            if let Some(token) = token_storage::get_bearer_token_for_account(token_id)? {
                Ok(format!("Bearer {}", token))
            } else {
                anyhow::bail!(
                    "Bearer token not found for account '{}'",
                    account.alias
                )
            }
        }
    }
}

/// Get provider-specific auth requirements for an account
#[allow(dead_code)]
pub fn get_account_auth_provider(account: &AccountConfig) -> Option<&str> {
    match &account.auth {
        AuthMethod::OAuth { provider, .. } => Some(provider),
        _ => None,
    }
}
