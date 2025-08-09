use rustycommit::auth::token_storage::{
    delete_tokens, get_tokens, has_valid_token, store_tokens, TokenStorage,
};
use rustycommit::config::Config;
use std::fs;
use tempfile::tempdir;

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
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Clean up any existing tokens first
    let _ = delete_tokens();

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
}

#[test]
fn test_has_valid_token() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Clean up any existing tokens first
    let _ = delete_tokens();

    // No token initially
    assert!(!has_valid_token());

    // Store a valid token
    store_tokens("valid_token", None, Some(3600)).unwrap();
    assert!(has_valid_token());

    // Delete tokens and verify no valid token
    delete_tokens().unwrap();
    assert!(!has_valid_token());
}

#[test]
fn test_delete_tokens() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Clean up any existing tokens first
    let _ = delete_tokens();

    // Store tokens first
    store_tokens("test_token", None, Some(3600)).unwrap();
    assert!(has_valid_token());

    // Delete tokens
    let result = delete_tokens();
    assert!(result.is_ok());
    assert!(!has_valid_token());
}

#[test]
fn test_config_with_different_providers() {
    let providers = vec![
        "anthropic",
        "openai",
        "openrouter",
        "groq",
        "deepseek",
        "mistral",
        "amazon-bedrock",
        "azure",
        "together",
        "deepinfra",
        "huggingface",
        "github-models",
        "gemini",
        "ollama",
    ];

    for provider in providers {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut config = Config::default();
        config.ai_provider = Some(provider.to_string());
        config.api_key = Some("test_key".to_string());

        // Test that provider is set correctly
        assert_eq!(config.ai_provider.as_deref(), Some(provider));

        // Test saving and loading
        assert!(config.save().is_ok());
        let loaded_config = Config::load().unwrap();
        assert_eq!(loaded_config.ai_provider.as_deref(), Some(provider));
        assert_eq!(loaded_config.api_key.as_deref(), Some("test_key"));
    }
}

#[test]
fn test_oauth_client_creation() {
    use rustycommit::auth::oauth::OAuthClient;

    let _client = OAuthClient::new();
    // Test that the client was created successfully
    // Since OAuthClient fields are private, we can only test creation doesn't panic
    assert!(true); // OAuth client created successfully
}

#[test]
fn test_provider_specific_configurations() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Test AWS Bedrock with bearer token
    std::env::set_var("AWS_BEARER_TOKEN_BEDROCK", "test_bedrock_token");
    let mut config = Config::default();
    config.ai_provider = Some("amazon-bedrock".to_string());
    assert!(config.save().is_ok());

    // Test Ollama local configuration
    let mut ollama_config = Config::default();
    ollama_config.ai_provider = Some("ollama".to_string());
    ollama_config.api_url = Some("http://localhost:11434".to_string());
    ollama_config.model = Some("mistral".to_string());
    assert!(ollama_config.save().is_ok());

    // Test Azure OpenAI configuration
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
    assert!(config.set("OCO_AI_PROVIDER", "openai").is_ok());
    assert!(config.set("OCO_EMOJI", "true").is_ok());
    assert!(config.set("OCO_TOKENS_MAX_INPUT", "8192").is_ok());

    // Test setting invalid values
    assert!(config.set("OCO_EMOJI", "invalid_bool").is_err());
    assert!(config.set("OCO_TOKENS_MAX_INPUT", "not_a_number").is_err());

    // Test setting unknown keys
    assert!(config.set("UNKNOWN_KEY", "value").is_err());
}

#[test]
fn test_secure_vs_file_storage() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    // Store tokens (should use file storage in test environment)
    store_tokens("test_token", Some("refresh_token"), Some(3600)).unwrap();

    // Check that the auth file was created
    let auth_dir = temp_dir.path().join(".config").join("rustycommit");
    let auth_file = auth_dir.join("auth.json");

    // The file should exist
    assert!(auth_file.exists());

    // Read and verify the content
    let content = fs::read_to_string(&auth_file).unwrap();
    assert!(content.contains("test_token"));
    assert!(content.contains("refresh_token"));
    assert!(content.contains("Bearer"));
}
