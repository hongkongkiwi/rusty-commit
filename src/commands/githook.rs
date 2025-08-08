use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::cli::{HookAction, HookCommand};
use crate::git;

const HOOK_NAME: &str = "prepare-commit-msg";
const HOOK_CONTENT: &str = r#"#!/bin/sh
# Rusty Commit Git Hook
exec < /dev/tty && rco --hook "$@" || true
"#;

pub async fn execute(cmd: HookCommand) -> Result<()> {
    match cmd.action {
        HookAction::Set => install_hook(),
        HookAction::Unset => uninstall_hook(),
    }
}

fn install_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let hooks_dir = Path::new(&repo_root).join(".git").join("hooks");

    // Create hooks directory if it doesn't exist
    fs::create_dir_all(&hooks_dir).context("Failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join(HOOK_NAME);

    // Check if hook already exists
    if hook_path.exists() {
        let existing_content = fs::read_to_string(&hook_path)?;
        if existing_content.contains("rco --hook") {
            println!("{}", "Hook already installed".yellow());
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
    fs::write(&hook_path, HOOK_CONTENT).context("Failed to write hook file")?;

    // Make it executable
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).context("Failed to make hook executable")?;
    }

    println!("{}", "✅ Git hook installed successfully!".green());
    println!("The hook will run automatically when you use 'git commit'");

    Ok(())
}

fn uninstall_hook() -> Result<()> {
    git::assert_git_repo()?;

    let repo_root = git::get_repo_root()?;
    let hook_path = Path::new(&repo_root)
        .join(".git")
        .join("hooks")
        .join(HOOK_NAME);

    if !hook_path.exists() {
        println!("{}", "No hook installed".yellow());
        return Ok(());
    }

    let content = fs::read_to_string(&hook_path)?;
    if !content.contains("rco --hook") {
        println!("{}", "Hook exists but is not a Rusty Commit hook".yellow());
        return Ok(());
    }

    fs::remove_file(&hook_path).context("Failed to remove hook file")?;

    // Restore backup if it exists
    let backup_path = hook_path.with_extension("backup");
    if backup_path.exists() {
        fs::rename(&backup_path, &hook_path).context("Failed to restore backup hook")?;
        println!("{}", "Restored previous hook from backup".green());
    }

    println!("{}", "✅ Git hook uninstalled successfully!".green());

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
