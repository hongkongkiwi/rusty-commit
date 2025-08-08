use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;
use std::fs;
use std::process::Command as StdCommand;

fn init_test_git_repo(dir: &std::path::Path) {
    // Initialize git repo
    StdCommand::new("git")
        .arg("init")
        .current_dir(dir)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    StdCommand::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git email");

    StdCommand::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(dir)
        .output()
        .expect("Failed to set git name");
}

#[test]
fn test_auth_command_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("auth")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Authenticate with Claude using OAuth"));
}

#[test]
fn test_auth_status_not_authenticated() {
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", temp_dir.path())
        .arg("auth")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Not authenticated"));
}

#[test]
fn test_auth_logout_when_not_logged_in() {
    let temp_dir = tempdir().unwrap();
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", temp_dir.path())
        .arg("auth")
        .arg("logout")
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully logged out"));
}

#[test]
fn test_config_commands_comprehensive() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Test setting various provider configurations
    let test_cases = vec![
        ("OCO_AI_PROVIDER", "anthropic"),
        ("OCO_AI_PROVIDER", "openai"),
        ("OCO_AI_PROVIDER", "openrouter"),
        ("OCO_AI_PROVIDER", "groq"),
        ("OCO_AI_PROVIDER", "deepseek"),
        ("OCO_AI_PROVIDER", "mistral"),
        ("OCO_AI_PROVIDER", "amazon-bedrock"),
        ("OCO_AI_PROVIDER", "azure"),
        ("OCO_AI_PROVIDER", "together"),
        ("OCO_AI_PROVIDER", "deepinfra"),
        ("OCO_AI_PROVIDER", "huggingface"),
        ("OCO_AI_PROVIDER", "github-models"),
        ("OCO_AI_PROVIDER", "github-copilot"),
        ("OCO_AI_PROVIDER", "gemini"),
        ("OCO_AI_PROVIDER", "ollama"),
    ];

    for (key, value) in test_cases {
        // Set config
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("{} set to: {}", key, value)));

        // Get config
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg(key)
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("{}: {}", key, value)));
    }
}

#[test]
fn test_config_model_settings() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let model_tests = vec![
        ("gpt-4o", "openai"),
        ("claude-3-5-haiku-20241022", "anthropic"),
        ("llama-3.1-70b-versatile", "groq"),
        ("deepseek-chat", "deepseek"),
        ("mistral-large-latest", "mistral"),
        ("gemini-1.5-pro", "gemini"),
        ("meta-llama/Llama-3.2-3B-Instruct", "together"),
    ];

    for (model, provider) in model_tests {
        // Set provider first
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set model
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_MODEL={}", model))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("OCO_MODEL set to: {}", model)));

        // Verify model was set
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("OCO_MODEL")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("OCO_MODEL: {}", model)));
    }
}

#[test]
fn test_config_boolean_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let boolean_tests = vec![
        ("OCO_EMOJI", "true"),
        ("OCO_EMOJI", "false"),
        ("OCO_GITPUSH", "true"),
        ("OCO_GITPUSH", "false"),
        ("OCO_DESCRIPTION_CAPITALIZE", "true"),
        ("OCO_DESCRIPTION_CAPITALIZE", "false"),
    ];

    for (key, value) in boolean_tests {
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("{} set to: {}", key, value)));
    }
}

#[test]
fn test_config_numeric_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let numeric_tests = vec![
        ("OCO_TOKENS_MAX_INPUT", "8192"),
        ("OCO_TOKENS_MAX_OUTPUT", "1000"),
        ("OCO_DESCRIPTION_MAX_LENGTH", "72"),
    ];

    for (key, value) in numeric_tests {
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("{} set to: {}", key, value)));
    }
}

#[test]
fn test_config_api_urls() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let url_tests = vec![
        ("https://api.openai.com/v1", "openai"),
        ("https://api.anthropic.com", "anthropic"),
        ("https://openrouter.ai/api/v1", "openrouter"),
        ("https://api.groq.com/openai/v1", "groq"),
        ("https://api.deepseek.com", "deepseek"),
        ("https://api.mistral.ai/v1", "mistral"),
        ("https://api.together.xyz/v1", "together"),
        ("https://api.deepinfra.com/v1/openai", "deepinfra"),
        ("https://api-inference.huggingface.co/v1", "huggingface"),
        ("https://models.inference.ai.azure.com", "github-models"),
        ("https://generativelanguage.googleapis.com/v1beta", "gemini"),
        ("http://localhost:11434", "ollama"),
    ];

    for (url, provider) in url_tests {
        // Set provider first
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set API URL
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_API_URL={}", url))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("OCO_API_URL set to: {}", url)));
    }
}

#[test]
fn test_config_invalid_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let invalid_tests = vec![
        ("OCO_EMOJI", "not_a_boolean"),
        ("OCO_TOKENS_MAX_INPUT", "not_a_number"),
        ("OCO_TOKENS_MAX_OUTPUT", "negative_number"),
        ("INVALID_KEY", "any_value"),
    ];

    for (key, value) in invalid_tests {
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stderr(predicate::str::contains(format!("Failed to set {}", key)));
    }
}

#[test]
fn test_config_status() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Test status with no configuration
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Secure Storage Status"))
        .stdout(predicate::str::contains("No API key configured"));

    // Set some configuration and test status
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("OCO_AI_PROVIDER=openai")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI Provider: openai"));
}

#[test]
fn test_config_reset() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Set some configuration
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("OCO_EMOJI=true")
        .arg("OCO_GITPUSH=true")
        .assert()
        .success();

    // Reset specific key
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("reset")
        .arg("OCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("Reset keys: OCO_EMOJI"));

    // Verify reset
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("get")
        .arg("OCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("OCO_EMOJI: false"));

    // Reset all
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("reset")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("All configuration reset to defaults"));
}

#[test]
fn test_main_command_options() {
    let temp_dir = tempdir().unwrap();
    init_test_git_repo(temp_dir.path());

    // Test --help
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rusty Commit"))
        .stdout(predicate::str::contains("--fgm"))
        .stdout(predicate::str::contains("--context"))
        .stdout(predicate::str::contains("--yes"));

    // Test --version
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0.0"));
}

#[test]
fn test_hook_commands() {
    let temp_dir = tempdir().unwrap();
    init_test_git_repo(temp_dir.path());

    // Test hook help
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("hook")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup git hooks"));

    // Note: We can't easily test actual hook installation without 
    // risking modifying the test system's git hooks
}

#[test]
fn test_commitlint_commands() {
    let temp_dir = tempdir().unwrap();
    init_test_git_repo(temp_dir.path());

    // Test commitlint help
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("commitlint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate commitlint configuration"));
}

#[test]
fn test_error_handling_no_git_repo() {
    let temp_dir = tempdir().unwrap();
    // Don't initialize git repo

    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("HOME", temp_dir.path())
        .assert()
        .failure(); // Should fail when not in git repo
}

#[test]
fn test_error_handling_no_api_key() {
    let temp_dir = tempdir().unwrap();
    init_test_git_repo(temp_dir.path());

    // Create a staged file for commit
    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();
    StdCommand::new("git")
        .args(["add", "test.txt"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("HOME", temp_dir.path())
        .assert()
        .failure() // Should fail without API key
        .stderr(predicate::str::contains("API key").or(predicate::str::contains("authentication")));
}

#[test]
fn test_multiple_provider_configurations() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Test switching between providers
    let providers = vec![
        ("openai", "gpt-4o-mini"),
        ("anthropic", "claude-3-5-haiku-20241022"),
        ("groq", "llama-3.1-70b-versatile"),
        ("openrouter", "openai/gpt-4o-mini"),
    ];

    for (provider, model) in providers {
        // Set provider
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set model
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("OCO_MODEL={}", model))
            .assert()
            .success();

        // Verify both are set correctly
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("OCO_AI_PROVIDER")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("OCO_AI_PROVIDER: {}", provider)));

        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("OCO_MODEL")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("OCO_MODEL: {}", model)));
    }
}