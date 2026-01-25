#![allow(deprecated)]
#![allow(
    clippy::field_reassign_with_default,
    clippy::assertions_on_constants,
    clippy::overly_complex_bool_expr,
    clippy::useless_vec
)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_update_check() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("update")
        .arg("--check")
        .assert()
        .success()
        .stdout(predicate::str::contains("Current version:"))
        .stdout(predicate::str::contains("Latest version:"))
        .stdout(predicate::str::contains("Install method:"));
}

#[test]
fn test_update_help() {
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("update")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Check for updates and update rusty-commit",
        ))
        .stdout(predicate::str::contains("--check"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--version"));
}

#[test]
fn test_update_with_version() {
    // Test that specifying an invalid version fails gracefully
    let mut cmd = Command::cargo_bin("rco").unwrap();
    cmd.arg("update")
        .arg("--version")
        .arg("invalid.version")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}
