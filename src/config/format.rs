use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::Config;

/// Configuration file format
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Toml,
    Json,
}

impl ConfigFormat {
    /// Detect format from file extension
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => ConfigFormat::Toml,
            Some("json") => ConfigFormat::Json,
            _ => ConfigFormat::Toml, // Default to TOML
        }
    }

    /// Parse config from string based on format
    pub fn parse(&self, contents: &str) -> Result<Config> {
        match self {
            ConfigFormat::Toml => toml::from_str(contents).context("Failed to parse TOML config"),
            ConfigFormat::Json => {
                serde_json::from_str(contents).context("Failed to parse JSON config")
            }
        }
    }

    /// Serialize config to string based on format
    pub fn serialize(&self, config: &Config) -> Result<String> {
        match self {
            ConfigFormat::Toml => {
                toml::to_string_pretty(config).context("Failed to serialize to TOML")
            }
            ConfigFormat::Json => {
                serde_json::to_string_pretty(config).context("Failed to serialize to JSON")
            }
        }
    }
}

/// Configuration locations with priority
#[derive(Debug)]
pub struct ConfigLocations {
    /// Repository-specific config (highest priority)
    pub repo: Option<PathBuf>,
    /// Global config
    pub global: PathBuf,
}

impl ConfigLocations {
    /// Get all config locations to check
    pub fn get() -> Result<Self> {
        // Global config locations (in priority order)
        let global = if let Ok(config_home) = std::env::var("RCO_CONFIG_HOME") {
            PathBuf::from(config_home).join("config.toml")
        } else {
            let home = dirs::home_dir().context("Could not find home directory")?;
            home.join(".config").join("rustycommit").join("config.toml")
        };

        // Repository-specific config (if in a git repo)
        let repo = if let Ok(repo) = git2::Repository::open_from_env() {
            let workdir = repo
                .workdir()
                .context("Could not get repository working directory")?;

            // Check for multiple possible config file names
            let possible_configs = [
                workdir.join(".rustycommit.toml"),
                workdir.join(".rustycommit.json"),
                workdir.join(".rco.toml"),
                workdir.join(".rco.json"),
            ];

            possible_configs.into_iter().find(|p| p.exists())
        } else {
            None
        };

        Ok(ConfigLocations { repo, global })
    }

    /// Load config with proper priority: repo > global > default
    pub fn load_merged() -> Result<Config> {
        let locations = Self::get()?;

        // Start with default config
        let mut config = Config::default();

        // Load global config if exists
        if locations.global.exists() {
            if let Ok(contents) = fs::read_to_string(&locations.global) {
                let format = ConfigFormat::from_path(&locations.global);
                if let Ok(global_config) = format.parse(&contents) {
                    config.merge(global_config);
                }
            }
        }

        // Load repo-specific config if exists (highest priority)
        if let Some(repo_path) = &locations.repo {
            if let Ok(contents) = fs::read_to_string(repo_path) {
                let format = ConfigFormat::from_path(repo_path);
                if let Ok(repo_config) = format.parse(&contents) {
                    config.merge(repo_config);
                }
            }
        }

        // Load values from environment variables (RCO_ prefix)
        config.load_from_environment();

        // Try to load API key from secure storage if not in file or env
        if config.api_key.is_none() {
            if let Ok(Some(key)) = crate::config::secure_storage::get_secret("RCO_API_KEY") {
                config.api_key = Some(key);
            }
        }

        // Also check for OAuth tokens
        if config.api_key.is_none() {
            if let Some(_token) = crate::auth::token_storage::get_access_token()
                .ok()
                .flatten()
            {
                // Token is handled separately in auth module, but we can set a flag
                // to indicate OAuth is available
            }
        }

        Ok(config)
    }

    /// Save config to appropriate location
    pub fn save(config: &Config, location: ConfigLocation) -> Result<()> {
        let locations = Self::get()?;

        let (path, format) = match location {
            ConfigLocation::Global => {
                // Ensure directory exists
                if let Some(parent) = locations.global.parent() {
                    fs::create_dir_all(parent)?;
                }
                (locations.global, ConfigFormat::Toml)
            }
            ConfigLocation::Repo => {
                // Use existing repo config or create new one
                let path = locations.repo.unwrap_or_else(|| {
                    if let Ok(repo) = git2::Repository::open_from_env() {
                        if let Some(workdir) = repo.workdir() {
                            return workdir.join(".rustycommit.toml");
                        }
                    }
                    PathBuf::from(".rustycommit.toml")
                });
                let format = ConfigFormat::from_path(&path);
                (path, format)
            }
        };

        let contents = format.serialize(config)?;
        fs::write(&path, contents).context("Failed to write config file")?;

        Ok(())
    }
}

/// Where to save configuration
#[derive(Debug, Clone, Copy)]
pub enum ConfigLocation {
    Global,
    Repo,
}
