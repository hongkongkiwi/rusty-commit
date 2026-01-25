#![allow(
    clippy::field_reassign_with_default,
    clippy::assertions_on_constants,
    clippy::overly_complex_bool_expr,
    clippy::useless_vec
)]

use rusty_commit::auth::token_storage::{
    delete_tokens, get_tokens, has_valid_token, store_tokens, TokenStorage,
};
use rusty_commit::config::Config;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

// Mutex to ensure tests that modify global state run sequentially
static TEST_MUTEX: Mutex<()> = Mutex::new(());

// Helper to handle mutex properly without poisoning
fn with_test_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|poisoned| {
        // Clear the poison and continue - this handles poisoned mutexes
        poisoned.into_inner()
    });
    f()
}

// Helper function to set up clean environment for each test
fn setup_clean_env(test_name: &str) -> tempfile::TempDir {
    // Clean up any existing environment variables
    std::env::remove_var("RCO_CONFIG_HOME");
    std::env::remove_var("HOME");

    // Clear all RCO environment variables that might interfere with tests
    let env_vars_to_clear = [
        "RCO_AI_PROVIDER",
        "RCO_API_KEY",
        "RCO_MODEL",
        "RCO_EMOJI",
        "RCO_GITPUSH",
        "RCO_LANGUAGE",
        "RCO_TOKENS_MAX_OUTPUT",
        "RCO_API_URL",
        "RCO_TOKENS_MAX_INPUT",
        "RCO_COMMIT_TYPE",
        "RCO_DESCRIPTION",
        "RCO_DESCRIPTION_CAPITALIZE",
        "RCO_DESCRIPTION_ADD_PERIOD",
        "RCO_DESCRIPTION_MAX_LENGTH",
        "RCO_MESSAGE_TEMPLATE_PLACEHOLDER",
        "RCO_PROMPT_MODULE",
        "RCO_ONE_LINE_COMMIT",
        "RCO_WHY",
        "RCO_OMIT_SCOPE",
        "RCO_ACTION_ENABLED",
        "RCO_TEST_MOCK_TYPE",
        "RCO_HOOK_AUTO_UNCOMMENT",
        "RCO_COMMITLINT_CONFIG",
        "RCO_CUSTOM_PROMPT",
    ];

    for var in &env_vars_to_clear {
        std::env::remove_var(var);
    }

    // Ensure repo-level config is ignored during tests for isolation
    std::env::set_var("RCO_IGNORE_REPO_CONFIG", "1");
    // Disable secure storage to force file-based, deterministic behavior in CI
    std::env::set_var("RCO_DISABLE_SECURE_STORAGE", "1");

    // Create unique temp directory with timestamp to ensure uniqueness
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let unique_name = format!("{}_{}", test_name, timestamp);

    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join(&unique_name);

    // Ensure the config directory exists
    std::fs::create_dir_all(&config_dir).unwrap();

    std::env::set_var("HOME", temp_dir.path());
    std::env::set_var("RCO_CONFIG_HOME", &config_dir);

    // Clean up any existing tokens first
    let _ = delete_tokens();

    temp_dir
}

fn cleanup_env() {
    let _ = delete_tokens();
    std::env::remove_var("RCO_CONFIG_HOME");
    std::env::remove_var("HOME");
    std::env::remove_var("RCO_IGNORE_REPO_CONFIG");
    std::env::remove_var("RCO_DISABLE_SECURE_STORAGE");
}

#[test]
fn test_token_storage_creation() {
    let token = TokenStorage {
        access_token: "test_access_token".to_string(),
        refresh_token: Some("test_refresh_token".to_string()),
        expires_at: Some(9999999999), // Far future
        token_type: "Bearer".to_string(),
        scope: Some("test_scope".to_string()),
    };

    assert_eq!(token.access_token, "test_access_token");
    assert_eq!(token.refresh_token.as_deref(), Some("test_refresh_token"));
    assert_eq!(token.token_type, "Bearer");
    assert!(!token.is_expired());
}

#[test]
fn test_token_expiry() {
    let expired_token = TokenStorage {
        access_token: "test_token".to_string(),
        refresh_token: None,
        expires_at: Some(1), // Past timestamp
        token_type: "Bearer".to_string(),
        scope: None,
    };

    assert!(expired_token.is_expired());

    let future_token = TokenStorage {
        access_token: "test_token".to_string(),
        refresh_token: None,
        expires_at: Some(9999999999), // Far future
        token_type: "Bearer".to_string(),
        scope: None,
    };

    assert!(!future_token.is_expired());

    // Token without expiry should not be expired
    let no_expiry_token = TokenStorage {
        access_token: "test_token".to_string(),
        refresh_token: None,
        expires_at: None,
        token_type: "Bearer".to_string(),
        scope: None,
    };

    assert!(!no_expiry_token.is_expired());
}

#[test]
fn test_token_expires_soon() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Token expires in 2 minutes (should be "expires soon" - within 5 min buffer)
    let soon_token = TokenStorage {
        access_token: "test_token".to_string(),
        refresh_token: None,
        expires_at: Some(now + 120), // 2 minutes from now
        token_type: "Bearer".to_string(),
        scope: None,
    };

    assert!(soon_token.expires_soon());

    // Token expires in 10 minutes (should NOT be "expires soon" - beyond 5 min buffer)
    let later_token = TokenStorage {
        access_token: "test_token".to_string(),
        refresh_token: None,
        expires_at: Some(now + 600), // 10 minutes from now
        token_type: "Bearer".to_string(),
        scope: None,
    };

    assert!(!later_token.expires_soon());
}

#[test]
fn test_store_and_retrieve_tokens() {
    with_test_lock(|| {
        let _temp_dir = setup_clean_env("test_store_and_retrieve_tokens");

        // Store tokens
        let result = store_tokens("access_123", Some("refresh_456"), Some(3600));
        assert!(result.is_ok());

        // Retrieve tokens
        let tokens = get_tokens().unwrap();
        assert!(tokens.is_some());

        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "access_123");
        assert_eq!(tokens.refresh_token.as_deref(), Some("refresh_456"));
        assert_eq!(tokens.token_type, "Bearer");

        cleanup_env();
    });
}

#[test]
fn test_has_valid_token() {
    with_test_lock(|| {
        let _temp_dir = setup_clean_env("test_has_valid_token");

        // No token initially
        assert!(!has_valid_token());

        // Store a valid token
        store_tokens("valid_token", None, Some(3600)).unwrap();
        assert!(has_valid_token());

        // Delete tokens and verify no valid token
        delete_tokens().unwrap();
        assert!(!has_valid_token());

        cleanup_env();
    });
}

#[test]
fn test_delete_tokens() {
    with_test_lock(|| {
        let _temp_dir = setup_clean_env("test_delete_tokens");

        // Store tokens first
        store_tokens("test_token", None, Some(3600)).unwrap();
        assert!(has_valid_token());

        // Delete tokens
        let result = delete_tokens();
        assert!(result.is_ok());
        assert!(!has_valid_token());

        cleanup_env();
    });
}

#[test]
fn test_config_with_different_providers() {
    with_test_lock(|| {
        // Test one representative provider to ensure the mechanism works
        // without complex race condition issues
        let _temp_dir = setup_clean_env("test_config_provider");

        // Ensure environment variables are cleared right before the test
        // to prevent race conditions with parallel tests
        std::env::remove_var("RCO_AI_PROVIDER");
        std::env::remove_var("RCO_API_KEY");

        let mut config = Config::default();
        config.ai_provider = Some("anthropic".to_string());
        config.api_key = Some("test_key".to_string());

        // Test that provider is set correctly
        assert_eq!(config.ai_provider.as_deref(), Some("anthropic"));

        // Test saving and loading
        assert!(config.save().is_ok());

        // Ensure environment is still clean before loading
        std::env::remove_var("RCO_AI_PROVIDER");
        std::env::remove_var("RCO_API_KEY");

        let loaded_config = Config::load().unwrap();

        assert_eq!(loaded_config.ai_provider.as_deref(), Some("anthropic"));
        assert_eq!(loaded_config.api_key.as_deref(), Some("test_key"));

        cleanup_env();
    });
}

#[test]
fn test_oauth_client_creation() {
    use rusty_commit::auth::oauth::OAuthClient;

    let _client = OAuthClient::new();
    // Test that the client was created successfully
    // Since OAuthClient fields are private, we can only test creation doesn't panic
    assert!(true); // OAuth client created successfully
}

#[test]
fn test_provider_specific_configurations() {
    with_test_lock(|| {
        // Test AWS Bedrock with bearer token
        let _temp_dir1 = setup_clean_env("test_provider_specific_configurations_bedrock");
        std::env::set_var("AWS_BEARER_TOKEN_BEDROCK", "test_bedrock_token");
        let mut config = Config::default();
        config.ai_provider = Some("amazon-bedrock".to_string());
        assert!(config.save().is_ok());
        std::env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
        cleanup_env();

        // Test Ollama local configuration
        let _temp_dir2 = setup_clean_env("test_provider_specific_configurations_ollama");
        let mut ollama_config = Config::default();
        ollama_config.ai_provider = Some("ollama".to_string());
        ollama_config.api_url = Some("http://localhost:11434".to_string());
        ollama_config.model = Some("mistral".to_string());
        assert!(ollama_config.save().is_ok());
        cleanup_env();

        // Test Azure OpenAI configuration
        let _temp_dir3 = setup_clean_env("test_provider_specific_configurations_azure");
        let mut azure_config = Config::default();
        azure_config.ai_provider = Some("azure".to_string());
        azure_config.api_key = Some("azure_key".to_string());
        azure_config.api_url = Some("https://test.openai.azure.com".to_string());
        azure_config.model = Some("gpt-35-turbo".to_string());
        assert!(azure_config.save().is_ok());

        let loaded_azure = Config::load().unwrap();
        assert_eq!(loaded_azure.ai_provider.as_deref(), Some("azure"));
        assert_eq!(loaded_azure.api_key.as_deref(), Some("azure_key"));
        assert_eq!(
            loaded_azure.api_url.as_deref(),
            Some("https://test.openai.azure.com")
        );
        cleanup_env();
    });
}

#[test]
fn test_environment_variable_auth() {
    // Test various environment variables that providers might use
    let env_vars = vec![
        ("ANTHROPIC_API_KEY", "anthropic_test"),
        ("OPENAI_API_KEY", "openai_test"),
        ("GROQ_API_KEY", "groq_test"),
        ("DEEPSEEK_API_KEY", "deepseek_test"),
        ("MISTRAL_API_KEY", "mistral_test"),
        ("TOGETHER_API_KEY", "together_test"),
        ("HF_API_KEY", "hf_test"),
        ("GOOGLE_API_KEY", "google_test"),
        ("AWS_BEARER_TOKEN_BEDROCK", "bedrock_test"),
        ("AWS_ACCESS_KEY_ID", "aws_access_test"),
        ("AWS_SECRET_ACCESS_KEY", "aws_secret_test"),
    ];

    for (env_var, test_value) in env_vars {
        std::env::set_var(env_var, test_value);
        let retrieved = std::env::var(env_var).unwrap();
        assert_eq!(retrieved, test_value);
        std::env::remove_var(env_var);
    }
}

#[test]
fn test_config_validation() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let mut config = Config::default();

    // Test setting valid values
    assert!(config.set("RCO_AI_PROVIDER", "openai").is_ok());
    assert!(config.set("RCO_EMOJI", "true").is_ok());
    assert!(config.set("RCO_TOKENS_MAX_INPUT", "8192").is_ok());

    // Test setting invalid values
    assert!(config.set("RCO_EMOJI", "invalid_bool").is_err());
    assert!(config.set("RCO_TOKENS_MAX_INPUT", "not_a_number").is_err());

    // Test setting unknown keys
    assert!(config.set("UNKNOWN_KEY", "value").is_err());
}

#[test]
fn test_secure_vs_file_storage() {
    with_test_lock(|| {
        let _temp_dir = setup_clean_env("test_secure_vs_file_storage");
        let config_dir = std::env::var("RCO_CONFIG_HOME").unwrap();

        // Store tokens (should use file storage in test environment)
        store_tokens("test_token", Some("refresh_token"), Some(3600)).unwrap();

        // Check that the auth file was created
        let auth_file = std::path::PathBuf::from(&config_dir).join("auth.json");

        // The file should exist
        assert!(auth_file.exists());

        // Read and verify the content
        let content = fs::read_to_string(&auth_file).unwrap();
        assert!(content.contains("test_token"));
        assert!(content.contains("refresh_token"));
        assert!(content.contains("Bearer"));

        cleanup_env();
    });
}
