//! Error formatting utilities for Rusty Commit CLI.
//!
//! Provides structured, beautiful error messages with hints and context.

use anyhow::Result;
use colored::Colorize;

use super::styling::{Styling, Theme};

/// A structured error with context and hints.
#[derive(Debug, Clone)]
pub struct StructuredError {
    /// The main error message.
    message: String,
    /// The provider that caused the error (if applicable).
    provider: Option<String>,
    /// The model that was being used (if applicable).
    model: Option<String>,
    /// The underlying error (if available).
    underlying: Option<String>,
    /// Contextual information.
    context: Vec<(String, String)>,
    /// Helpful hints for resolution.
    hints: Vec<String>,
    /// Exit code for the error.
    exit_code: i32,
}

impl StructuredError {
    /// Create a new structured error.
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            provider: None,
            model: None,
            underlying: None,
            context: Vec::new(),
            hints: Vec::new(),
            exit_code: 1,
        }
    }

    /// Set the provider.
    pub fn with_provider(mut self, provider: &str) -> Self {
        self.provider = Some(provider.to_string());
        self
    }

    /// Set the model.
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// Set the underlying error.
    pub fn with_underlying(mut self, error: &str) -> Self {
        self.underlying = Some(error.to_string());
        self
    }

    /// Add contextual information.
    #[allow(dead_code)]
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.push((key.to_string(), value.to_string()));
        self
    }

    /// Add a hint.
    #[allow(dead_code)]
    pub fn with_hint(mut self, hint: &str) -> Self {
        self.hints.push(hint.to_string());
        self
    }

    /// Add multiple hints.
    pub fn with_hints<T: IntoIterator<Item = String>>(mut self, hints: T) -> Self {
        self.hints.extend(hints);
        self
    }

    /// Set the exit code.
    pub fn with_exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    /// Get the exit code.
    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    /// Format the error for display.
    pub fn display(&self, _theme: &Theme) -> String {
        let mut output = String::new();

        // Error header
        output.push_str(&format!(
            "{} {}\n",
            Styling::error("X"),
            Styling::header(&self.message)
        ));

        // Divider
        output.push_str(&Styling::divider(50));
        output.push('\n');

        // Context information
        if let Some(ref provider) = self.provider {
            output.push_str(&format!("{}: {}\n", "Provider".dimmed(), provider));
        }
        if let Some(ref model) = self.model {
            output.push_str(&format!("{}: {}\n", "Model".dimmed(), model));
        }
        if let Some(ref underlying) = self.underlying {
            output.push_str(&format!("{}: {}\n", "Error".dimmed(), underlying));
        }

        // Additional context
        for (key, value) in &self.context {
            output.push_str(&format!("{}: {}\n", key.dimmed(), value));
        }

        // Hints
        if !self.hints.is_empty() {
            output.push('\n');
            output.push_str("Suggestions:\n");
            for hint in &self.hints {
                output.push_str(&format!("  - {}\n", hint));
            }
        }

        output
    }

    /// Format as JSON.
    #[allow(dead_code)]
    pub fn to_json(&self) -> String {
        use serde_json::json;

        let hints_array: Vec<String> = self.hints.clone();
        let context_obj: serde_json::Map<String, serde_json::Value> = self
            .context
            .iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();

        let obj = json!({
            "error": self.message,
            "provider": self.provider,
            "model": self.model,
            "underlying": self.underlying,
            "context": context_obj,
            "hints": hints_array,
            "exit_code": self.exit_code,
        });

        serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format as markdown.
    #[allow(dead_code)]
    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str("## Error\n\n");
        output.push_str(&format!("**{}**\n\n", self.message));

        if let Some(ref provider) = self.provider {
            output.push_str(&format!("- **Provider:** {}\n", provider));
        }
        if let Some(ref model) = self.model {
            output.push_str(&format!("- **Model:** {}\n", model));
        }
        if let Some(ref underlying) = self.underlying {
            output.push_str(&format!("- **Error:** {}\n", underlying));
        }

        if !self.hints.is_empty() {
            output.push_str("\n## Suggestions\n\n");
            for hint in &self.hints {
                output.push_str(&format!("- {}\n", hint));
            }
        }

        output
    }
}

#[allow(dead_code)]
/// Helper to convert anyhow errors to structured errors.
pub trait ToStructured {
    fn to_structured(&self) -> StructuredError;
}

impl ToStructured for anyhow::Error {
    fn to_structured(&self) -> StructuredError {
        StructuredError::new(&self.to_string())
    }
}

/// Common error patterns with built-in hints.
#[allow(dead_code)]
pub mod patterns {
    use super::*;

    /// Rate limit exceeded error.
    pub fn rate_limit(provider: &str, model: &str) -> StructuredError {
        StructuredError::new("API rate limit exceeded")
            .with_provider(provider)
            .with_model(model)
            .with_hints(vec![
                "Wait a few seconds and try again".to_string(),
                "Use a lighter/faster model".to_string(),
                "Check the provider's rate limits".to_string(),
            ])
    }

    /// Authentication error.
    pub fn auth(provider: &str) -> StructuredError {
        StructuredError::new("Authentication failed")
            .with_provider(provider)
            .with_exit_code(401)
            .with_hints(vec![
                "Run 'rco auth login' to authenticate".to_string(),
                "Check your API key is valid".to_string(),
                "Ensure your account has access to the model".to_string(),
            ])
    }

    /// Invalid API key error.
    pub fn invalid_api_key(provider: &str) -> StructuredError {
        StructuredError::new("Invalid API key")
            .with_provider(provider)
            .with_exit_code(401)
            .with_hints(vec![
                "Check your API key is correct".to_string(),
                "Run 'rco auth login' to re-authenticate".to_string(),
                "Verify your API key has the right permissions".to_string(),
            ])
    }

    /// No changes to commit error.
    pub fn no_changes() -> StructuredError {
        StructuredError::new("No changes to commit")
            .with_exit_code(0)
            .with_hints(vec![
                "Stage some changes with 'git add'".to_string(),
                "Use 'git add -A' to stage all changes".to_string(),
                "Check for untracked files".to_string(),
            ])
    }

    /// Not a git repository error.
    pub fn not_git_repo() -> StructuredError {
        StructuredError::new("Not a git repository")
            .with_exit_code(128)
            .with_hints(vec![
                "Initialize a git repository with 'git init'".to_string(),
                "Navigate to a git repository".to_string(),
                "Clone a repository first".to_string(),
            ])
    }

    /// Provider not found error.
    pub fn provider_not_found(provider: &str) -> StructuredError {
        StructuredError::new(&format!("Provider not found: {}", provider))
            .with_exit_code(1)
            .with_hints(vec![
                "Check the provider name is correct".to_string(),
                "Run 'rco config describe' to see available providers".to_string(),
                "Supported providers: openai, anthropic, ollama, gemini, and more".to_string(),
            ])
    }

    /// Model not found error.
    pub fn model_not_found(model: &str, provider: &str) -> StructuredError {
        StructuredError::new(&format!("Model not found: {}", model))
            .with_provider(provider)
            .with_exit_code(1)
            .with_hints(vec![
                "Check the model name is correct".to_string(),
                "Run 'rco model --list' to see available models".to_string(),
                "Try using the default model for this provider".to_string(),
            ])
    }

    /// Network error.
    pub fn network(error: &str) -> StructuredError {
        StructuredError::new("Network error")
            .with_underlying(error)
            .with_hints(vec![
                "Check your internet connection".to_string(),
                "Verify the API endpoint is accessible".to_string(),
                "Check for firewall or proxy issues".to_string(),
                "Try again later".to_string(),
            ])
    }

    /// Timeout error.
    pub fn timeout(provider: &str) -> StructuredError {
        StructuredError::new("Request timed out")
            .with_provider(provider)
            .with_hints(vec![
                "Try again - it may be a temporary issue".to_string(),
                "Use a smaller/faster model".to_string(),
                "Check the provider's status page".to_string(),
            ])
    }
}

#[allow(dead_code)]
/// Print a structured error with the appropriate format.
pub fn print_error(error: &StructuredError, theme: &Theme) {
    match theme.use_colors {
        true => {
            eprintln!("{}", error.display(theme));
        }
        false => {
            eprintln!("Error: {}", error.message);
            if let Some(ref provider) = error.provider {
                eprintln!("Provider: {}", provider);
            }
            if let Some(ref model) = error.model {
                eprintln!("Model: {}", model);
            }
            if let Some(ref underlying) = error.underlying {
                eprintln!("Error: {}", underlying);
            }
            if !error.hints.is_empty() {
                eprintln!("Suggestions:");
                for hint in &error.hints {
                    eprintln!("  - {}", hint);
                }
            }
        }
    }
}

#[allow(dead_code)]
/// Exit with a structured error.
pub fn exit_with_error(error: &StructuredError) -> ! {
    let theme = Theme::new();
    print_error(error, &theme);
    std::process::exit(error.exit_code());
}

#[allow(dead_code)]
/// Convert an anyhow Result to a StructuredError with optional context.
pub fn context<T, E: std::error::Error + Send + Sync>(
    result: Result<T, E>,
    message: &str,
) -> Result<T, StructuredError> {
    result.map_err(|e| StructuredError::new(message).with_underlying(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_error_new() {
        let error = StructuredError::new("Test error");
        assert_eq!(error.message, "Test error");
        assert!(error.provider.is_none());
        assert!(error.hints.is_empty());
        assert_eq!(error.exit_code(), 1);
    }

    #[test]
    fn test_structured_error_with_chain() {
        let error = StructuredError::new("Main error")
            .with_provider("TestProvider")
            .with_model("TestModel")
            .with_underlying("Underlying error")
            .with_hint("Hint 1")
            .with_hint("Hint 2")
            .with_exit_code(42);

        assert_eq!(error.message, "Main error");
        assert_eq!(error.provider, Some("TestProvider".to_string()));
        assert_eq!(error.model, Some("TestModel".to_string()));
        assert_eq!(error.underlying, Some("Underlying error".to_string()));
        assert_eq!(error.hints.len(), 2);
        assert_eq!(error.exit_code(), 42);
    }

    #[test]
    fn test_error_patterns_rate_limit() {
        let error = patterns::rate_limit("Anthropic", "claude-3-5-haiku");
        assert!(error.message.contains("rate limit"));
        assert_eq!(error.provider, Some("Anthropic".to_string()));
        assert_eq!(error.model, Some("claude-3-5-haiku".to_string()));
        assert!(!error.hints.is_empty());
    }

    #[test]
    fn test_error_patterns_auth() {
        let error = patterns::auth("OpenAI");
        assert!(error.message.contains("Authentication"));
        assert_eq!(error.exit_code(), 401);
    }

    #[test]
    fn test_error_patterns_no_changes() {
        let error = patterns::no_changes();
        assert!(error.message.contains("No changes"));
        assert_eq!(error.exit_code(), 0);
    }

    #[test]
    fn test_error_to_json() {
        let error = StructuredError::new("Test")
            .with_hint("Hint 1");
        let json = error.to_json();
        assert!(json.contains("Test"));
        assert!(json.contains("Hint 1"));
    }

    #[test]
    fn test_error_to_markdown() {
        let error = StructuredError::new("Test Error")
            .with_provider("TestProvider")
            .with_hint("Try again");
        let md = error.to_markdown();
        assert!(md.contains("## Error"));
        assert!(md.contains("Test Error"));
        assert!(md.contains("Provider"));
        assert!(md.contains("## Suggestions"));
    }

    #[test]
    fn test_structured_error_display() {
        let theme = Theme::new();
        let error = StructuredError::new("Test error")
            .with_hint("Test hint");
        let display = error.display(&theme);
        assert!(display.contains("Test error"));
        assert!(display.contains("Suggestions"));
        assert!(display.contains("Test hint"));
    }
}
