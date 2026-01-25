use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Get current Unix timestamp in seconds.
///
/// Returns `None` if system time is before Unix epoch (extremely rare).
fn current_unix_timestamp() -> Option<u64> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// OAuth token storage structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenStorage {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

impl TokenStorage {
    /// Get the path to the auth token file
    fn auth_file_path() -> Result<PathBuf> {
        let config_dir = if let Ok(config_home) = std::env::var("RCO_CONFIG_HOME") {
            PathBuf::from(config_home)
        } else {
            let home = home_dir().context("Could not find home directory")?;
            home.join(".config").join("rustycommit")
        };

        // Ensure directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir.join("auth.json"))
    }

    /// Save tokens to file
    pub fn save(&self) -> Result<()> {
        let path = Self::auth_file_path()?;

        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(self).context("Failed to serialize token data")?;

        // Write to file with restricted permissions
        fs::write(&path, json).context("Failed to write auth token file")?;

        // Set file permissions to 600 (user read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms).context("Failed to set auth file permissions")?;
        }

        Ok(())
    }

    /// Load tokens from file
    pub fn load() -> Result<Option<Self>> {
        let path = Self::auth_file_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).context("Failed to read auth token file")?;

        let storage: TokenStorage =
            serde_json::from_str(&contents).context("Failed to parse auth token file")?;

        Ok(Some(storage))
    }

    /// Delete token file
    pub fn delete() -> Result<()> {
        let path = Self::auth_file_path()?;

        if path.exists() {
            fs::remove_file(&path).context("Failed to delete auth token file")?;
        }

        Ok(())
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = current_unix_timestamp().unwrap_or(u64::MAX);
            now >= expires_at
        } else {
            false
        }
    }

    /// Check if token will expire soon (within 5 minutes)
    #[allow(dead_code)]
    pub fn expires_soon(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = current_unix_timestamp().unwrap_or(u64::MAX);
            now >= expires_at.saturating_sub(300) // 5 minutes buffer, saturating to avoid underflow
        } else {
            false
        }
    }
}

/// Store OAuth tokens (file-based storage with fallback to secure storage)
pub fn store_tokens(
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<u64>,
) -> Result<()> {
    // First try secure storage if available
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            // Attempt secure storage; on failure fall through to file storage
            if let Err(e) =
                crate::config::secure_storage::store_secret("claude_access_token", access_token)
            {
                eprintln!(
                    "Note: Could not store access token in secure storage: {}",
                    e
                );
            } else {
                if let Some(refresh) = refresh_token {
                    if let Err(e) =
                        crate::config::secure_storage::store_secret("claude_refresh_token", refresh)
                    {
                        eprintln!(
                            "Note: Could not store refresh token in secure storage: {}",
                            e
                        );
                    }
                }

                if let Some(expires_in) = expires_in {
                    let expires_at = current_unix_timestamp().unwrap_or(u64::MAX) + expires_in;
                    if let Err(e) = crate::config::secure_storage::store_secret(
                        "claude_token_expires_at",
                        &expires_at.to_string(),
                    ) {
                        eprintln!(
                            "Note: Could not store token expiry in secure storage: {}",
                            e
                        );
                    }
                }

                // If we successfully stored the access token, prefer secure storage and return
                return Ok(());
            }
        }
    }

    // Fall back to file storage
    let expires_at = expires_in.map(|exp| current_unix_timestamp().unwrap_or(u64::MAX) + exp);

    let storage = TokenStorage {
        access_token: access_token.to_string(),
        refresh_token: refresh_token.map(|s| s.to_string()),
        expires_at,
        token_type: "Bearer".to_string(),
        scope: Some("openid profile email".to_string()),
    };

    storage.save()?;
    Ok(())
}

/// Get stored OAuth tokens (tries secure storage first, then file)
pub fn get_tokens() -> Result<Option<TokenStorage>> {
    // First try secure storage if available
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            if let Ok(Some(access_token)) =
                crate::config::secure_storage::get_secret("claude_access_token")
            {
                let refresh_token =
                    crate::config::secure_storage::get_secret("claude_refresh_token")
                        .ok()
                        .flatten();

                let expires_at =
                    crate::config::secure_storage::get_secret("claude_token_expires_at")
                        .ok()
                        .flatten()
                        .and_then(|s| s.parse::<u64>().ok());

                return Ok(Some(TokenStorage {
                    access_token,
                    refresh_token,
                    expires_at,
                    token_type: "Bearer".to_string(),
                    scope: Some("openid profile email".to_string()),
                }));
            }
        }
    }

    // Fall back to file storage
    TokenStorage::load()
}

/// Delete stored OAuth tokens (from both secure storage and file)
pub fn delete_tokens() -> Result<()> {
    // Delete from secure storage if available
    #[cfg(feature = "secure-storage")]
    {
        let _ = crate::config::secure_storage::delete_secret("claude_access_token");
        let _ = crate::config::secure_storage::delete_secret("claude_refresh_token");
        let _ = crate::config::secure_storage::delete_secret("claude_token_expires_at");
    }

    // Delete file storage
    TokenStorage::delete()?;
    Ok(())
}

/// Get just the access token (for convenience)
pub fn get_access_token() -> Result<Option<String>> {
    Ok(get_tokens()?.map(|t| t.access_token))
}

/// Check if we have valid (non-expired) tokens
pub fn has_valid_token() -> bool {
    if let Ok(Some(tokens)) = get_tokens() {
        !tokens.is_expired()
    } else {
        false
    }
}

// ============================================
// Account-scoped token storage (for multi-account support)
// ============================================

/// Generate storage key for an account
#[allow(dead_code)]
fn account_storage_key(account_id: &str, key_type: &str) -> String {
    format!("rco_account_{}_{}", account_id, key_type)
}

/// Store OAuth tokens for a specific account
#[allow(dead_code)]
pub fn store_tokens_for_account(
    _account_id: &str,
    access_token: &str,
    refresh_token: Option<&str>,
    expires_in: Option<u64>,
) -> Result<()> {
    // Use secure storage if available
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let access_key = account_storage_key(_account_id, "access_token");
            if let Err(e) = crate::config::secure_storage::store_secret(&access_key, access_token) {
                eprintln!(
                    "Note: Could not store access token in secure storage: {}",
                    e
                );
            } else {
                // Store refresh token
                if let Some(refresh) = refresh_token {
                    let refresh_key = account_storage_key(_account_id, "refresh_token");
                    let _ = crate::config::secure_storage::store_secret(&refresh_key, refresh);
                }

                // Store expiry
                if let Some(expires_in) = expires_in {
                    let expires_at = current_unix_timestamp().unwrap_or(u64::MAX) + expires_in;
                    let expiry_key = account_storage_key(_account_id, "token_expires_at");
                    let _ = crate::config::secure_storage::store_secret(
                        &expiry_key,
                        &expires_at.to_string(),
                    );
                }

                return Ok(());
            }
        }
    }

    // Fall back to file storage (not recommended for multi-account)
    // For now, just store in the legacy location
    store_tokens(access_token, refresh_token, expires_in)
}

/// Get OAuth tokens for a specific account
#[allow(dead_code)]
pub fn get_tokens_for_account(_account_id: &str) -> Result<Option<TokenStorage>> {
    // Try secure storage first
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let access_key = account_storage_key(_account_id, "access_token");
            if let Ok(Some(access_token)) = crate::config::secure_storage::get_secret(&access_key) {
                let refresh_key = account_storage_key(_account_id, "refresh_token");
                let refresh_token = crate::config::secure_storage::get_secret(&refresh_key)
                    .ok()
                    .flatten();

                let expiry_key = account_storage_key(_account_id, "token_expires_at");
                let expires_at = crate::config::secure_storage::get_secret(&expiry_key)
                    .ok()
                    .flatten()
                    .and_then(|s| s.parse::<u64>().ok());

                return Ok(Some(TokenStorage {
                    access_token,
                    refresh_token,
                    expires_at,
                    token_type: "Bearer".to_string(),
                    scope: Some("openid profile email".to_string()),
                }));
            }
        }
    }

    // Fall back to legacy storage for backward compatibility
    get_tokens()
}

/// Delete OAuth tokens for a specific account
#[allow(dead_code)]
pub fn delete_tokens_for_account(_account_id: &str) -> Result<()> {
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            for key_type in ["access_token", "refresh_token", "token_expires_at"] {
                let key = account_storage_key(_account_id, key_type);
                let _ = crate::config::secure_storage::delete_secret(&key);
            }
        }
    }

    Ok(())
}

/// Store API key for a specific account
#[allow(dead_code)]
pub fn store_api_key_for_account(_account_id: &str, _api_key: &str) -> Result<()> {
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let key = account_storage_key(_account_id, "api_key");
            crate::config::secure_storage::store_secret(&key, _api_key)?;
            return Ok(());
        }
    }

    // Fall back: can't store in file securely, warn user
    anyhow::bail!(
        "Secure storage not available. Cannot store API key for account '{}'.",
        _account_id
    )
}

/// Get API key for a specific account
#[allow(dead_code)]
pub fn get_api_key_for_account(_account_id: &str) -> Result<Option<String>> {
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let key = account_storage_key(_account_id, "api_key");
            return Ok(crate::config::secure_storage::get_secret(&key)?);
        }
    }

    Ok(None)
}

/// Store bearer token for a specific account
#[allow(dead_code)]
pub fn store_bearer_token_for_account(_account_id: &str, _token: &str) -> Result<()> {
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let key = account_storage_key(_account_id, "bearer_token");
            crate::config::secure_storage::store_secret(&key, _token)?;
            return Ok(());
        }
    }

    anyhow::bail!(
        "Secure storage not available. Cannot store bearer token for account '{}'.",
        _account_id
    )
}

/// Get bearer token for a specific account
#[allow(dead_code)]
pub fn get_bearer_token_for_account(_account_id: &str) -> Result<Option<String>> {
    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            let key = account_storage_key(_account_id, "bearer_token");
            return Ok(crate::config::secure_storage::get_secret(&key)?);
        }
    }

    Ok(None)
}

/// Delete all stored credentials for an account
#[allow(dead_code)]
pub fn delete_all_for_account(_account_id: &str) -> Result<()> {
    delete_tokens_for_account(_account_id)?;

    #[cfg(feature = "secure-storage")]
    {
        if crate::config::secure_storage::is_available() {
            for key_type in ["api_key", "bearer_token"] {
                let key = account_storage_key(_account_id, key_type);
                let _ = crate::config::secure_storage::delete_secret(&key);
            }
        }
    }

    Ok(())
}
