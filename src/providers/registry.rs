//! Provider Registry - Central registry for all AI providers
//!
//! This module provides a extensible registry pattern for AI providers.
//! New providers can be added by implementing the `ProviderBuilder` trait
//! and registering them with the `ProviderRegistry`.

use crate::config::Config;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::RwLock;

/// Trait for building AI provider instances
pub trait ProviderBuilder: Send + Sync {
    /// The provider name/identifier
    fn name(&self) -> &'static str;

    /// Alternative names for this provider (aliases)
    fn aliases(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Provider category for documentation
    fn category(&self) -> ProviderCategory {
        ProviderCategory::Standard
    }

    /// Create a provider instance from config
    fn create(&self, config: &Config) -> Result<Box<dyn super::AIProvider>>;

    /// Whether this provider requires an API key
    fn requires_api_key(&self) -> bool {
        true
    }

    /// Default model for this provider (if applicable)
    fn default_model(&self) -> Option<&'static str> {
        None
    }
}

/// Provider categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderCategory {
    /// Direct API providers (OpenAI, Anthropic, etc.)
    Standard,
    /// OpenAI-compatible API providers
    OpenAICompatible,
    /// Self-hosted/local providers
    Local,
    /// Cloud marketplace providers
    Cloud,
}

/// Registry entry for a provider (metadata only, no builder)
#[derive(Clone)]
pub struct ProviderEntry {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub category: ProviderCategory,
    pub requires_api_key: bool,
    pub default_model: Option<&'static str>,
}

impl ProviderEntry {
    pub fn from_builder(builder: &dyn ProviderBuilder) -> Self {
        Self {
            name: builder.name(),
            aliases: builder.aliases(),
            category: builder.category(),
            requires_api_key: builder.requires_api_key(),
            default_model: builder.default_model(),
        }
    }

    /// Check if this entry matches a provider name
    pub fn matches(&self, provider: &str) -> bool {
        let lower = provider.to_lowercase();
        self.name.eq_ignore_ascii_case(&lower)
            || self.aliases.iter().any(|&a| a.eq_ignore_ascii_case(&lower))
    }
}

/// The provider registry - a thread-safe registry of all available providers
pub struct ProviderRegistry {
    entries: RwLock<HashMap<&'static str, ProviderEntry>>,
    builders: RwLock<HashMap<&'static str, Box<dyn ProviderBuilder>>>,
    by_alias: RwLock<HashMap<&'static str, &'static str>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            builders: RwLock::new(HashMap::new()),
            by_alias: RwLock::new(HashMap::new()),
        }
    }

    /// Register a provider builder
    pub fn register(&self, builder: Box<dyn ProviderBuilder>) {
        let name = builder.name();
        let entry = ProviderEntry::from_builder(&*builder);

        // Register primary name
        self.entries.write().unwrap().insert(name, entry.clone());
        self.builders.write().unwrap().insert(name, builder);

        // Register aliases
        for &alias in &entry.aliases {
            self.by_alias.write().unwrap().insert(alias, name);
        }
    }

    /// Get a provider entry by name or alias
    pub fn get(&self, provider: &str) -> Option<ProviderEntry> {
        let lower = provider.to_lowercase();

        // Try direct lookup
        if let Some(entry) = self.entries.read().unwrap().get(lower.as_str()) {
            return Some(entry.clone());
        }

        // Try alias lookup
        if let Some(&primary) = self.by_alias.read().unwrap().get(lower.as_str()) {
            return self.entries.read().unwrap().get(primary).cloned();
        }

        None
    }

    /// Get all registered providers
    pub fn all(&self) -> Vec<ProviderEntry> {
        self.entries.read().unwrap().values().cloned().collect()
    }

    /// Get providers by category
    pub fn by_category(&self, category: ProviderCategory) -> Vec<ProviderEntry> {
        self.entries
            .read()
            .unwrap()
            .values()
            .filter(|e| e.category == category)
            .cloned()
            .collect()
    }

    /// Check if any providers are registered
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Get count of registered providers
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Create a provider instance
    pub fn create(&self, name: &str, config: &Config) -> Result<Option<Box<dyn super::AIProvider>>> {
        let lower = name.to_lowercase();

        // Try direct lookup first
        if let Some(builder) = self.builders.read().unwrap().get(lower.as_str()) {
            return Ok(Some(builder.create(config)?));
        }

        // Try alias lookup
        if let Some(&primary) = self.by_alias.read().unwrap().get(lower.as_str()) {
            if let Some(builder) = self.builders.read().unwrap().get(primary) {
                return Ok(Some(builder.create(config)?));
            }
        }

        Ok(None)
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
