use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Rusty Commit - AI-powered commit message generator",
        ));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rustycommit"));
}

#[test]
fn test_config_command_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("config")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Manage Rusty Commit configuration",
        ));
}

#[test]
fn test_hook_command_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("hook")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup git hooks"));
}

#[test]
fn test_commitlint_command_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("commitlint")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generate commitlint configuration",
        ));
}

#[test]
fn test_config_set_and_get() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Set a config value
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("OCO_EMOJI=true")
        .assert()
        .success()
        .stdout(predicate::str::contains("OCO_EMOJI set to: true"));

    // Get the config value
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("get")
        .arg("OCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("OCO_EMOJI: true"));
}

#[test]
fn test_config_reset() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Set a config value
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("OCO_EMOJI=true")
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

    // Verify it was reset to default
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("get")
        .arg("OCO_EMOJI")
        .assert()
        .success()
        .stdout(predicate::str::contains("OCO_EMOJI: false"));
}

#[test]
fn test_config_reset_all() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    // Set multiple config values
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("OCO_EMOJI=true")
        .arg("OCO_GITPUSH=true")
        .assert()
        .success();

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
fn test_invalid_config_key() {
    let temp_dir = tempdir().unwrap();
    let home = temp_dir.path();

    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.env("HOME", home)
        .arg("config")
        .arg("set")
        .arg("INVALID_KEY=value")
        .assert()
        .success()
        .stderr(predicate::str::contains("Failed to set INVALID_KEY"));
}

#[test]
fn test_not_in_git_repo() {
    let temp_dir = tempdir().unwrap();

    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.current_dir(temp_dir.path())
        .env("HOME", temp_dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a git repository"));
}
