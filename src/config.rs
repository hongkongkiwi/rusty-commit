pub mod accounts;
pub mod format;
pub mod migrations;
pub mod secure_storage;

use anyhow::{Context, Result};
use colored::Colorize;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    // API Configuration
    pub api_key: Option<String>,
    pub api_url: Option<String>,
    pub ai_provider: Option<String>,
    pub model: Option<String>,

    // Token limits
    pub tokens_max_input: Option<usize>,
    pub tokens_max_output: Option<u32>,

    // Commit message configuration
    pub commit_type: Option<String>,
    pub emoji: Option<bool>,
    pub description: Option<bool>,
    pub description_capitalize: Option<bool>,
    pub description_add_period: Option<bool>,
    pub description_max_length: Option<usize>,

    // Language and customization
    pub language: Option<String>,
    pub message_template_placeholder: Option<String>,
    pub prompt_module: Option<String>,

    // Behavior
    pub gitpush: Option<bool>,
    pub one_line_commit: Option<bool>,
    pub why: Option<bool>,
    pub omit_scope: Option<bool>,
    pub generate_count: Option<u8>,
    pub clipboard_on_timeout: Option<bool>,

    // GitHub Actions
    pub action_enabled: Option<bool>,

    // Testing
    pub test_mock_type: Option<String>,

    // Hooks
    pub hook_auto_uncomment: Option<bool>,
    pub pre_gen_hook: Option<Vec<String>>,
    pub pre_commit_hook: Option<Vec<String>>,
    pub post_commit_hook: Option<Vec<String>>,
    pub hook_strict: Option<bool>,
    pub hook_timeout_ms: Option<u64>,

    // Global commitlint configuration
    pub commitlint_config: Option<String>,
    pub custom_prompt: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            api_url: None,
            ai_provider: Some("openai".to_string()),
            model: Some("gpt-3.5-turbo".to_string()),
            tokens_max_input: Some(4096),
            tokens_max_output: Some(500),
            commit_type: Some("conventional".to_string()),
            emoji: Some(false),
            description: Some(false),
            description_capitalize: Some(true),
            description_add_period: Some(false),
            description_max_length: Some(100),
            language: Some("en".to_string()),
            message_template_placeholder: Some("$msg".to_string()),
            prompt_module: Some("conventional-commit".to_string()),
            gitpush: Some(false),
            one_line_commit: Some(false),
            why: Some(false),
            omit_scope: Some(false),
            generate_count: Some(1),
            clipboard_on_timeout: Some(true),
            action_enabled: Some(false),
            test_mock_type: None,
            hook_auto_uncomment: Some(false),
            pre_gen_hook: None,
            pre_commit_hook: None,
            post_commit_hook: None,
            hook_strict: Some(true),
            hook_timeout_ms: Some(30000),
            commitlint_config: None,
            custom_prompt: None,
        }
    }
}

impl Config {
    /// Get the new global config path
    #[allow(dead_code)]
    pub fn global_config_path() -> Result<PathBuf> {
        if let Ok(config_home) = env::var("RCO_CONFIG_HOME") {
            Ok(PathBuf::from(config_home).join("config.toml"))
        } else {
            let home = home_dir().context("Could not find home directory")?;
            Ok(home.join(".config").join("rustycommit").join("config.toml"))
        }
    }

    /// Load configuration with proper priority handling
    pub fn load() -> Result<Self> {
        // Use the new format system to load with priority
        format::ConfigLocations::load_merged()
    }

    pub fn save(&self) -> Result<()> {
        // Save to global config by default
        self.save_to(format::ConfigLocation::Global)
    }

    /// Save configuration to a specific location
    pub fn save_to(&self, location: format::ConfigLocation) -> Result<()> {
        // Create a copy for saving (without sensitive data)
        let mut save_config = self.clone();

        // If we have an API key and secure storage is available, store it securely
        if let Some(ref api_key) = self.api_key {
            if secure_storage::is_available() {
                match secure_storage::store_secret("RCO_API_KEY", api_key) {
                    Ok(_) => {
                        // Don't save API key to file if stored securely
                        save_config.api_key = None;
                    }
                    Err(e) => {
                        // Fall back to file storage; keep api_key in file
                        eprintln!("Warning: Secure storage unavailable, falling back to file: {e}");
                    }
                }
            }
        }

        format::ConfigLocations::save(&save_config, location)
    }

    /// Helper function to get environment variable with RCO_ prefix
    fn get_env_var(base_name: &str) -> Option<String> {
        let rco_key = format!("RCO_{}", base_name);

        // Check RCO_ prefix
        env::var(&rco_key).ok()
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        // Handle undefined/null values
        if value == "undefined" || value == "null" {
            return Ok(());
        }

        match key {
            // Support RCO_ prefix
            "RCO_API_KEY" => {
                self.api_key = Some(value.to_string());
                // Also try to store in secure storage (use RCO_ key)
                if secure_storage::is_available() {
                    let _ = secure_storage::store_secret("RCO_API_KEY", value);
                }
            }
            "RCO_API_URL" => self.api_url = Some(value.to_string()),
            "RCO_AI_PROVIDER" => self.ai_provider = Some(value.to_string()),
            "RCO_MODEL" => self.model = Some(value.to_string()),
            "RCO_TOKENS_MAX_INPUT" => {
                self.tokens_max_input = Some(
                    value
                        .parse()
                        .context("Invalid number for TOKENS_MAX_INPUT")?,
                );
            }
            "RCO_TOKENS_MAX_OUTPUT" => {
                self.tokens_max_output = Some(
                    value
                        .parse()
                        .context("Invalid number for TOKENS_MAX_OUTPUT")?,
                );
            }
            "RCO_COMMIT_TYPE" => {
                self.commit_type = Some(value.to_string());
            }
            "RCO_PROMPT_MODULE" => {
                // Map legacy prompt module to commit type
                let commit_type = match value {
                    "conventional-commit" => "conventional",
                    _ => value,
                };
                self.commit_type = Some(commit_type.to_string());
                self.prompt_module = Some(value.to_string());
            }
            "RCO_EMOJI" => {
                self.emoji = Some(value.parse().context("Invalid boolean for EMOJI")?);
            }
            "RCO_DESCRIPTION_CAPITALIZE" => {
                self.description_capitalize = Some(
                    value
                        .parse()
                        .context("Invalid boolean for DESCRIPTION_CAPITALIZE")?,
                );
            }
            "RCO_DESCRIPTION_ADD_PERIOD" => {
                self.description_add_period = Some(
                    value
                        .parse()
                        .context("Invalid boolean for DESCRIPTION_ADD_PERIOD")?,
                );
            }
            "RCO_DESCRIPTION_MAX_LENGTH" => {
                self.description_max_length = Some(
                    value
                        .parse()
                        .context("Invalid number for DESCRIPTION_MAX_LENGTH")?,
                );
            }
            "RCO_LANGUAGE" => self.language = Some(value.to_string()),
            "RCO_MESSAGE_TEMPLATE_PLACEHOLDER" => {
                self.message_template_placeholder = Some(value.to_string());
            }
            "RCO_GITPUSH" => {
                self.gitpush = Some(value.parse().context("Invalid boolean for GITPUSH")?);
            }
            "RCO_ONE_LINE_COMMIT" => {
                self.one_line_commit = Some(
                    value
                        .parse()
                        .context("Invalid boolean for ONE_LINE_COMMIT")?,
                );
            }
            "RCO_ACTION_ENABLED" => {
                self.action_enabled = Some(
                    value
                        .parse()
                        .context("Invalid boolean for ACTION_ENABLED")?,
                );
            }
            "RCO_DESCRIPTION" => {
                self.description = Some(value.parse().context("Invalid boolean for DESCRIPTION")?);
            }
            "RCO_WHY" => {
                self.why = Some(value.parse().context("Invalid boolean for WHY")?);
            }
            "RCO_OMIT_SCOPE" => {
                self.omit_scope = Some(value.parse().context("Invalid boolean for OMIT_SCOPE")?);
            }
            "RCO_TEST_MOCK_TYPE" => {
                self.test_mock_type = Some(value.to_string());
            }
            "RCO_HOOK_AUTO_UNCOMMENT" => {
                self.hook_auto_uncomment = Some(
                    value
                        .parse()
                        .context("Invalid boolean for HOOK_AUTO_UNCOMMENT")?,
                );
            }
            "RCO_PRE_GEN_HOOK" => {
                let items = value
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.pre_gen_hook = Some(items);
            }
            "RCO_PRE_COMMIT_HOOK" => {
                let items = value
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.pre_commit_hook = Some(items);
            }
            "RCO_POST_COMMIT_HOOK" => {
                let items = value
                    .split(';')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.post_commit_hook = Some(items);
            }
            "RCO_HOOK_STRICT" => {
                self.hook_strict = Some(value.parse().context("Invalid boolean for HOOK_STRICT")?);
            }
            "RCO_HOOK_TIMEOUT_MS" => {
                self.hook_timeout_ms = Some(
                    value
                        .parse()
                        .context("Invalid number for HOOK_TIMEOUT_MS")?,
                );
            }
            "RCO_COMMITLINT_CONFIG" => {
                self.commitlint_config = Some(value.to_string());
            }
            "RCO_CUSTOM_PROMPT" => {
                self.custom_prompt = Some(value.to_string());
            }
            "RCO_GENERATE_COUNT" => {
                self.generate_count = Some(
                    value
                        .parse()
                        .context("Invalid number for GENERATE_COUNT (1-5)")?,
                );
            }
            "RCO_CLIPBOARD_ON_TIMEOUT" => {
                self.clipboard_on_timeout = Some(
                    value.parse().context("Invalid boolean for CLIPBOARD_ON_TIMEOUT")?,
                );
            }
            // Ignore unsupported keys
            "RCO_API_CUSTOM_HEADERS" => {
                // Silently ignore these legacy keys
                return Ok(());
            }
            _ => anyhow::bail!("Unknown configuration key: {}", key),
        }

        self.save()?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<String> {
        let value = match key {
            "RCO_API_KEY" => {
                // Try to get from memory first, then from secure storage
                self.api_key
                    .as_ref()
                    .map(|s| s.to_string())
                    .or_else(|| secure_storage::get_secret("RCO_API_KEY").ok().flatten())
            }
            "RCO_API_URL" => self.api_url.as_ref().map(|s| s.to_string()),
            "RCO_AI_PROVIDER" => self.ai_provider.as_ref().map(|s| s.to_string()),
            "RCO_MODEL" => self.model.as_ref().map(|s| s.to_string()),
            "RCO_TOKENS_MAX_INPUT" => self.tokens_max_input.map(|v| v.to_string()),
            "RCO_TOKENS_MAX_OUTPUT" => self.tokens_max_output.map(|v| v.to_string()),
            "RCO_COMMIT_TYPE" => self.commit_type.as_ref().map(|s| s.to_string()),
            "RCO_EMOJI" => self.emoji.map(|v| v.to_string()),
            "RCO_DESCRIPTION_CAPITALIZE" => self.description_capitalize.map(|v| v.to_string()),
            "RCO_DESCRIPTION_ADD_PERIOD" => self.description_add_period.map(|v| v.to_string()),
            "RCO_DESCRIPTION_MAX_LENGTH" => self.description_max_length.map(|v| v.to_string()),
            "RCO_LANGUAGE" => self.language.as_ref().map(|s| s.to_string()),
            "RCO_MESSAGE_TEMPLATE_PLACEHOLDER" => self
                .message_template_placeholder
                .as_ref()
                .map(|s| s.to_string()),
            "RCO_GITPUSH" => self.gitpush.map(|v| v.to_string()),
            "RCO_ONE_LINE_COMMIT" => self.one_line_commit.map(|v| v.to_string()),
            "RCO_ACTION_ENABLED" => self.action_enabled.map(|v| v.to_string()),
            "RCO_COMMITLINT_CONFIG" => self.commitlint_config.as_ref().map(|s| s.to_string()),
            "RCO_CUSTOM_PROMPT" => self.custom_prompt.as_ref().map(|s| s.to_string()),
            "RCO_GENERATE_COUNT" => self.generate_count.map(|v| v.to_string()),
            "RCO_CLIPBOARD_ON_TIMEOUT" => self.clipboard_on_timeout.map(|v| v.to_string()),
            _ => None,
        };

        value.ok_or_else(|| anyhow::anyhow!("Configuration key '{}' not found or not set", key))
    }

    pub fn reset(&mut self, keys: Option<&[String]>) -> Result<()> {
        if let Some(key_list) = keys {
            let default = Self::default();
            for key in key_list {
                match key.as_str() {
                    "RCO_API_KEY" => {
                        self.api_key = default.api_key.clone();
                        // Also clear from secure storage
                        let _ = secure_storage::delete_secret("RCO_API_KEY");
                    }
                    "RCO_API_URL" => self.api_url = default.api_url.clone(),
                    "RCO_AI_PROVIDER" => self.ai_provider = default.ai_provider.clone(),
                    "RCO_MODEL" => self.model = default.model.clone(),
                    "RCO_TOKENS_MAX_INPUT" => self.tokens_max_input = default.tokens_max_input,
                    "RCO_TOKENS_MAX_OUTPUT" => self.tokens_max_output = default.tokens_max_output,
                    "RCO_COMMIT_TYPE" => self.commit_type = default.commit_type.clone(),
                    "RCO_EMOJI" => self.emoji = default.emoji,
                    "RCO_DESCRIPTION_CAPITALIZE" => {
                        self.description_capitalize = default.description_capitalize
                    }
                    "RCO_DESCRIPTION_ADD_PERIOD" => {
                        self.description_add_period = default.description_add_period
                    }
                    "RCO_DESCRIPTION_MAX_LENGTH" => {
                        self.description_max_length = default.description_max_length
                    }
                    "RCO_LANGUAGE" => self.language = default.language.clone(),
                    "RCO_MESSAGE_TEMPLATE_PLACEHOLDER" => {
                        self.message_template_placeholder =
                            default.message_template_placeholder.clone()
                    }
                    "RCO_GITPUSH" => self.gitpush = default.gitpush,
                    "RCO_ONE_LINE_COMMIT" => self.one_line_commit = default.one_line_commit,
                    "RCO_ACTION_ENABLED" => self.action_enabled = default.action_enabled,
                    "RCO_PRE_GEN_HOOK" => self.pre_gen_hook = default.pre_gen_hook.clone(),
                    "RCO_PRE_COMMIT_HOOK" => self.pre_commit_hook = default.pre_commit_hook.clone(),
                    "RCO_POST_COMMIT_HOOK" => {
                        self.post_commit_hook = default.post_commit_hook.clone()
                    }
                    "RCO_HOOK_STRICT" => self.hook_strict = default.hook_strict,
                    "RCO_HOOK_TIMEOUT_MS" => self.hook_timeout_ms = default.hook_timeout_ms,
                    "RCO_GENERATE_COUNT" => self.generate_count = default.generate_count,
                    "RCO_CLIPBOARD_ON_TIMEOUT" => self.clipboard_on_timeout = default.clipboard_on_timeout,
                    _ => anyhow::bail!("Unknown configuration key: {}", key),
                }
            }
        } else {
            *self = Self::default();
        }

        self.save()?;
        Ok(())
    }

    /// Load and merge global commitlint configuration
    pub fn load_with_commitlint(&mut self) -> Result<()> {
        // First check for COMMITLINT_CONFIG environment variable
        if let Ok(commitlint_path) = env::var("COMMITLINT_CONFIG") {
            self.commitlint_config = Some(commitlint_path);
        }

        // If no explicit config path, check default locations
        if self.commitlint_config.is_none() {
            let home = home_dir().context("Could not find home directory")?;

            // Check for global commitlint configs in priority order
            let possible_paths = [
                home.join(".commitlintrc.js"),
                home.join(".commitlintrc.json"),
                home.join(".commitlintrc.yml"),
                home.join(".commitlintrc.yaml"),
                home.join("commitlint.config.js"),
            ];

            for path in &possible_paths {
                if path.exists() {
                    self.commitlint_config = Some(path.to_string_lossy().to_string());
                    break;
                }
            }
        }

        Ok(())
    }

    /// Load commitlint rules and modify commit type accordingly
    pub fn apply_commitlint_rules(&mut self) -> Result<()> {
        if let Some(ref config_path) = self.commitlint_config.clone() {
            let path = PathBuf::from(config_path);
            if path.exists() {
                // For now, just set to conventional commits if commitlint config exists
                // Full commitlint parsing would require a JS engine or specific parsing
                if self.commit_type.is_none() {
                    self.commit_type = Some("conventional".to_string());
                }

                // In a full implementation, we would parse the commitlint config
                // and extract rules, but for now we'll use conventional commits
                println!("📋 Found commitlint config at: {}", config_path);
                println!("🔧 Using conventional commit format for consistency");
            }
        }
        Ok(())
    }

    /// Get the effective prompt (custom or generated)
    pub fn get_effective_prompt(
        &self,
        diff: &str,
        context: Option<&str>,
        full_gitmoji: bool,
    ) -> String {
        if let Some(ref custom_prompt) = self.custom_prompt {
            // Security warning: custom prompts receive diff content
            tracing::warn!(
                "Using custom prompt template - diff content will be included in the prompt. \
                Ensure your custom prompt does not exfiltrate or log sensitive code."
            );
            eprintln!(
                "{}",
                "Warning: Using custom prompt template. Your diff content will be sent to the AI provider."
                    .yellow()
            );

            // Replace placeholders in custom prompt
            let mut prompt = custom_prompt.clone();
            prompt = prompt.replace("$diff", diff);
            if let Some(ctx) = context {
                prompt = prompt.replace("$context", ctx);
            }
            prompt
        } else {
            // Use the standard prompt generation
            super::providers::build_prompt(diff, context, self, full_gitmoji)
        }
    }

    /// Merge another config into this one (other takes priority over self)
    pub fn merge(&mut self, other: Config) {
        macro_rules! merge_field {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field;
                }
            };
        }

        merge_field!(api_key);
        merge_field!(api_url);
        merge_field!(ai_provider);
        merge_field!(model);
        merge_field!(tokens_max_input);
        merge_field!(tokens_max_output);
        merge_field!(commit_type);
        merge_field!(emoji);
        merge_field!(description);
        merge_field!(description_capitalize);
        merge_field!(description_add_period);
        merge_field!(description_max_length);
        merge_field!(language);
        merge_field!(message_template_placeholder);
        merge_field!(prompt_module);
        merge_field!(gitpush);
        merge_field!(one_line_commit);
        merge_field!(why);
        merge_field!(omit_scope);
        merge_field!(action_enabled);
        merge_field!(test_mock_type);
        merge_field!(hook_auto_uncomment);
        merge_field!(pre_gen_hook);
        merge_field!(pre_commit_hook);
        merge_field!(post_commit_hook);
        merge_field!(hook_strict);
        merge_field!(hook_timeout_ms);
        merge_field!(commitlint_config);
        merge_field!(custom_prompt);
        merge_field!(generate_count);
        merge_field!(clipboard_on_timeout);
    }

    /// Load configuration values from environment variables
    /// Uses RCO_ environment variables
    pub fn load_from_environment(&mut self) {
        // Macro to reduce code duplication
        macro_rules! load_env_var {
            ($field:ident, $base_name:expr) => {
                if let Some(value) = Self::get_env_var($base_name) {
                    self.$field = Some(value);
                }
            };
        }

        macro_rules! load_env_var_parse {
            ($field:ident, $base_name:expr, $type:ty) => {
                if let Some(value) = Self::get_env_var($base_name) {
                    if let Ok(parsed) = value.parse::<$type>() {
                        self.$field = Some(parsed);
                    }
                }
            };
        }

        load_env_var!(api_key, "API_KEY");
        load_env_var!(api_url, "API_URL");
        load_env_var!(ai_provider, "AI_PROVIDER");
        load_env_var!(model, "MODEL");
        load_env_var_parse!(tokens_max_input, "TOKENS_MAX_INPUT", usize);
        load_env_var_parse!(tokens_max_output, "TOKENS_MAX_OUTPUT", u32);
        load_env_var!(commit_type, "COMMIT_TYPE");
        load_env_var_parse!(emoji, "EMOJI", bool);
        load_env_var_parse!(description, "DESCRIPTION", bool);
        load_env_var_parse!(description_capitalize, "DESCRIPTION_CAPITALIZE", bool);
        load_env_var_parse!(description_add_period, "DESCRIPTION_ADD_PERIOD", bool);
        load_env_var_parse!(description_max_length, "DESCRIPTION_MAX_LENGTH", usize);
        load_env_var!(language, "LANGUAGE");
        load_env_var!(message_template_placeholder, "MESSAGE_TEMPLATE_PLACEHOLDER");
        load_env_var!(prompt_module, "PROMPT_MODULE");
        load_env_var_parse!(gitpush, "GITPUSH", bool);
        load_env_var_parse!(one_line_commit, "ONE_LINE_COMMIT", bool);
        load_env_var_parse!(why, "WHY", bool);
        load_env_var_parse!(omit_scope, "OMIT_SCOPE", bool);
        load_env_var_parse!(action_enabled, "ACTION_ENABLED", bool);
        load_env_var!(test_mock_type, "TEST_MOCK_TYPE");
        load_env_var_parse!(hook_auto_uncomment, "HOOK_AUTO_UNCOMMENT", bool);
        load_env_var!(commitlint_config, "COMMITLINT_CONFIG");
        load_env_var!(custom_prompt, "CUSTOM_PROMPT");
        load_env_var_parse!(generate_count, "GENERATE_COUNT", u8);
        load_env_var_parse!(clipboard_on_timeout, "CLIPBOARD_ON_TIMEOUT", bool);
    }
}

// ============================================
// Multi-account support methods
// ============================================

#[allow(dead_code)]
impl Config {
    /// Get the active account config, if available
    pub fn get_active_account(&self) -> Result<Option<accounts::AccountConfig>> {
        if let Some(accounts_config) = accounts::AccountsConfig::load()? {
            if let Some(account) = accounts_config.get_active_account() {
                return Ok(Some(account.clone()));
            }
        }
        Ok(None)
    }

    /// Check if we have any accounts configured
    pub fn has_accounts(&self) -> bool {
        accounts::AccountsConfig::load()
            .map(|c| c.map(|ac| !ac.accounts.is_empty()).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Get a specific account by alias
    pub fn get_account(&self, alias: &str) -> Result<Option<accounts::AccountConfig>> {
        if let Some(accounts_config) = accounts::AccountsConfig::load()? {
            if let Some(account) = accounts_config.get_account(alias) {
                return Ok(Some(account.clone()));
            }
        }
        Ok(None)
    }

    /// List all accounts
    pub fn list_accounts(&self) -> Result<Vec<accounts::AccountConfig>> {
        if let Some(accounts_config) = accounts::AccountsConfig::load()? {
            Ok(accounts_config.list_accounts().into_iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Set an account as the default (active) account
    pub fn set_default_account(&mut self, alias: &str) -> Result<()> {
        let mut accounts_config = accounts::AccountsConfig::load()?.unwrap_or_default();
        accounts_config.set_active_account(alias)?;
        accounts_config.save()?;
        Ok(())
    }

    /// Remove an account
    pub fn remove_account(&mut self, alias: &str) -> Result<()> {
        let mut accounts_config = accounts::AccountsConfig::load()?.unwrap_or_default();
        if accounts_config.remove_account(alias) {
            accounts_config.save()?;
        }
        Ok(())
    }
}
