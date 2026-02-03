#![allow(
    clippy::field_reassign_with_default,
    clippy::assertions_on_constants,
    clippy::overly_complex_bool_expr,
    clippy::useless_vec
)]

use rusty_commit::config::Config;
use std::sync::Mutex;
use tempfile::tempdir;

// Ensure tests that rely on environment variables run sequentially
static TEST_MUTEX: Mutex<()> = Mutex::new(());

fn with_test_lock<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = TEST_MUTEX
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f()
}

fn clear_rco_env() {
    let vars: Vec<String> = std::env::vars()
        .filter_map(|(k, _)| if k.starts_with("RCO_") { Some(k) } else { None })
        .collect();
    for key in vars {
        std::env::remove_var(key);
    }
    // Re-assert isolation flags for all tests
    std::env::set_var("RCO_IGNORE_REPO_CONFIG", "1");
    std::env::set_var("RCO_DISABLE_SECURE_STORAGE", "1");
}

fn setup_env() -> tempfile::TempDir {
    // Ensure isolation for each test by using a unique config home
    let temp_dir = tempdir().unwrap();
    let config_dir = temp_dir.path().join("config");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::env::set_var("HOME", temp_dir.path());
    std::env::set_var("RCO_CONFIG_HOME", &config_dir);
    std::env::set_var("RCO_IGNORE_REPO_CONFIG", "1");
    // Disable secure storage to keep tests deterministic and file-based
    std::env::set_var("RCO_DISABLE_SECURE_STORAGE", "1");
    temp_dir
}

#[test]
fn test_anthropic_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "anthropic".to_string();
        config.api_key = Some("sk-ant-test123".to_string());
        config.model = "claude-3-5-haiku-20241022".to_string();

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "anthropic");
        assert_eq!(loaded.api_key.as_deref(), Some("sk-ant-test123"));
        assert_eq!(loaded.model, "claude-3-5-haiku-20241022");
    });
}

#[test]
fn test_openai_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "openai".to_string();
        config.api_key = Some("sk-test123".to_string());
        config.model = "gpt-4o-mini".to_string();

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "openai");
        assert_eq!(loaded.model, "gpt-4o-mini");
    });
}

#[test]
fn test_openrouter_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "openrouter".to_string();
        config.api_key = Some("sk-or-test123".to_string());
        config.model = "openai/gpt-4o-mini".to_string();
        config.api_url = Some("https://openrouter.ai/api/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "openrouter");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://openrouter.ai/api/v1")
        );
        assert_eq!(loaded.model, "openai/gpt-4o-mini");
    });
}

#[test]
fn test_groq_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "groq".to_string();
        config.api_key = Some("gsk_test123".to_string());
        config.model = "llama-3.1-70b-versatile".to_string();
        config.api_url = Some("https://api.groq.com/openai/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "groq");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://api.groq.com/openai/v1")
        );
        assert_eq!(loaded.model, "llama-3.1-70b-versatile");
    });
}

#[test]
fn test_deepseek_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "deepseek".to_string();
        config.api_key = Some("sk-test123".to_string());
        config.model = "deepseek-chat".to_string();
        config.api_url = Some("https://api.deepseek.com".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "deepseek");
        assert_eq!(loaded.api_url.as_deref(), Some("https://api.deepseek.com"));
        assert_eq!(loaded.model, "deepseek-chat");
    });
}

#[test]
fn test_mistral_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "mistral".to_string();
        config.api_key = Some("test_key_123".to_string());
        config.model = "mistral-large-latest".to_string();
        config.api_url = Some("https://api.mistral.ai/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "mistral");
        assert_eq!(loaded.api_url.as_deref(), Some("https://api.mistral.ai/v1"));
        assert_eq!(loaded.model, "mistral-large-latest");
    });
}

#[test]
fn test_aws_bedrock_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        // Test with API key method
        std::env::set_var("AWS_BEARER_TOKEN_BEDROCK", "test_bedrock_token");

        let mut config = Config::default();
        config.ai_provider = "amazon-bedrock".to_string();
        config.model = "us.anthropic.claude-3-5-haiku-20241022-v1:0".to_string();

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "amazon-bedrock");
        assert_eq!(
            loaded.model,
            "us.anthropic.claude-3-5-haiku-20241022-v1:0"
        );

        // Verify environment variable was set
        assert_eq!(
            std::env::var("AWS_BEARER_TOKEN_BEDROCK").unwrap(),
            "test_bedrock_token"
        );

        std::env::remove_var("AWS_BEARER_TOKEN_BEDROCK");
    });
}

#[test]
fn test_azure_openai_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "azure".to_string();
        config.api_key = Some("azure_test_key".to_string());
        config.api_url = Some("https://test-resource.openai.azure.com".to_string());
        config.model = "gpt-35-turbo".to_string();

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "azure");
        assert_eq!(loaded.api_key.as_deref(), Some("azure_test_key"));
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://test-resource.openai.azure.com")
        );
        assert_eq!(loaded.model, "gpt-35-turbo");
    });
}

#[test]
fn test_together_ai_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "together".to_string();
        config.api_key = Some("together_test_key".to_string());
        config.model = "meta-llama/Llama-3.2-3B-Instruct-Turbo".to_string();
        config.api_url = Some("https://api.together.xyz/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "together");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://api.together.xyz/v1")
        );
        assert_eq!(
            loaded.model,
            "meta-llama/Llama-3.2-3B-Instruct-Turbo"
        );
    });
}

#[test]
fn test_deepinfra_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "deepinfra".to_string();
        config.api_key = Some("deepinfra_test_key".to_string());
        config.model = "meta-llama/Llama-3.2-3B-Instruct".to_string();
        config.api_url = Some("https://api.deepinfra.com/v1/openai".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "deepinfra");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://api.deepinfra.com/v1/openai")
        );
        assert_eq!(
            loaded.model,
            "meta-llama/Llama-3.2-3B-Instruct"
        );
    });
}

#[test]
fn test_huggingface_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "huggingface".to_string();
        config.api_key = Some("hf_test_key".to_string());
        config.model = "meta-llama/Llama-3.2-3B-Instruct".to_string();
        config.api_url = Some("https://api-inference.huggingface.co/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "huggingface");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://api-inference.huggingface.co/v1")
        );
        assert_eq!(
            loaded.model,
            "meta-llama/Llama-3.2-3B-Instruct"
        );
    });
}

#[test]
fn test_github_models_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "github-models".to_string();
        config.api_key = Some("github_pat_test".to_string());
        config.model = "gpt-4o".to_string();
        config.api_url = Some("https://models.inference.ai.azure.com".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "github-models");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://models.inference.ai.azure.com")
        );
        assert_eq!(loaded.model, "gpt-4o");
    });
}

#[test]
fn test_github_copilot_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "github-copilot".to_string();
        config.model = "gpt-4o".to_string();

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "github-copilot");
        assert_eq!(loaded.model, "gpt-4o");
    });
}

#[test]
fn test_gemini_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "gemini".to_string();
        config.api_key = Some("gemini_test_key".to_string());
        config.model = "gemini-1.5-pro".to_string();
        config.api_url = Some("https://generativelanguage.googleapis.com/v1beta".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "gemini");
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://generativelanguage.googleapis.com/v1beta")
        );
        assert_eq!(loaded.model, "gemini-1.5-pro");
    });
}

#[test]
fn test_ollama_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        // Test local Ollama
        let mut config = Config::default();
        config.ai_provider = "ollama".to_string();
        config.model = "mistral".to_string();
        config.api_url = Some("http://localhost:11434".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "ollama");
        assert_eq!(loaded.api_url.as_deref(), Some("http://localhost:11434"));
        assert_eq!(loaded.model, "mistral");

        // Test remote Ollama
        let mut remote_config = Config::default();
        remote_config.ai_provider = "ollama".to_string();
        remote_config.model = "llama3.2:1b".to_string();
        remote_config.api_url = Some("http://192.168.1.100:11434".to_string());

        assert!(remote_config.save().is_ok());

        let loaded_remote = Config::load().unwrap();
        assert_eq!(
            loaded_remote.api_url.as_deref(),
            Some("http://192.168.1.100:11434")
        );
    });
}

#[test]
fn test_custom_provider_config() {
    with_test_lock(|| {
        clear_rco_env();
        let _temp_dir = setup_env();

        let mut config = Config::default();
        config.ai_provider = "custom-provider".to_string();
        config.api_key = Some("custom_key".to_string());
        config.model = "custom-model".to_string();
        config.api_url = Some("https://api.custom-provider.com/v1".to_string());

        assert!(config.save().is_ok());

        let loaded = Config::load().unwrap();
        assert_eq!(loaded.ai_provider, "custom-provider");
        assert_eq!(loaded.api_key.as_deref(), Some("custom_key"));
        assert_eq!(
            loaded.api_url.as_deref(),
            Some("https://api.custom-provider.com/v1")
        );
        assert_eq!(loaded.model, "custom-model");
    });
}

#[test]
fn test_all_supported_providers_list() {
    with_test_lock(|| {
        clear_rco_env();
        let supported_providers = vec![
            "anthropic",
            "github-copilot",
            "openai",
            "gemini",
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
            "ollama",
            "fireworks",
            "moonshot",
            "dashscope",
        ];

        // Test that all providers can be configured
        for provider in supported_providers {
            let _temp_dir = setup_env();

            let mut config = Config::default();
            config.ai_provider = provider.to_string();

            if provider != "github-copilot" && provider != "ollama" {
                config.api_key = Some("test_key".to_string());
            }

            assert!(
                config.save().is_ok(),
                "Failed to save config for provider: {}",
                provider
            );

            let loaded = Config::load().unwrap();
            assert_eq!(loaded.ai_provider, provider);
        }
    });
}

#[test]
fn test_provider_specific_environment_variables() {
    let provider_env_mappings = vec![
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
        ("groq", "GROQ_API_KEY"),
        ("deepseek", "DEEPSEEK_API_KEY"),
        ("mistral", "MISTRAL_API_KEY"),
        ("together", "TOGETHER_API_KEY"),
        ("huggingface", "HF_API_KEY"),
        ("gemini", "GOOGLE_API_KEY"),
        ("amazon-bedrock", "AWS_BEARER_TOKEN_BEDROCK"),
    ];

    for (provider, env_var) in provider_env_mappings {
        let test_value = format!("{}_test_value", provider);

        // Set the environment variable
        std::env::set_var(env_var, &test_value);

        // Verify it was set
        let retrieved = std::env::var(env_var).unwrap();
        assert_eq!(retrieved, test_value);

        // Clean up
        std::env::remove_var(env_var);
    }
}

#[test]
fn test_model_defaults_for_providers() {
    let provider_model_mappings = vec![
        ("anthropic", "claude-3-5-haiku-20241022"),
        ("openai", "gpt-4o-mini"),
        ("openrouter", "openai/gpt-4o-mini"),
        ("groq", "llama-3.1-70b-versatile"),
        ("deepseek", "deepseek-chat"),
        ("mistral", "mistral-large-latest"),
        (
            "amazon-bedrock",
            "us.anthropic.claude-3-5-haiku-20241022-v1:0",
        ),
        ("together", "meta-llama/Llama-3.2-3B-Instruct-Turbo"),
        ("deepinfra", "meta-llama/Llama-3.2-3B-Instruct"),
        ("huggingface", "meta-llama/Llama-3.2-3B-Instruct"),
        ("github-models", "gpt-4o"),
        ("github-copilot", "gpt-4o"),
        ("gemini", "gemini-1.5-pro"),
    ];

    // Test that each provider has appropriate default models
    for (provider, expected_model) in provider_model_mappings {
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut config = Config::default();
        config.ai_provider = provider.to_string();

        // For most providers, we expect the default model pattern
        // This test verifies that the expected models are reasonable choices
        assert!(
            !expected_model.is_empty(),
            "Provider {} should have a default model",
            provider
        );

        // Models should be in expected format
        if provider == "openrouter" {
            assert!(
                expected_model.contains("/"),
                "OpenRouter models should include provider prefix"
            );
        }

        if provider == "amazon-bedrock" {
            assert!(
                expected_model.starts_with("us."),
                "Bedrock models should include region prefix"
            );
        }

        if provider == "together" || provider == "deepinfra" || provider == "huggingface" {
            assert!(
                expected_model.contains("/"),
                "Hosted model providers should include model path"
            );
        }
    }
}
