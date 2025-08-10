use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::tempdir;

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

    // Create initial commit to establish main branch
    fs::write(dir.join(".gitignore"), "").expect("Failed to create .gitignore");
    StdCommand::new("git")
        .args(["add", ".gitignore"])
        .current_dir(dir)
        .output()
        .expect("Failed to add .gitignore");

    StdCommand::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(dir)
        .output()
        .expect("Failed to create initial commit");
}

#[test]
fn test_auth_command_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("auth")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Authenticate with Claude using OAuth",
        ));
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
        ("RCO_AI_PROVIDER", "anthropic"),
        ("RCO_AI_PROVIDER", "openai"),
        ("RCO_AI_PROVIDER", "openrouter"),
        ("RCO_AI_PROVIDER", "groq"),
        ("RCO_AI_PROVIDER", "deepseek"),
        ("RCO_AI_PROVIDER", "mistral"),
        ("RCO_AI_PROVIDER", "amazon-bedrock"),
        ("RCO_AI_PROVIDER", "azure"),
        ("RCO_AI_PROVIDER", "together"),
        ("RCO_AI_PROVIDER", "deepinfra"),
        ("RCO_AI_PROVIDER", "huggingface"),
        ("RCO_AI_PROVIDER", "github-models"),
        ("RCO_AI_PROVIDER", "github-copilot"),
        ("RCO_AI_PROVIDER", "gemini"),
        ("RCO_AI_PROVIDER", "ollama"),
        ("RCO_AI_PROVIDER", "fireworks"),
        ("RCO_AI_PROVIDER", "moonshot"),
        ("RCO_AI_PROVIDER", "dashscope"),
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
            .stdout(predicate::str::contains(format!(
                "{} set to: {}",
                key, value
            )));

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
        ("kimi-k2", "moonshot"),
        ("qwen3-coder-32b-instruct", "dashscope"),
    ];

    for (model, provider) in model_tests {
        // Set provider first
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("RCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set model
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("RCO_MODEL={}", model))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "RCO_MODEL set to: {}",
                model
            )));

        // Verify model was set
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("RCO_MODEL")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("RCO_MODEL: {}", model)));
    }
}

#[test]
fn test_config_boolean_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let boolean_tests = vec![
        ("RCO_EMOJI", "true"),
        ("RCO_EMOJI", "false"),
        ("RCO_GITPUSH", "true"),
        ("RCO_GITPUSH", "false"),
        ("RCO_DESCRIPTION_CAPITALIZE", "true"),
        ("RCO_DESCRIPTION_CAPITALIZE", "false"),
    ];

    for (key, value) in boolean_tests {
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "{} set to: {}",
                key, value
            )));
    }
}

#[test]
fn test_config_numeric_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let numeric_tests = vec![
        ("RCO_TOKENS_MAX_INPUT", "8192"),
        ("RCO_TOKENS_MAX_OUTPUT", "1000"),
        ("RCO_DESCRIPTION_MAX_LENGTH", "72"),
    ];

    for (key, value) in numeric_tests {
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("{}={}", key, value))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "{} set to: {}",
                key, value
            )));
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
        ("https://api.fireworks.ai/inference/v1", "fireworks"),
        ("https://api.moonshot.cn/v1", "moonshot"),
        (
            "https://dashscope.aliyuncs.com/compatible-mode/v1",
            "dashscope",
        ),
    ];

    for (url, provider) in url_tests {
        // Set provider first
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("RCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set API URL
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("RCO_API_URL={}", url))
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "RCO_API_URL set to: {}",
                url
            )));
    }
}

#[test]
fn test_config_invalid_values() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let invalid_tests = vec![
        ("RCO_EMOJI", "not_a_boolean"),
        ("RCO_TOKENS_MAX_INPUT", "not_a_number"),
        ("RCO_TOKENS_MAX_OUTPUT", "negative_number"),
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
        .arg("RCO_AI_PROVIDER=openai")
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
        .arg("RCO_EMOJI=true")
        .arg("RCO_GITPUSH=true")
        .assert()
        .success();

    // Reset specific key
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("reset")
        .arg("RCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("Reset keys: RCO_EMOJI"));

    // Verify reset
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("get")
        .arg("RCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("RCO_EMOJI: false"));

    // Reset all
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("reset")
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "All configuration reset to defaults",
        ));
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

    // Test --version (don't pin exact number to avoid churn)
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rco "));
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
        .stdout(predicate::str::contains(
            "Generate commitlint configuration",
        ));
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
            .arg(format!("RCO_AI_PROVIDER={}", provider))
            .assert()
            .success();

        // Set model
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("set")
            .arg(format!("RCO_MODEL={}", model))
            .assert()
            .success();

        // Verify both are set correctly
        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("RCO_AI_PROVIDER")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!(
                "RCO_AI_PROVIDER: {}",
                provider
            )));

        let mut cmd = Command::cargo_bin("rco").unwrap();
        cmd.env("HOME", home)
            .arg("config")
            .arg("get")
            .arg("RCO_MODEL")
            .assert()
            .success()
            .stdout(predicate::str::contains(format!("RCO_MODEL: {}", model)));
    }
}
