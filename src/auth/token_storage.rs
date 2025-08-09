use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
        let home = home_dir().context("Could not find home directory")?;
        let config_dir = home.join(".config").join("rustycommit");

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
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Check if token will expire soon (within 5 minutes)
    pub fn expires_soon(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            now >= expires_at - 300 // 5 minutes buffer
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
            crate::config::secure_storage::store_secret("claude_access_token", access_token)?;

            if let Some(refresh) = refresh_token {
                crate::config::secure_storage::store_secret("claude_refresh_token", refresh)?;
            }

            if let Some(expires_in) = expires_in {
                let expires_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs()
                    + expires_in;
                crate::config::secure_storage::store_secret(
                    "claude_token_expires_at",
                    &expires_at.to_string(),
                )?;
            }

            return Ok(());
        }
    }

    // Fall back to file storage
    let expires_at = expires_in.map(|exp| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + exp
    });

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
