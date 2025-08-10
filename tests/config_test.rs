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

fn setup_test_env(test_name: &str) -> tempfile::TempDir {
    // Clean up any previous environment variables
    std::env::remove_var("HOME");
    std::env::remove_var("RCO_CONFIG_HOME");

    let temp_dir = tempdir().unwrap();
    // Sanitize test name to avoid invalid characters in file paths
    let sanitized_name = test_name.replace("::", "_").replace(" ", "_");
    let config_dir = temp_dir.path().join("config").join(sanitized_name);
    fs::create_dir_all(&config_dir).unwrap();

    std::env::set_var("HOME", temp_dir.path());
    std::env::set_var("RCO_CONFIG_HOME", &config_dir);

    temp_dir
}

fn cleanup_test_env() {
    std::env::remove_var("HOME");
    std::env::remove_var("RCO_CONFIG_HOME");
}

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.ai_provider.as_deref(), Some("openai"));
    assert_eq!(config.model.as_deref(), Some("gpt-3.5-turbo"));
    assert_eq!(config.tokens_max_input, Some(4096));
    assert_eq!(config.tokens_max_output, Some(500));
    assert_eq!(config.commit_type.as_deref(), Some("conventional"));
    assert_eq!(config.emoji, Some(false));
}

#[test]
fn test_save_and_load_config() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_save_and_load_config");

        let mut config = Config::default();
        config.api_key = Some("test_key".to_string());
        config.emoji = Some(true);
        config.tokens_max_output = Some(1000);

        // Save the config
        config.save().unwrap();

        // Load the config back
        let loaded_config = Config::load().unwrap();
        assert_eq!(loaded_config.api_key.as_deref(), Some("test_key"));
        assert_eq!(loaded_config.emoji, Some(true));
        assert_eq!(loaded_config.tokens_max_output, Some(1000));

        cleanup_test_env();
    });
}

#[test]
fn test_parse_legacy_format() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_parse_legacy_format");

        // Set environment variables to simulate legacy format
        std::env::set_var("RCO_API_KEY", "sk-test-key");
        std::env::set_var("RCO_AI_PROVIDER", "openai");
        std::env::set_var("RCO_MODEL", "gpt-4");
        std::env::set_var("RCO_EMOJI", "true");
        std::env::set_var("RCO_GITPUSH", "false");
        std::env::set_var("RCO_LANGUAGE", "en");
        std::env::set_var("RCO_TOKENS_MAX_OUTPUT", "1000");

        let config = Config::load().unwrap();

        assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
        assert_eq!(config.ai_provider.as_deref(), Some("openai"));
        assert_eq!(config.model.as_deref(), Some("gpt-4"));
        assert_eq!(config.emoji, Some(true));
        assert_eq!(config.gitpush, Some(false));
        assert_eq!(config.language.as_deref(), Some("en"));
        assert_eq!(config.tokens_max_output, Some(1000));

        // Clean up environment variables
        std::env::remove_var("RCO_API_KEY");
        std::env::remove_var("RCO_AI_PROVIDER");
        std::env::remove_var("RCO_MODEL");
        std::env::remove_var("RCO_EMOJI");
        std::env::remove_var("RCO_GITPUSH");
        std::env::remove_var("RCO_LANGUAGE");
        std::env::remove_var("RCO_TOKENS_MAX_OUTPUT");

        cleanup_test_env();
    });
}

#[test]
fn test_set_and_get_config_values() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_set_and_get_config_values");

        let mut config = Config::default();

        // Test setting various types
        config.set("RCO_API_KEY", "new_key").unwrap();
        assert_eq!(config.get("RCO_API_KEY").unwrap(), "new_key");

        config.set("RCO_EMOJI", "true").unwrap();
        assert_eq!(config.get("RCO_EMOJI").unwrap(), "true");

        config.set("RCO_TOKENS_MAX_INPUT", "8192").unwrap();
        assert_eq!(config.get("RCO_TOKENS_MAX_INPUT").unwrap(), "8192");

        // Test invalid values
        assert!(config.set("RCO_EMOJI", "not_a_bool").is_err());
        assert!(config.set("RCO_TOKENS_MAX_INPUT", "not_a_number").is_err());

        // Test unknown key
        assert!(config.set("UNKNOWN_KEY", "value").is_err());

        cleanup_test_env();
    });
}

#[test]
fn test_reset_config() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_reset_config");

        let mut config = Config::default();

        // Modify some values
        config.api_key = Some("custom_key".to_string());
        config.emoji = Some(true);
        config.tokens_max_output = Some(1000);

        // Reset specific keys
        config.reset(Some(&vec!["RCO_EMOJI".to_string()])).unwrap();
        assert_eq!(config.api_key.as_deref(), Some("custom_key"));
        assert_eq!(config.emoji, Some(false)); // Reset to default
        assert_eq!(config.tokens_max_output, Some(1000));

        // Reset all
        config.reset(None).unwrap();
        assert_eq!(config.api_key, None);
        assert_eq!(config.emoji, Some(false));
        assert_eq!(config.tokens_max_output, Some(500));

        cleanup_test_env();
    });
}

#[test]
fn test_legacy_prompt_module_mapping() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_legacy_prompt_module_mapping");

        let mut config = Config::default();

        // Test mapping of legacy prompt module
        config
            .set("RCO_PROMPT_MODULE", "conventional-commit")
            .unwrap();
        assert_eq!(config.commit_type.as_deref(), Some("conventional"));

        config.set("RCO_PROMPT_MODULE", "gitmoji").unwrap();
        assert_eq!(config.commit_type.as_deref(), Some("gitmoji"));

        cleanup_test_env();
    });
}

#[test]
fn test_ignore_undefined_values() {
    with_test_lock(|| {
        let _temp_dir = setup_test_env("test_ignore_undefined_values");

        let mut config = Config::default();
        let original_value = config.api_url.clone();

        // These should be ignored
        config.set("RCO_API_URL", "undefined").unwrap();
        assert_eq!(config.api_url, original_value);

        config.set("RCO_API_URL", "null").unwrap();
        assert_eq!(config.api_url, original_value);

        cleanup_test_env();
    });
}
