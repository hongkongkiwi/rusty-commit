#![allow(
    clippy::field_reassign_with_default,
    clippy::assertions_on_constants,
    clippy::overly_complex_bool_expr,
    clippy::useless_vec
)]

use rusty_commit::config::Config;
use rusty_commit::providers::{build_prompt, create_provider};

#[test]
fn test_build_prompt_conventional() {
    let mut config = Config::default();
    config.commit_type = Some("conventional".to_string());
    config.language = Some("en".to_string());
    config.description_capitalize = Some(true);
    config.description_add_period = Some(false);
    config.description_max_length = Some(100);

    let diff = "diff --git a/test.txt b/test.txt\n+hello world";
    let prompt = build_prompt(diff, None, &config, false);

    assert!(prompt.contains("conventional commit format"));
    assert!(prompt.contains("Capitalize the first letter"));
    assert!(prompt.contains("Do not end the description with a period"));
    assert!(prompt.contains("under 100 characters"));
    assert!(prompt.contains(diff));
}

#[test]
fn test_build_prompt_gitmoji() {
    let mut config = Config::default();
    config.commit_type = Some("gitmoji".to_string());

    let diff = "test diff";
    let prompt = build_prompt(diff, None, &config, false);

    assert!(prompt.contains("GitMoji format"));
    assert!(prompt.contains("🐛"));
    assert!(prompt.contains("✨"));

    // Test full gitmoji spec
    let prompt_full = build_prompt(diff, None, &config, true);
    assert!(prompt_full.contains("full emoji specification"));
}

#[test]
fn test_build_prompt_with_context() {
    let config = Config::default();
    let diff = "test diff";
    let context = "Fixed authentication bug";

    let prompt = build_prompt(diff, Some(context), &config, false);

    assert!(prompt.contains("Additional context: Fixed authentication bug"));
}

#[test]
fn test_build_prompt_with_language() {
    let mut config = Config::default();
    config.language = Some("es".to_string());

    let diff = "test diff";
    let prompt = build_prompt(diff, None, &config, false);

    assert!(prompt.contains("Generate the commit message in es language"));
}

#[test]
fn test_create_provider_openai() {
    let mut config = Config::default();
    config.ai_provider = Some("openai".to_string());
    config.api_key = Some("test_key".to_string());

    let provider = create_provider(&config);
    assert!(provider.is_ok());
}

#[test]
fn test_create_provider_anthropic() {
    let mut config = Config::default();
    config.ai_provider = Some("anthropic".to_string());
    config.api_key = Some("test_key".to_string());

    let provider = create_provider(&config);
    assert!(provider.is_ok());
}

#[test]
fn test_create_provider_ollama() {
    let mut config = Config::default();
    config.ai_provider = Some("ollama".to_string());

    let provider = create_provider(&config);
    assert!(provider.is_ok());
}

#[test]
fn test_create_provider_gemini() {
    let mut config = Config::default();
    config.ai_provider = Some("gemini".to_string());
    config.api_key = Some("test_key".to_string());

    let provider = create_provider(&config);
    assert!(provider.is_ok());
}

#[cfg(feature = "azure")]
#[test]
fn test_create_provider_azure() {
    let mut config = Config::default();
    config.ai_provider = Some("azure".to_string());
    config.api_key = Some("test_key".to_string());
    config.api_url = Some("https://test.openai.azure.com".to_string());

    let provider = create_provider(&config);
    assert!(
        provider.is_ok(),
        "Azure provider creation failed: {:?}",
        provider.err()
    );
}

#[test]
fn test_create_provider_invalid() {
    let mut config = Config::default();
    config.ai_provider = Some("invalid_provider".to_string());

    let provider = create_provider(&config);
    assert!(provider.is_err());
}

#[test]
fn test_create_provider_missing_api_key() {
    let mut config = Config::default();
    config.ai_provider = Some("openai".to_string());
    config.api_key = None;

    let provider = create_provider(&config);
    assert!(provider.is_err());
}
