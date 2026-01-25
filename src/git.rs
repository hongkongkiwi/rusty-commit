//! Git repository operations and utilities.
//!
//! This module provides functions for interacting with Git repositories,
//! including staging files, getting diffs, and querying repository status.

use anyhow::{Context, Result};
use git2::{DiffOptions, Repository, StatusOptions};
use std::process::Command;

/// Ensures the current directory is within a Git repository.
///
/// # Errors
///
/// Returns an error if the current directory is not within a Git repository.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// git::assert_git_repo().expect("Not in a git repository");
/// ```
pub fn assert_git_repo() -> Result<()> {
    Repository::open_from_env().context(
        "Not in a git repository. Please run this command from within a git repository.",
    )?;
    Ok(())
}

/// Returns a list of files that are currently staged for commit.
///
/// # Errors
///
/// Returns an error if the repository cannot be accessed.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// let staged = git::get_staged_files().unwrap();
/// for file in staged {
///     println!("Staged: {}", file);
/// }
/// ```
pub fn get_staged_files() -> Result<Vec<String>> {
    let repo = Repository::open_from_env()?;
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(false);

    let statuses = repo.statuses(Some(&mut status_opts))?;

    let mut staged_files = Vec::new();
    for entry in statuses.iter() {
        let status = entry.status();
        if status.contains(git2::Status::INDEX_NEW)
            || status.contains(git2::Status::INDEX_MODIFIED)
            || status.contains(git2::Status::INDEX_DELETED)
            || status.contains(git2::Status::INDEX_RENAMED)
            || status.contains(git2::Status::INDEX_TYPECHANGE)
        {
            if let Some(path) = entry.path() {
                staged_files.push(path.to_string());
            }
        }
    }

    Ok(staged_files)
}

/// Returns a list of all changed files (staged, modified, and untracked).
///
/// # Errors
///
/// Returns an error if the repository cannot be accessed.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// let changed = git::get_changed_files().unwrap();
/// println!("Found {} changed files", changed.len());
/// ```
pub fn get_changed_files() -> Result<Vec<String>> {
    let repo = Repository::open_from_env()?;
    let mut status_opts = StatusOptions::new();
    status_opts.include_untracked(true);

    let statuses = repo.statuses(Some(&mut status_opts))?;

    let mut changed_files = Vec::new();
    for entry in statuses.iter() {
        let status = entry.status();
        // Include files that are modified in working tree or untracked, but not ignored
        if !status.contains(git2::Status::IGNORED) && !status.is_empty() {
            if let Some(path) = entry.path() {
                changed_files.push(path.to_string());
            }
        }
    }

    Ok(changed_files)
}

/// Stages the specified files for commit.
///
/// # Arguments
///
/// * `files` - A slice of file paths to stage
///
/// # Errors
///
/// Returns an error if the git add command fails.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// let files = vec!["src/main.rs".to_string(), "Cargo.toml".to_string()];
/// git::stage_files(&files).unwrap();
/// ```
pub fn stage_files(files: &[String]) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    let output = Command::new("git")
        .arg("add")
        .args(files)
        .output()
        .context("Failed to stage files")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stage files: {}", stderr);
    }

    Ok(())
}

/// Returns the diff of all staged changes.
///
/// This compares the staging area (index) with HEAD to show what will be committed.
///
/// # Errors
///
/// Returns an error if the diff cannot be generated.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// let diff = git::get_staged_diff().unwrap();
/// println!("Staged changes:\n{}", diff);
/// ```
pub fn get_staged_diff() -> Result<String> {
    let repo = Repository::open_from_env()?;

    // Get HEAD tree
    let head = repo.head()?;
    let head_tree = head.peel_to_tree()?;

    // Get index (staging area)
    let mut index = repo.index()?;
    let oid = index.write_tree()?;
    let index_tree = repo.find_tree(oid)?;

    // Create diff between HEAD and index
    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut diff_opts))?;

    // Convert diff to string
    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        diff_text.push_str(content);
        true
    })?;

    Ok(diff_text)
}

/// Returns the absolute path to the repository root.
///
/// # Errors
///
/// Returns an error if not in a Git repository or if the path cannot be determined.
///
/// # Examples
///
/// ```no_run
/// use rusty_commit::git;
///
/// let root = git::get_repo_root().unwrap();
/// println!("Repository root: {}", root);
/// ```
pub fn get_repo_root() -> Result<String> {
    let repo = Repository::open_from_env()?;
    let workdir = repo
        .workdir()
        .context("Could not find repository working directory")?;
    Ok(workdir.to_string_lossy().to_string())
}

/// Returns the current branch name
pub fn get_current_branch() -> Result<String> {
    let repo = Repository::open_from_env()?;
    let head = repo.head()?;
    let branch_name = head
        .shorthand()
        .context("Could not get current branch name")?
        .to_string();
    Ok(branch_name)
}

/// Returns commits between two branches
pub fn get_commits_between(base: &str, head: &str) -> Result<Vec<String>> {
    let repo = Repository::open_from_env()?;

    let base_commit = repo.revparse_single(base)?;
    let head_commit = repo.revparse_single(head)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(head_commit.id())?;
    revwalk.hide(base_commit.id())?;

    let mut commits = Vec::new();
    for oid in revwalk {
        let oid = oid?;
        if let Ok(commit) = repo.find_commit(oid) {
            commits.push(format!(
                "{} - {}",
                commit.id().to_string().chars().take(7).collect::<String>(),
                commit.message().unwrap_or("")
            ));
        }
    }

    Ok(commits)
}

/// Returns the diff between two branches
pub fn get_diff_between(base: &str, head: &str) -> Result<String> {
    let repo = Repository::open_from_env()?;

    let base_commit = repo.revparse_single(base)?;
    let head_commit = repo.revparse_single(head)?;

    let base_tree = base_commit
        .as_tree()
        .ok_or(anyhow::anyhow!("Failed to get base commit tree"))?;
    let head_tree = head_commit
        .as_tree()
        .ok_or(anyhow::anyhow!("Failed to get head commit tree"))?;

    let mut diff_opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(base_tree), Some(head_tree), Some(&mut diff_opts))?;

    let mut diff_text = String::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = std::str::from_utf8(line.content()).unwrap_or("");
        diff_text.push_str(content);
        true
    })?;

    Ok(diff_text)
}

/// Returns the remote URL for the origin
pub fn get_remote_url() -> Result<String> {
    let repo = Repository::open_from_env()?;

    let remote = repo.find_remote("origin")?;
    let url = remote
        .url()
        .context("Could not get remote URL")?
        .to_string();

    Ok(url)
}

/// Returns recent commit messages for style analysis
pub fn get_recent_commit_messages(count: usize) -> Result<Vec<String>> {
    let repo = Repository::open_from_env()?;

    // Get HEAD reference
    let head = repo.head()?;

    // Get the commit
    let commit = head.peel_to_commit()?;

    // Walk back through history
    let mut commits = Vec::new();
    let mut current = Some(commit);

    while let Some(c) = current {
        if commits.len() >= count {
            break;
        }

        if let Some(msg) = c.message() {
            commits.push(msg.to_string());
        }

        current = c.parent(0).ok();
    }

    Ok(commits)
}
