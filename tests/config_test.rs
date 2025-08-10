use rusty_commit::config::Config;
use std::fs;
use tempfile::tempdir;

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
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".opencommit");

    // Create a config with test values
    let config_content = r#"
api_key = "test_key"
ai_provider = "openai"
emoji = true
tokens_max_output = 1000
"#;

    // Write the config file directly
    fs::write(&config_path, config_content).unwrap();

    // Override the config path for testing
    std::env::set_var("HOME", temp_dir.path());

    // Load the config back
    let loaded_config = Config::load().unwrap();
    assert_eq!(loaded_config.api_key.as_deref(), Some("test_key"));
    assert_eq!(loaded_config.emoji, Some(true));
    assert_eq!(loaded_config.tokens_max_output, Some(1000));
}

#[test]
fn test_parse_legacy_format() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join(".opencommit");

    // Write legacy format config
    let legacy_content = r#"
RCO_API_KEY=sk-test-key
RCO_AI_PROVIDER=openai
RCO_MODEL=gpt-4
RCO_EMOJI=true
RCO_GITPUSH=false
# This is a comment
RCO_LANGUAGE=en
RCO_TOKENS_MAX_OUTPUT=1000

RCO_DESCRIPTION=false
RCO_WHY=true
RCO_API_CUSTOM_HEADERS=undefined
"#;

    fs::write(&config_path, legacy_content).unwrap();

    // Load config with legacy format
    std::env::set_var("HOME", temp_dir.path());
    let config = Config::load().unwrap();

    assert_eq!(config.api_key.as_deref(), Some("sk-test-key"));
    assert_eq!(config.ai_provider.as_deref(), Some("openai"));
    assert_eq!(config.model.as_deref(), Some("gpt-4"));
    assert_eq!(config.emoji, Some(true));
    assert_eq!(config.gitpush, Some(false));
    assert_eq!(config.language.as_deref(), Some("en"));
    assert_eq!(config.tokens_max_output, Some(1000));
}

#[test]
fn test_set_and_get_config_values() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

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
}

#[test]
fn test_reset_config() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

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
}

#[test]
fn test_legacy_prompt_module_mapping() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let mut config = Config::default();

    // Test mapping of legacy prompt module
    config
        .set("RCO_PROMPT_MODULE", "conventional-commit")
        .unwrap();
    assert_eq!(config.commit_type.as_deref(), Some("conventional"));

    config.set("RCO_PROMPT_MODULE", "gitmoji").unwrap();
    assert_eq!(config.commit_type.as_deref(), Some("gitmoji"));
}

#[test]
fn test_ignore_undefined_values() {
    let temp_dir = tempdir().unwrap();
    std::env::set_var("HOME", temp_dir.path());

    let mut config = Config::default();
    let original_value = config.api_url.clone();

    // These should be ignored
    config.set("RCO_API_URL", "undefined").unwrap();
    assert_eq!(config.api_url, original_value);

    config.set("RCO_API_URL", "null").unwrap();
    assert_eq!(config.api_url, original_value);
}
