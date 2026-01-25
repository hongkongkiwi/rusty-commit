use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::cli::{HookAction, HookCommand};
use crate::git;

const PREPARE_COMMIT_MSG_HOOK: &str = "prepare-commit-msg";
const PREPARE_COMMIT_MSG_CONTENT: &str = r#"#!/bin/sh
# Rusty Commit Git Hook
exec < /dev/tty && rco --hook "$@" || true
"#;

const COMMIT_MSG_HOOK: &str = "commit-msg";
const COMMIT_MSG_CONTENT: &str = r#"#!/bin/sh
# Rusty Commit Git Hook - Non-interactive commit message generation
# This hook generates a commit message and lets you edit it
rco --hook "$@" || true
"#;

const PRECOMMIT_HOOK_CONTENT: &str = r#"- repo: https://github.com/hongkongkiwi/precommit-rusty-commit
  rev: v1.0.18  # TODO: Update with the latest tag
  hooks:
    - id: rusty-commit-msg"#;

pub async fn execute(cmd: HookCommand) -> Result<()> {
    match cmd.action {
        HookAction::PrepareCommitMsg => install_prepare_commit_msg_hook(),
        HookAction::CommitMsg => install_commit_msg_hook(),
        HookAction::Unset => uninstall_all_hooks(),
        HookAction::Precommit { set, unset } => {
            if set {
                install_precommit_hook()?;
            } else if unset {
                uninstall_precommit_hook()?;
            } else {
                anyhow::bail!("Please specify either --set or --unset for pre-commit hooks");
            }
            Ok(())
        }
    }
}

fn install_prepare_commit_msg_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join(PREPARE_COMMIT_MSG_HOOK);

    // Check if hook already exists
    if hook_path.exists() {
        let existing_content = fs::read_to_string(&hook_path)?;
        if existing_content.contains("rco --hook") {
            println!("{}", "prepare-commit-msg hook already installed".yellow());
            return Ok(());
        }

        // Backup existing hook
        let backup_path = hook_path.with_extension("backup");
        fs::copy(&hook_path, &backup_path).context("Failed to backup existing hook")?;
        println!(
            "{}",
            format!("Backed up existing hook to {}", backup_path.display()).yellow()
        );
    }

    // Write the hook file
    fs::write(&hook_path, PREPARE_COMMIT_MSG_CONTENT).context("Failed to write hook file")?;

    // Make it executable
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).context("Failed to make hook executable")?;
    }

    println!(
        "{}",
        "✅ prepare-commit-msg hook installed successfully!".green()
    );
    println!("The hook will run automatically when you use 'git commit'");
    println!("Note: This hook is interactive (prompts for confirmation)");

    Ok(())
}

fn install_commit_msg_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join(COMMIT_MSG_HOOK);

    // Check if hook already exists
    if hook_path.exists() {
        let existing_content = fs::read_to_string(&hook_path)?;
        if existing_content.contains("rco --hook") {
            println!("{}", "commit-msg hook already installed".yellow());
            return Ok(());
        }

        // Backup existing hook
        let backup_path = hook_path.with_extension("backup");
        fs::copy(&hook_path, &backup_path).context("Failed to backup existing hook")?;
        println!(
            "{}",
            format!("Backed up existing hook to {}", backup_path.display()).yellow()
        );
    }

    // Write the hook file
    fs::write(&hook_path, COMMIT_MSG_CONTENT).context("Failed to write hook file")?;

    // Make it executable
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).context("Failed to make hook executable")?;
    }

    println!("{}", "✅ commit-msg hook installed successfully!".green());
    println!("This hook generates commit messages without prompting (non-interactive)");

    Ok(())
}

fn uninstall_all_hooks() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");

    let mut uninstalled = Vec::new();

    // Uninstall prepare-commit-msg
    let prepare_hook_path = hooks_dir.join(PREPARE_COMMIT_MSG_HOOK);
    if prepare_hook_path.exists() {
        let content = fs::read_to_string(&prepare_hook_path)?;
        if content.contains("rco --hook") {
            fs::remove_file(&prepare_hook_path)
                .context("Failed to remove prepare-commit-msg hook")?;
            uninstalled.push("prepare-commit-msg");

            // Restore backup if it exists
            let backup_path = prepare_hook_path.with_extension("backup");
            if backup_path.exists() {
                fs::rename(&backup_path, &prepare_hook_path).ok();
            }
        }
    }

    // Uninstall commit-msg
    let commit_msg_path = hooks_dir.join(COMMIT_MSG_HOOK);
    if commit_msg_path.exists() {
        let content = fs::read_to_string(&commit_msg_path)?;
        if content.contains("rco --hook") {
            fs::remove_file(&commit_msg_path).context("Failed to remove commit-msg hook")?;
            uninstalled.push("commit-msg");

            // Restore backup if it exists
            let backup_path = commit_msg_path.with_extension("backup");
            if backup_path.exists() {
                fs::rename(&backup_path, &commit_msg_path).ok();
            }
        }
    }

    if uninstalled.is_empty() {
        println!("{}", "No Rusty Commit hooks installed".yellow());
    } else {
        println!(
            "{}",
            format!("✅ Uninstalled hooks: {}", uninstalled.join(", ")).green()
        );
    }

    Ok(())
}

pub fn is_hook_called(args: &[String]) -> bool {
    args.iter().any(|arg| arg == "--hook")
}

pub async fn prepare_commit_msg_hook(args: &[String]) -> Result<()> {
    // This function is called when git invokes the prepare-commit-msg hook
    // It should generate a commit message and write it to the file specified in args

    if args.len() < 3 {
        anyhow::bail!("Invalid hook arguments");
    }

    let commit_msg_file = &args[2];

    // Get staged diff
    let diff = git::get_staged_diff()?;
    if diff.is_empty() {
        return Ok(());
    }

    // Generate commit message
    let config = crate::config::Config::load()?;
    let provider = crate::providers::create_provider(&config)?;
    let message = provider
        .generate_commit_message(&diff, None, false, &config)
        .await?;

    // Write to commit message file
    fs::write(commit_msg_file, message).context("Failed to write commit message")?;

    Ok(())
}

fn install_precommit_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let config_path = Path::new(&repo_root).join(".pre-commit-config.yaml");

    // Check if hook already exists
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        if content.contains("hongkongkiwi/precommit-rusty-commit") {
            println!("{}", "Pre-commit hook already installed".yellow());
            println!("To update, run: pre-commit autoupdate");
            return Ok(());
        }
    }

    // Create or append to .pre-commit-config.yaml
    let hook_entry = format!("\n{}", PRECOMMIT_HOOK_CONTENT);
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .and_then(|mut f| f.write_all(hook_entry.as_bytes()))
        .context("Failed to write to .pre-commit-config.yaml")?;

    println!("{}", "✅ Pre-commit hook installed successfully!".green());
    println!("Run 'pre-commit install' to activate the hook");
    println!("Then use 'git commit' as normal - the hook will generate commit messages");

    Ok(())
}

fn uninstall_precommit_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let config_path = Path::new(&repo_root).join(".pre-commit-config.yaml");

    if !config_path.exists() {
        println!("{}", "No .pre-commit-config.yaml found".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(&config_path)?;

    // Check if our hook exists
    if !content.contains("hongkongkiwi/precommit-rusty-commit") {
        println!("{}", "Pre-commit hook not found".yellow());
        return Ok(());
    }

    // Remove the hook entry (lines containing our repo)
    let new_content: Vec<&str> = content
        .lines()
        .filter(|line| {
            !line
                .trim_start()
                .starts_with("hongkongkiwi/precommit-rusty-commit")
                && !line.trim_start().starts_with("rev:")
                && !line.trim_start().starts_with("hooks:")
                && !line.trim_start().starts_with("- id:")
        })
        .collect();

    // Clean up multiple blank lines
    let cleaned: Vec<&str> = new_content
        .iter()
        .filter(|line| !line.trim().is_empty())
        .copied()
        .collect();

    fs::write(&config_path, cleaned.join("\n") + "\n")
        .context("Failed to update .pre-commit-config.yaml")?;

    println!("{}", "✅ Pre-commit hook uninstalled successfully!".green());

    Ok(())
}
