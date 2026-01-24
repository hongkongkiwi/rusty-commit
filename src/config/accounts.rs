use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Authentication method for an account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum AuthMethod {
    #[serde(rename = "api_key")]
    ApiKey { key_id: String },
    #[serde(rename = "oauth")]
    OAuth { provider: String, account_id: String },
    #[serde(rename = "env_var")]
    EnvVar { name: String },
    #[serde(rename = "bearer")]
    Bearer { token_id: String },
}

/// Single account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub alias: String,
    pub provider: String,
    pub api_url: Option<String>,
    pub model: Option<String>,
    pub auth: AuthMethod,
    pub tokens_max_input: Option<usize>,
    pub tokens_max_output: Option<u32>,
    #[serde(default)]
    pub is_default: bool,
}

/// All accounts configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountsConfig {
    pub active_account: Option<String>,
    pub accounts: HashMap<String, AccountConfig>,
}

impl AccountsConfig {
    /// Get the path to the accounts file
    fn accounts_file_path() -> Result<PathBuf> {
        let config_dir = if let Ok(config_home) = std::env::var("RCO_CONFIG_HOME") {
            PathBuf::from(config_home)
        } else {
            let home = dirs::home_dir().context("Could not find home directory")?;
            home.join(".config").join("rustycommit")
        };

        // Ensure directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        }

        Ok(config_dir.join("accounts.toml"))
    }

    /// Load accounts from file
    pub fn load() -> Result<Option<Self>> {
        let path = Self::accounts_file_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&path).context("Failed to read accounts file")?;

        let config: AccountsConfig =
            toml::from_str(&contents).context("Failed to parse accounts file")?;

        Ok(Some(config))
    }

    /// Save accounts to file
    pub fn save(&self) -> Result<()> {
        let path = Self::accounts_file_path()?;

        let toml_content = toml::to_string_pretty(self).context("Failed to serialize accounts")?;

        fs::write(&path, toml_content).context("Failed to write accounts file")?;

        // Set file permissions to 600 (user read/write only) on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&path, perms).context("Failed to set accounts file permissions")?;
        }

        Ok(())
    }

    /// Get an account by alias
    pub fn get_account(&self, alias: &str) -> Option<&AccountConfig> {
        self.accounts.get(alias)
    }

    /// Get an account by alias (mutable)
    pub fn get_account_mut(&mut self, alias: &str) -> Option<&mut AccountConfig> {
        self.accounts.get_mut(alias)
    }

    /// Add or update an account
    pub fn add_account(&mut self, account: AccountConfig) {
        self.accounts.insert(account.alias.clone(), account);
    }

    /// Remove an account
    pub fn remove_account(&mut self, alias: &str) -> bool {
        self.accounts.remove(alias).is_some()
    }

    /// List all accounts
    pub fn list_accounts(&self) -> Vec<&AccountConfig> {
        self.accounts.values().collect()
    }

    /// Set an account as active
    pub fn set_active_account(&mut self, alias: &str) -> Result<()> {
        if !self.accounts.contains_key(alias) {
            anyhow::bail!("Account '{}' not found", alias);
        }
        self.active_account = Some(alias.to_string());
        Ok(())
    }

    /// Get the active account
    pub fn get_active_account(&self) -> Option<&AccountConfig> {
        if let Some(alias) = &self.active_account {
            self.accounts.get(alias)
        } else {
            None
        }
    }

    /// Get active account alias
    pub fn get_active_alias(&self) -> Option<&str> {
        self.active_account.as_deref()
    }

    /// Get a unique key ID for an account and auth method
    pub fn get_key_id(account_alias: &str, auth_type: &str) -> String {
        format!("rco_{}_{}", account_alias, auth_type)
    }
}

/// Get the secure storage key prefix for an account
pub fn account_storage_key(account_alias: &str, key_type: &str) -> String {
    format!("rco_account_{}_{}", account_alias, key_type)
}

/// Delete all storage keys for an account
pub fn delete_account_storage(account_alias: &str) {
    #[cfg(feature = "secure-storage")]
    {
        use crate::config::secure_storage;

        for key_type in ["access_token", "refresh_token", "api_key", "bearer_token"] {
            let key = account_storage_key(account_alias, key_type);
            let _ = secure_storage::delete_secret(&key);
        }
    }
    // For file-based storage, keys are stored in the accounts.toml itself
    // API keys in auth method are encrypted/obfuscated if needed
}
